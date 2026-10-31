//! `g5_write_{bo_table}` / `g5_board_file` 읽기

use bp_g5compat::board::{Bind, SqlPart};
use bp_g5compat::table::Prefix;
use sqlx::{FromRow, MySql, MySqlPool, query::QueryAs};

#[derive(Debug, Clone, FromRow)]
pub struct WriteRow {
    pub wr_id: i64,
    pub wr_num: i64,
    pub wr_reply: String,
    pub wr_parent: i64,
    pub wr_is_comment: i64,
    pub wr_comment: i64,
    pub wr_comment_reply: String,
    pub ca_name: String,
    pub wr_option: String,
    pub wr_subject: String,
    pub wr_content: String,
    pub wr_link1: String,
    pub wr_link2: String,
    pub wr_hit: i64,
    pub wr_good: i64,
    pub wr_nogood: i64,
    pub mb_id: String,
    pub wr_password: String,
    pub wr_name: String,
    pub wr_datetime: String,
    pub wr_file: i64,
    pub wr_last: String,
    pub wr_ip: String,
}

pub const WRITE_COLS: &str = "CAST(wr_id AS SIGNED) wr_id, CAST(wr_num AS SIGNED) wr_num, wr_reply, \
    CAST(wr_parent AS SIGNED) wr_parent, CAST(wr_is_comment AS SIGNED) wr_is_comment, \
    CAST(wr_comment AS SIGNED) wr_comment, wr_comment_reply, ca_name, CAST(wr_option AS CHAR) wr_option, \
    wr_subject, wr_content, wr_link1, wr_link2, CAST(wr_hit AS SIGNED) wr_hit, CAST(wr_good AS SIGNED) wr_good, \
    CAST(wr_nogood AS SIGNED) wr_nogood, mb_id, wr_password, wr_name, CAST(wr_datetime AS CHAR) wr_datetime, \
    CAST(wr_file AS SIGNED) wr_file, wr_last, wr_ip";

fn bind_all<'q, O>(
    mut q: QueryAs<'q, MySql, O, sqlx::mysql::MySqlArguments>,
    binds: &'q [Bind],
) -> QueryAs<'q, MySql, O, sqlx::mysql::MySqlArguments> {
    for b in binds {
        q = match b {
            Bind::Str(s) => q.bind(s.as_str()),
            Bind::Int(i) => q.bind(*i),
        };
    }
    q
}

pub async fn get(db: &MySqlPool, table: &str, wr_id: i64) -> sqlx::Result<Option<WriteRow>> {
    sqlx::query_as::<_, WriteRow>(&format!("SELECT {WRITE_COLS} FROM {table} WHERE wr_id = ?"))
        .bind(wr_id)
        .fetch_optional(db)
        .await
}

pub async fn get_many(db: &MySqlPool, table: &str, ids: &[i64]) -> sqlx::Result<Vec<WriteRow>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let marks = vec!["?"; ids.len()].join(",");
    let sql = format!("SELECT {WRITE_COLS} FROM {table} WHERE wr_id IN ({marks})");
    let mut q = sqlx::query_as::<_, WriteRow>(&sql);
    for id in ids {
        q = q.bind(*id);
    }
    let mut rows = q.fetch_all(db).await?;
    // 요청한 순서 유지
    rows.sort_by_key(|r| ids.iter().position(|i| *i == r.wr_id).unwrap_or(usize::MAX));
    Ok(rows)
}

/// 일반 목록 (공지 제외, 원글만)
pub async fn list_plain(
    db: &MySqlPool,
    table: &str,
    exclude: &[i64],
    order: &str,
    from: i64,
    rows: i64,
) -> sqlx::Result<Vec<WriteRow>> {
    let not_in = if exclude.is_empty() {
        String::new()
    } else {
        format!(" AND wr_id NOT IN ({})", vec!["?"; exclude.len()].join(","))
    };
    let sql = format!(
        "SELECT {WRITE_COLS} FROM {table} WHERE wr_is_comment = 0{not_in} ORDER BY {order} LIMIT ?, ?"
    );
    let mut q = sqlx::query_as::<_, WriteRow>(&sql);
    for id in exclude {
        q = q.bind(*id);
    }
    q.bind(from).bind(rows).fetch_all(db).await
}

