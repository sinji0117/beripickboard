use bp_g5compat::table::Prefix;
use sqlx::{FromRow, MySqlPool};

/// `g5_member` 중 인증·프로필에 필요한 컬럼. datetime 은 0일자 때문에 문자열로 읽는다.
#[derive(Debug, Clone, FromRow)]
pub struct Member {
    pub mb_id: String,
    pub mb_password: String,
    pub mb_name: String,
    pub mb_nick: String,
    pub mb_email: String,
    pub mb_level: i64,
    pub mb_point: i64,
    pub mb_today_login: String,
    pub mb_intercept_date: String,
    pub mb_leave_date: String,
    pub mb_email_certify: String,
    pub mb_certify: String,
    pub mb_adult: i64,
}

pub async fn find(db: &MySqlPool, p: &Prefix, mb_id: &str) -> sqlx::Result<Option<Member>> {
    sqlx::query_as::<_, Member>(&format!(
        "SELECT mb_id, mb_password, mb_name, mb_nick, mb_email, \
         CAST(mb_level AS SIGNED) AS mb_level, CAST(mb_point AS SIGNED) AS mb_point, \
         CAST(mb_today_login AS CHAR) AS mb_today_login, mb_intercept_date, mb_leave_date, \
         CAST(mb_email_certify AS CHAR) AS mb_email_certify, mb_certify, CAST(mb_adult AS SIGNED) AS mb_adult \
         FROM {} WHERE mb_id = ?",
        p.table("member")
    ))
    .bind(mb_id)
    .fetch_optional(db)
    .await
}

pub async fn update_password(db: &MySqlPool, p: &Prefix, mb_id: &str, hash: &str) -> sqlx::Result<()> {
    sqlx::query(&format!(
        "UPDATE {} SET mb_password = ? WHERE mb_id = ?",
        p.table("member")
    ))
    .bind(hash)
    .bind(mb_id)
    .execute(db)
    .await?;
    Ok(())
}

/// common.php: 오늘 첫 접속이면 접속일시·IP 갱신
pub async fn update_today_login(
    db: &MySqlPool,
    p: &Prefix,
    mb_id: &str,
    ymdhis: &str,
    ip: &str,
) -> sqlx::Result<()> {
    sqlx::query(&format!(
        "UPDATE {} SET mb_today_login = ?, mb_login_ip = ? WHERE mb_id = ?",
        p.table("member")
    ))
    .bind(ymdhis)
    .bind(ip)
    .bind(mb_id)
    .execute(db)
    .await?;
    Ok(())
}
