//! `insert_point()` 이식 (지급 방향). 사용(음수)·소멸 처리는 v0.0.6 포인트 모듈에서 완성.

use super::{Now, config::G5Config};
use bp_g5compat::{point::expire_date, table::Prefix};
use md5::{Digest, Md5};
use sqlx::MySqlPool;

pub struct PointInput<'a> {
    pub mb_id: &'a str,
    pub point: i32,
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
    /// 포인트 미사용, 0포인트 등 (PHP 반환값 0)
    Skipped,
}

/// PHP와 같은 이름의 MySQL named lock(`g5pt_` + md5)을 잡아 공존 중인 PHP 사이트와도 중복 지급을 막는다.
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
    if input.point < 0 {
        // 사용 포인트(insert_use_point) 차감 로직은 아직 이식 전
        tracing::warn!("음수 포인트는 v0.0.6 에서 지원");
        return Ok(PointResult::Skipped);
    }

    // named lock 은 커넥션 단위이므로 하나의 커넥션에서 처리
    let mut conn = db.acquire().await?;
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
        // get_point_sum (기간 미사용 기준). cf_point_term > 0 의 소멸 선처리는 v0.0.6
        let sum: i64 = sqlx::query_scalar(&format!(
            "SELECT CAST(IFNULL(SUM(po_point), 0) AS SIGNED) FROM {} WHERE mb_id = ?",
            p.table("point")
        ))
        .bind(input.mb_id)
        .fetch_one(&mut *conn)
        .await?;
        let po_mb_point = sum + i64::from(input.point);
        let term = i32::try_from(cfg.cf_point_term).unwrap_or(0);
        let expire = expire_date(now.0.date_naive(), input.point, term, 0);

        sqlx::query(&format!(
            "INSERT INTO {} SET mb_id = ?, po_datetime = ?, po_content = ?, po_point = ?, po_use_point = 0, \
             po_mb_point = ?, po_expired = 0, po_expire_date = ?, po_rel_table = ?, po_rel_id = ?, po_rel_action = ?",
            p.table("point")
        ))
        .bind(input.mb_id)
        .bind(now.ymdhis())
        .bind(input.content)
        .bind(input.point)
        .bind(po_mb_point)
        .bind(expire)
        .bind(input.rel_table)
        .bind(input.rel_id)
        .bind(input.rel_action)
        .execute(&mut *conn)
        .await?;
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
