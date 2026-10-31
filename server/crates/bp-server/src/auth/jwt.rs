use super::ACCESS_TTL_SECS;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// mb_id
    pub sub: String,
    /// mb_level
    pub lvl: i64,
    /// "super" 등 관리자 구분 (없으면 생략)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adm: Option<String>,
    pub iat: i64,
    pub exp: i64,
}

pub fn issue(
    secret: &[u8],
    mb_id: &str,
    level: i64,
    admin: Option<&str>,
    now: i64,
) -> anyhow::Result<String> {
    let claims = Claims {
        sub: mb_id.to_owned(),
        lvl: level,
        adm: admin.map(str::to_owned),
        iat: now,
        exp: now + ACCESS_TTL_SECS,
    };
    Ok(encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )?)
}

pub fn verify(secret: &[u8], token: &str) -> Option<Claims> {
    let mut v = Validation::new(Algorithm::HS256);
    v.leeway = 30;
    decode::<Claims>(token, &DecodingKey::from_secret(secret), &v)
        .ok()
        .map(|d| d.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_tamper() {
        let key = b"0123456789abcdef0123456789abcdef";
        let now = chrono::Utc::now().timestamp();
        let t = issue(key, "admin", 10, Some("super"), now).unwrap_or_default();
        let c = verify(key, &t);
        assert_eq!(c.as_ref().map(|c| c.sub.as_str()), Some("admin"));
        assert!(verify(b"another-key-another-key-another!!", &t).is_none());
        let expired = issue(key, "a", 1, None, now - 3600).unwrap_or_default();
        assert!(verify(key, &expired).is_none());
    }
}
