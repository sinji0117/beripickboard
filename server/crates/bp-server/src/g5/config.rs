use bp_g5compat::table::Prefix;
use sqlx::{FromRow, MySqlPool};

/// `g5_config` 중 현재 사용하는 값.
#[derive(Debug, Clone, FromRow)]
pub struct G5Config {
    pub cf_title: String,
    pub cf_theme: String,
    pub cf_admin: String,
    pub cf_use_point: i64,
    pub cf_login_point: i64,
    pub cf_point_term: i64,
    pub cf_use_email_certify: i64,
    pub cf_search_part: i64,
    pub cf_cert_use: i64,
}

pub async fn load(db: &MySqlPool, p: &Prefix) -> sqlx::Result<G5Config> {
    sqlx::query_as::<_, G5Config>(&format!(
        "SELECT cf_title, cf_theme, cf_admin, CAST(cf_use_point AS SIGNED) AS cf_use_point, \
         CAST(cf_login_point AS SIGNED) AS cf_login_point, CAST(cf_point_term AS SIGNED) AS cf_point_term, \
         CAST(cf_use_email_certify AS SIGNED) AS cf_use_email_certify, \
         CAST(cf_search_part AS SIGNED) AS cf_search_part, CAST(cf_cert_use AS SIGNED) AS cf_cert_use FROM {} LIMIT 1",
        p.table("config")
    ))
    .fetch_one(db)
    .await
}
