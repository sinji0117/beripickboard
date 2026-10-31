//! 포인트 (lib/common.lib.php `insert_point`, `insert_use_point`, `get_point_sum`, `get_expire_point`) 이식.
//! PHP 와 같은 named lock 이름을 사용하므로 공존 중인 PHP 사이트와도 중복 지급·차감이 방지된다.

use super::{Now, config::G5Config};
use bp_g5compat::{point::expire_date, table::Prefix};
use md5::{Digest, Md5};
use sqlx::{MySqlConnection, MySqlPool};

pub struct PointInput<'a> {
    pub mb_id: &'a str,
    pub point: i64,
    pub content: &'a str,
    pub rel_table: &'a str,
    pub rel_id: &'a str,
    pub rel_action: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PointResult {
    Inserted,
    /// 같은 rel_table/rel_id/rel_action 이 이미 있음 (PHP 반환값 -1)
    Duplicate,
    /// 포인트 미사용, 0포인트, 비회원 등 (PHP 반환값 0)
    Skipped,
}

/// `get_point_sum()` — cf_point_term > 0 이면 소멸 포인트를 먼저 처리한다.
pub async fn point_sum(
    conn: &mut MySqlConnection,
    p: &Prefix,
    cfg: &G5Config,
    now: Now,
    mb_id: &str,
) -> sqlx::Result<i64> {
    let pt = p.table("point");
    if cfg.cf_point_term > 0 {
        let expire: i64 = sqlx::query_scalar(&format!(
            "SELECT CAST(IFNULL(SUM(po_point - po_use_point), 0) AS SIGNED) FROM {pt} \
             WHERE mb_id = ? AND po_expired = '0' AND po_expire_date <> '9999-12-31' AND po_expire_date < ?"
        ))
        .bind(mb_id)
        .bind(now.ymd())
        .fetch_one(&mut *conn)
        .await?;
        if expire > 0 {
            let mb_point: i64 = sqlx::query_scalar(&format!(
                "SELECT CAST(mb_point AS SIGNED) FROM {} WHERE mb_id = ?",
                p.table("member")
            ))
            .bind(mb_id)
            .fetch_optional(&mut *conn)
            .await?
            .unwrap_or(0);
            let rel_action = format!("expire-{}", uniqid(now));
            sqlx::query(&format!(
                "INSERT INTO {pt} SET mb_id = ?, po_datetime = ?, po_content = '포인트 소멸', po_point = ?, \
                 po_use_point = 0, po_mb_point = ?, po_expired = 1, po_expire_date = ?, \
                 po_rel_table = '@expire', po_rel_id = ?, po_rel_action = ?"
            ))
            .bind(mb_id)
            .bind(now.ymdhis())
            .bind(-expire)
            .bind(mb_point - expire)
            .bind(now.ymd())
            .bind(mb_id)
            .bind(rel_action)
            .execute(&mut *conn)
            .await?;
            use_point(conn, p, cfg, mb_id, -expire).await?;
        }
        sqlx::query(&format!(
            "UPDATE {pt} SET po_expired = '1' WHERE mb_id = ? AND po_expired <> '1' \
             AND po_expire_date <> '9999-12-31' AND po_expire_date < ?"
        ))
        .bind(mb_id)
        .bind(now.ymd())
        .execute(&mut *conn)
        .await?;
    }
    sqlx::query_scalar(&format!(
        "SELECT CAST(IFNULL(SUM(po_point), 0) AS SIGNED) FROM {pt} WHERE mb_id = ?"
    ))
    .bind(mb_id)
    .fetch_one(&mut *conn)
    .await
}

/// PHP uniqid('') 와 비슷한 13자리 16진수 (마이크로초 기반)
fn uniqid(now: Now) -> String {
    let us = now.0.timestamp_micros();
    format!("{:08x}{:05x}", us / 1_000_000, us % 1_000_000)
}

