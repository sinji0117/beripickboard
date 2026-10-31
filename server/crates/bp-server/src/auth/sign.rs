//! 짧은 수명의 서명 토큰 (다운로드 링크, 비밀글 열람 허가). JWT_SECRET 으로 HMAC-SHA256 서명.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as B64};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type H = Hmac<Sha256>;

fn mac(secret: &[u8], payload: &str) -> Option<Vec<u8>> {
    let mut m = H::new_from_slice(secret).ok()?;
    m.update(payload.as_bytes());
    Some(m.finalize().into_bytes().to_vec())
}

/// `payload` 는 `|` 로 구분된 필드. 마지막에 만료 시각(unix)을 붙여 서명한다.
pub fn sign(secret: &[u8], fields: &[&str], exp: i64) -> String {
    let payload = format!("{}|{exp}", fields.join("|"));
    let sig = mac(secret, &payload).unwrap_or_default();
    format!("{}.{}", B64.encode(&payload), B64.encode(sig))
}

/// 서명·만료를 확인하고 필드를 돌려준다 (만료 필드 제외).
pub fn verify(secret: &[u8], token: &str, now: i64) -> Option<Vec<String>> {
    let (p, s) = token.split_once('.')?;
    let payload = String::from_utf8(B64.decode(p).ok()?).ok()?;
    let sig = B64.decode(s).ok()?;
    let mut m = H::new_from_slice(secret).ok()?;
    m.update(payload.as_bytes());
    m.verify_slice(&sig).ok()?;
    let mut fields: Vec<String> = payload.split('|').map(str::to_owned).collect();
    let exp: i64 = fields.pop()?.parse().ok()?;
    (exp >= now).then_some(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_verify() {
        let k = b"0123456789abcdef0123456789abcdef";
        let t = sign(k, &["dl", "free", "3", "admin"], 100);
        assert_eq!(
            verify(k, &t, 50),
            Some(vec!["dl".into(), "free".into(), "3".into(), "admin".into()])
        );
        assert_eq!(verify(k, &t, 101), None);
        assert_eq!(verify(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", &t, 50), None);
        let tampered = t.replacen('A', "B", 1);
        assert!(tampered == t || verify(k, &tampered, 50).is_none());
    }
}