/// 검색 구간(spt) 최소값 = MIN(wr_num)
pub async fn min_wr_num(db: &MySqlPool, table: &str) -> sqlx::Result<i64> {
    sqlx::query_scalar(&format!(
        "SELECT CAST(IFNULL(MIN(wr_num), 0) AS SIGNED) FROM {table}"
    ))
    .fetch_one(db)
    .await
}

fn with_spt(search: &SqlPart, spt: i64, part: i64) -> SqlPart {
    let mut s = search.clone();
    s.sql.push_str(" AND (wr_num BETWEEN ? AND ?)");
    s.binds.push(Bind::Int(spt));
    s.binds.push(Bind::Int(spt + part));
    s
}

pub async fn search_count(
    db: &MySqlPool,
    table: &str,
    search: &SqlPart,
    spt: i64,
    part: i64,
) -> sqlx::Result<i64> {
    let s = with_spt(search, spt, part);
    let sql = format!("SELECT COUNT(DISTINCT wr_parent) FROM {table} WHERE {}", s.sql);
    let mut q = sqlx::query_scalar::<_, i64>(&sql);
    for b in &s.binds {
        q = match b {
            Bind::Str(v) => q.bind(v.as_str()),
            Bind::Int(i) => q.bind(*i),
        };
    }
    q.fetch_one(db).await
}

/// 검색 목록: 댓글 일치도 원글로 묶음 (DISTINCT wr_parent)
#[allow(clippy::too_many_arguments)]
pub async fn search_list(
    db: &MySqlPool,
    table: &str,
    search: &SqlPart,
    spt: i64,
    part: i64,
    order: &str,
    from: i64,
    rows: i64,
) -> sqlx::Result<Vec<WriteRow>> {
    #[derive(FromRow)]
    struct P {
        wr_parent: i64,
    }
    let s = with_spt(search, spt, part);
    let sql = format!(
        "SELECT DISTINCT CAST(wr_parent AS SIGNED) wr_parent FROM {table} WHERE {} ORDER BY {order} LIMIT ?, ?",
        s.sql
    );
    let ids: Vec<i64> = bind_all(sqlx::query_as::<_, P>(&sql), &s.binds)
        .bind(from)
        .bind(rows)
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|p| p.wr_parent)
        .filter(|p| *p > 0)
        .collect();
    get_many(db, table, &ids).await
}

pub async fn comments(db: &MySqlPool, table: &str, wr_id: i64) -> sqlx::Result<Vec<WriteRow>> {
    sqlx::query_as::<_, WriteRow>(&format!(
        "SELECT {WRITE_COLS} FROM {table} WHERE wr_parent = ? AND wr_is_comment = 1 ORDER BY wr_comment, wr_comment_reply"
    ))
    .bind(wr_id)
    .fetch_all(db)
    .await
}

/// 비밀 답글의 원글 작성자 (board.php is_owner)
pub async fn root_author(db: &MySqlPool, table: &str, wr_num: i64) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar(&format!(
        "SELECT mb_id FROM {table} WHERE wr_num = ? AND wr_reply = '' AND wr_is_comment = 0 LIMIT 1"
    ))
    .bind(wr_num)
    .fetch_optional(db)
    .await
}

#[derive(Debug, FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Neighbor {
    pub wr_id: i64,
    pub wr_subject: String,
    pub wr_datetime: String,
}

