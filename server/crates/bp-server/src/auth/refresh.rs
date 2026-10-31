//! 리프레시 토큰 저장소 (bp_refresh_token). 회전(rotation) + 재사용 감지 시 패밀리 전체 폐기.

use chrono::{DateTime, NaiveDateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, MySqlPool};

pub fn random_hex(bytes: usize) -> anyhow::Result<String> {
    let mut b = vec![0u8; bytes];
    getrandom::fill(&mut b).map_err(|e| anyhow::anyhow!("난수 생성 실패: {e}"))?;
    Ok(hex::encode(b))
}

pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

#[derive(Debug, FromRow)]
pub struct StoredToken {
    pub family_id: String,
    pub mb_id: String,
    pub auto_login: i8,
    pub expires_at: NaiveDateTime,
    pub revoked_at: Option<NaiveDateTime>,
}

pub struct NewToken<'a> {
    pub family_id: &'a str,
    pub mb_id: &'a str,
    pub auto_login: bool,
    pub user_agent: &'a str,
    pub ip: &'a str,
    pub ttl_secs: i64,
}

/// 새 토큰을 만들어 저장하고 원문을 돌려준다.
pub async fn create(db: &MySqlPool, t: NewToken<'_>, now: DateTime<Utc>) -> anyhow::Result<String> {
    let token = random_hex(32)?;
    let ua: String = t.user_agent.chars().take(255).collect();
    sqlx::query(
        "INSERT INTO bp_refresh_token (token_hash, family_id, mb_id, auto_login, user_agent, ip, created_at, expires_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(hash_token(&token))
    .bind(t.family_id)
    .bind(t.mb_id)
    .bind(t.auto_login)
    .bind(ua)
    .bind(t.ip.chars().take(45).collect::<String>())
    .bind(now.naive_utc())
    .bind((now + chrono::Duration::seconds(t.ttl_secs)).naive_utc())
    .execute(db)
    .await?;
    Ok(token)
}

pub async fn find(db: &MySqlPool, token: &str) -> sqlx::Result<Option<StoredToken>> {
    sqlx::query_as::<_, StoredToken>(
        "SELECT family_id, mb_id, auto_login, expires_at, revoked_at FROM bp_refresh_token WHERE token_hash = ?",
    )
    .bind(hash_token(token))
    .fetch_optional(db)
    .await
}

/// 아직 폐기되지 않은 경우에만 폐기. 동시에 두 요청이 같은 토큰을 쓰면 하나만 성공한다.
pub async fn revoke_one(db: &MySqlPool, token: &str, now: DateTime<Utc>) -> sqlx::Result<bool> {
    let r =
        sqlx::query("UPDATE bp_refresh_token SET revoked_at = ? WHERE token_hash = ? AND revoked_at IS NULL")
            .bind(now.naive_utc())
            .bind(hash_token(token))
            .execute(db)
            .await?;
    Ok(r.rows_affected() == 1)
}

pub async fn revoke_family(db: &MySqlPool, family_id: &str, now: DateTime<Utc>) -> sqlx::Result<()> {
    sqlx::query("UPDATE bp_refresh_token SET revoked_at = ? WHERE family_id = ? AND revoked_at IS NULL")
        .bind(now.naive_utc())
        .bind(family_id)
        .execute(db)
        .await?;
    Ok(())
}

/// 만료 1일 지난 토큰 정리 (로그인 시 가끔 호출)
pub async fn purge_expired(db: &MySqlPool, now: DateTime<Utc>) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM bp_refresh_token WHERE expires_at < ?")
        .bind((now - chrono::Duration::days(1)).naive_utc())
        .execute(db)
        .await?;
    Ok(())
}
