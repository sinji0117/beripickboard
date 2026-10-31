use bp_g5compat::table::Prefix;
use sqlx::{FromRow, MySqlPool};

/// `g5_board` 중 읽기에 필요한 컬럼 (정수는 SIGNED 로 통일)
#[derive(Debug, Clone, FromRow)]
pub struct Board {
    pub bo_table: String,
    pub gr_id: String,
    pub bo_subject: String,
    pub bo_admin: String,
    pub bo_list_level: i64,
    pub bo_read_level: i64,
    pub bo_write_level: i64,
    pub bo_reply_level: i64,
    pub bo_comment_level: i64,
    pub bo_download_level: i64,
    pub bo_read_point: i64,
    pub bo_download_point: i64,
    pub bo_use_category: i64,
    pub bo_category_list: String,
    pub bo_use_secret: i64,
    pub bo_use_good: i64,
    pub bo_use_nogood: i64,
    pub bo_use_ip_view: i64,
    pub bo_use_list_view: i64,
    pub bo_use_list_content: i64,
    pub bo_use_cert: String,
    pub bo_page_rows: i64,
    pub bo_subject_len: i64,
    pub bo_new: i64,
    pub bo_hot: i64,
    pub bo_skin: String,
    pub bo_gallery_cols: i64,
    pub bo_notice: String,
    pub bo_sort_field: String,
    pub bo_count_write: i64,
    pub bo_order: i64,
}

const BOARD_COLS: &str = "bo_table, gr_id, bo_subject, bo_admin, \
    CAST(bo_list_level AS SIGNED) bo_list_level, CAST(bo_read_level AS SIGNED) bo_read_level, \
    CAST(bo_write_level AS SIGNED) bo_write_level, CAST(bo_reply_level AS SIGNED) bo_reply_level, \
    CAST(bo_comment_level AS SIGNED) bo_comment_level, CAST(bo_download_level AS SIGNED) bo_download_level, \
    CAST(bo_read_point AS SIGNED) bo_read_point, CAST(bo_download_point AS SIGNED) bo_download_point, \
    CAST(bo_use_category AS SIGNED) bo_use_category, bo_category_list, CAST(bo_use_secret AS SIGNED) bo_use_secret, \
    CAST(bo_use_good AS SIGNED) bo_use_good, CAST(bo_use_nogood AS SIGNED) bo_use_nogood, \
    CAST(bo_use_ip_view AS SIGNED) bo_use_ip_view, CAST(bo_use_list_view AS SIGNED) bo_use_list_view, \
    CAST(bo_use_list_content AS SIGNED) bo_use_list_content, CAST(bo_use_cert AS CHAR) bo_use_cert, \
    CAST(bo_page_rows AS SIGNED) bo_page_rows, CAST(bo_subject_len AS SIGNED) bo_subject_len, \
    CAST(bo_new AS SIGNED) bo_new, CAST(bo_hot AS SIGNED) bo_hot, bo_skin, \
    CAST(bo_gallery_cols AS SIGNED) bo_gallery_cols, bo_notice, bo_sort_field, \
    CAST(bo_count_write AS SIGNED) bo_count_write, CAST(bo_order AS SIGNED) bo_order";

pub async fn find(db: &MySqlPool, p: &Prefix, bo_table: &str) -> sqlx::Result<Option<Board>> {
    sqlx::query_as::<_, Board>(&format!(
        "SELECT {BOARD_COLS} FROM {} WHERE bo_table = ?",
        p.table("board")
    ))
    .bind(bo_table)
    .fetch_optional(db)
    .await
}

pub async fn list_all(db: &MySqlPool, p: &Prefix) -> sqlx::Result<Vec<Board>> {
    sqlx::query_as::<_, Board>(&format!(
        "SELECT {BOARD_COLS} FROM {} ORDER BY gr_id, bo_order, bo_table",
        p.table("board")
    ))
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone, FromRow, Default)]
pub struct Group {
    pub gr_id: String,
    pub gr_subject: String,
    pub gr_admin: String,
    pub gr_use_access: i64,
}

pub async fn group(db: &MySqlPool, p: &Prefix, gr_id: &str) -> sqlx::Result<Group> {
    Ok(sqlx::query_as::<_, Group>(&format!(
        "SELECT gr_id, gr_subject, gr_admin, CAST(gr_use_access AS SIGNED) gr_use_access FROM {} WHERE gr_id = ?",
        p.table("group")
    ))
    .bind(gr_id)
    .fetch_optional(db)
    .await?
    .unwrap_or_default())
}

pub async fn is_group_member(db: &MySqlPool, p: &Prefix, gr_id: &str, mb_id: &str) -> sqlx::Result<bool> {
    let n: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM {} WHERE gr_id = ? AND mb_id = ?",
        p.table("group_member")
    ))
    .bind(gr_id)
    .bind(mb_id)
    .fetch_one(db)
    .await?;
    Ok(n > 0)
}