/// `insert_use_point()` — 사용한 포인트만큼 오래된 적립분부터 po_use_point 를 채운다(FIFO).
async fn use_point(
    conn: &mut MySqlConnection,
    p: &Prefix,
    cfg: &G5Config,
    mb_id: &str,
    point: i64,
) -> sqlx::Result<()> {
    let pt = p.table("point");
    let order = if cfg.cf_point_term != 0 {
        "po_expire_date ASC, po_id ASC"
    } else {
        "po_id ASC"
    };
    let mut remaining = point.abs();
    let mut guard = 1000;
    while remaining > 0 && guard > 0 {
        guard -= 1;
        let row: Option<(i64, i64, i64)> = sqlx::query_as(&format!(
            "SELECT CAST(po_id AS SIGNED), CAST(po_point AS SIGNED), CAST(po_use_point AS SIGNED) FROM {pt} \
             WHERE mb_id = ? AND po_expired = '0' AND po_point > po_use_point ORDER BY {order} LIMIT 1"
        ))
        .bind(mb_id)
        .fetch_optional(&mut *conn)
        .await?;
        let Some((po_id, po_point, po_use)) = row else {
            break;
        };
        let available = po_point - po_use;
        if available > remaining {
            let r = sqlx::query(&format!(
                "UPDATE {pt} SET po_use_point = po_use_point + ? WHERE po_id = ? AND po_expired = '0' \
                 AND (po_point - po_use_point) > ?"
            ))
            .bind(remaining)
            .bind(po_id)
            .bind(remaining)
            .execute(&mut *conn)
            .await?;
            if r.rows_affected() > 0 {
                break;
            }
        } else {
            let r = sqlx::query(&format!(
                "UPDATE {pt} SET po_use_point = po_use_point + ?, po_expired = '100' \
                 WHERE po_id = ? AND po_use_point = ? AND po_expired = '0'"
            ))
            .bind(available)
            .bind(po_id)
            .bind(po_use)
            .execute(&mut *conn)
            .await?;
            if r.rows_affected() > 0 {
                remaining -= available;
            }
        }
    }
    Ok(())
}

/// `insert_point()` 이식 (지급·차감 모두)
pub async fn insert_point(
    db: &MySqlPool,
    p: &Prefix,
    cfg: &G5Config,
    now: Now,
    input: PointInput<'_>,
) -> sqlx::Result<PointResult> {
    if cfg.cf_use_point == 0 || input.point == 0 || input.mb_id.is_empty() {
        return Ok(PointResult::Skipped);
    }
    // named lock 은 커넥션 단위이므로 하나의 커넥션에서 처리
    let mut conn = db.acquire().await?;
    let exists: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM {} WHERE mb_id = ?",
        p.table("member")
    ))
    .bind(input.mb_id)
    .fetch_one(&mut *conn)
    .await?;
    if exists == 0 {
        return Ok(PointResult::Skipped);
    }
    let has_rel = !(input.rel_table.is_empty() && input.rel_id.is_empty() && input.rel_action.is_empty());
    let lock_name = format!(
        "g5pt_{}",
        hex::encode(Md5::digest(
            format!(
                "{}|{}|{}|{}",
                input.mb_id, input.rel_table, input.rel_id, input.rel_action
            )
            .as_bytes()
        ))
    );
    if has_rel {
        sqlx::query("SELECT GET_LOCK(?, 5)")
            .bind(&lock_name)
            .execute(&mut *conn)
            .await?;
    }

    let result = async {
        let mb_point = point_sum(&mut conn, p, cfg, now, input.mb_id).await?;
        if has_rel {
            let cnt: i64 = sqlx::query_scalar(&format!(
                "SELECT COUNT(*) FROM {} WHERE mb_id = ? AND po_rel_table = ? AND po_rel_id = ? AND po_rel_action = ?",
                p.table("point")
            ))
            .bind(input.mb_id)
            .bind(input.rel_table)
            .bind(input.rel_id)
            .bind(input.rel_action)
            .fetch_one(&mut *conn)
            .await?;
            if cnt > 0 {
                return Ok(PointResult::Duplicate);
            }
        }
        let term = i32::try_from(cfg.cf_point_term).unwrap_or(0);
        let point32 = i32::try_from(input.point).unwrap_or(0);
        let expire = expire_date(now.0.date_naive(), point32, term, 0);
        let expired = i32::from(input.point < 0);
        let po_mb_point = mb_point + input.point;

        sqlx::query(&format!(
            "INSERT INTO {} SET mb_id = ?, po_datetime = ?, po_content = ?, po_point = ?, po_use_point = 0, \
             po_mb_point = ?, po_expired = ?, po_expire_date = ?, po_rel_table = ?, po_rel_id = ?, po_rel_action = ?",
            p.table("point")
        ))
        .bind(input.mb_id)
        .bind(now.ymdhis())
        .bind(input.content)
        .bind(input.point)
        .bind(po_mb_point)
        .bind(expired)
        .bind(expire)
        .bind(input.rel_table)
        .bind(input.rel_id)
        .bind(input.rel_action)
        .execute(&mut *conn)
        .await?;
        if input.point < 0 {
            use_point(&mut conn, p, cfg, input.mb_id, input.point).await?;
        }
        sqlx::query(&format!("UPDATE {} SET mb_point = ? WHERE mb_id = ?", p.table("member")))
            .bind(po_mb_point)
            .bind(input.mb_id)
            .execute(&mut *conn)
            .await?;
        Ok(PointResult::Inserted)
    }
    .await;

    if has_rel {
        let _ = sqlx::query("SELECT RELEASE_LOCK(?)")
            .bind(&lock_name)
            .execute(&mut *conn)
            .await;
    }
    result
}