/// view.php 이전글/다음글 (bo_use_list_view 가 꺼져 있을 때)
pub async fn neighbors(
    db: &MySqlPool,
    table: &str,
    w: &WriteRow,
    search: Option<&SqlPart>,
) -> sqlx::Result<(Option<Neighbor>, Option<Neighbor>)> {
    let cols = "CAST(wr_id AS SIGNED) wr_id, wr_subject, CAST(wr_datetime AS CHAR) wr_datetime";
    let extra = search.map(|s| format!(" AND {}", s.sql)).unwrap_or_default();
    let empty = vec![];
    let sb = search.map_or(&empty, |s| &s.binds);
    let one = |cond: &str, order: &str| {
        format!(
            "SELECT {cols} FROM {table} WHERE wr_is_comment = 0 AND {cond}{extra} ORDER BY {order} LIMIT 1"
        )
    };
    let q1 = one("wr_num = ? AND wr_reply < ?", "wr_num DESC, wr_reply DESC");
    let mut prev = bind_all(
        sqlx::query_as::<_, Neighbor>(&q1)
            .bind(w.wr_num)
            .bind(&w.wr_reply),
        sb,
    )
    .fetch_optional(db)
    .await?;
    if prev.is_none() {
        let q = one("wr_num < ?", "wr_num DESC, wr_reply DESC");
        prev = bind_all(sqlx::query_as::<_, Neighbor>(&q).bind(w.wr_num), sb)
            .fetch_optional(db)
            .await?;
    }
    let q2 = one("wr_num = ? AND wr_reply > ?", "wr_num, wr_reply");
    let mut next = bind_all(
        sqlx::query_as::<_, Neighbor>(&q2)
            .bind(w.wr_num)
            .bind(&w.wr_reply),
        sb,
    )
    .fetch_optional(db)
    .await?;
    if next.is_none() {
        let q = one("wr_num > ?", "wr_num, wr_reply");
        next = bind_all(sqlx::query_as::<_, Neighbor>(&q).bind(w.wr_num), sb)
            .fetch_optional(db)
            .await?;
    }
    Ok((prev, next))
}

pub async fn increment_hit(db: &MySqlPool, table: &str, wr_id: i64) -> sqlx::Result<()> {
    sqlx::query(&format!("UPDATE {table} SET wr_hit = wr_hit + 1 WHERE wr_id = ?"))
        .bind(wr_id)
        .execute(db)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, FromRow)]
pub struct FileRow {
    pub bf_no: i64,
    pub bf_source: String,
    pub bf_file: String,
    pub bf_download: i64,
    pub bf_content: String,
    pub bf_filesize: i64,
    pub bf_width: i64,
    pub bf_height: i64,
    pub bf_type: i64,
    pub bf_datetime: String,
}

const FILE_COLS: &str = "CAST(bf_no AS SIGNED) bf_no, bf_source, bf_file, CAST(bf_download AS SIGNED) bf_download, \
    bf_content, CAST(bf_filesize AS SIGNED) bf_filesize, CAST(bf_width AS SIGNED) bf_width, \
    CAST(bf_height AS SIGNED) bf_height, CAST(bf_type AS SIGNED) bf_type, CAST(bf_datetime AS CHAR) bf_datetime";

pub async fn files(db: &MySqlPool, p: &Prefix, bo_table: &str, wr_id: i64) -> sqlx::Result<Vec<FileRow>> {
    sqlx::query_as::<_, FileRow>(&format!(
        "SELECT {FILE_COLS} FROM {} WHERE bo_table = ? AND wr_id = ? AND bf_file <> '' ORDER BY bf_no",
        p.table("board_file")
    ))
    .bind(bo_table)
    .bind(wr_id)
    .fetch_all(db)
    .await
}

pub async fn file(
    db: &MySqlPool,
    p: &Prefix,
    bo_table: &str,
    wr_id: i64,
    no: i64,
) -> sqlx::Result<Option<FileRow>> {
    sqlx::query_as::<_, FileRow>(&format!(
        "SELECT {FILE_COLS} FROM {} WHERE bo_table = ? AND wr_id = ? AND bf_no = ?",
        p.table("board_file")
    ))
    .bind(bo_table)
    .bind(wr_id)
    .bind(no)
    .fetch_optional(db)
    .await
}

pub async fn increment_download(
    db: &MySqlPool,
    p: &Prefix,
    bo_table: &str,
    wr_id: i64,
    no: i64,
) -> sqlx::Result<()> {
    sqlx::query(&format!(
        "UPDATE {} SET bf_download = bf_download + 1 WHERE bo_table = ? AND wr_id = ? AND bf_no = ?",
        p.table("board_file")
    ))
    .bind(bo_table)
    .bind(wr_id)
    .bind(no)
    .execute(db)
    .await?;
    Ok(())
}

/// 인기검색어 기록 (중복은 UNIQUE 키로 무시)
pub async fn insert_popular(db: &MySqlPool, p: &Prefix, words: &[String], ymd: &str, ip: &str) {
    for w in words {
        let _ = sqlx::query(&format!(
            "INSERT IGNORE INTO {} SET pp_word = ?, pp_date = ?, pp_ip = ?",
            p.table("popular")
        ))
        .bind(w)
        .bind(ymd)
        .bind(ip)
        .execute(db)
        .await;
    }
}
