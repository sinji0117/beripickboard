//! g5 비밀번호 해시 호환.
//!
//! | 형식 | 예 | 출처 |
//! |---|---|---|
//! | PBKDF2 | `sha256:12000:{salt}:{hash}` | `lib/pbkdf2.compat.php` `create_hash()` (기본) |
//! | PBKDF2 (PHP 호환 모드) | `sha1:2400:{salt}:{hash}` | `create_hash($pw, true)` |
//! | MySQL `PASSWORD()` | `*` + 40 hex | `G5_STRING_ENCRYPT_FUNCTION = 'sql_password'` 시절 |
//! | MySQL `OLD_PASSWORD()` | 16 hex | 아주 오래된 사이트. 검증 불가 → 비밀번호 재설정 필요 |
//!
//! 주의: g5는 salt를 **base64 문자열 그대로** PBKDF2 salt로 쓴다(디코드하지 않음).

use base64::{Engine, engine::general_purpose::STANDARD as B64};
use hmac::Hmac;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use subtle::ConstantTimeEq;

/// g5 `create_hash()` 기본값.
pub const PBKDF2_ITERATIONS: u32 = 12000;
pub const PBKDF2_SALT_BYTES: usize = 24;
pub const PBKDF2_HASH_BYTES: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashKind {
    Pbkdf2Sha256,
    Pbkdf2Sha1,
    MysqlPassword,
    MysqlOldPassword,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Verify {
    /// 일치
    Ok,
    /// 일치하지만 현재 기준(sha256/12000)보다 약함 → 로그인 직후 재해시 권장 (g5 `needs_upgrade`)
    OkNeedsUpgrade,
    /// 불일치
    Mismatch,
    /// 검증할 수 없는 형식 (OLD_PASSWORD 등) → 비밀번호 재설정 유도
    Unsupported,
}

pub fn detect(hash: &str) -> HashKind {
    let parts: Vec<&str> = hash.split(':').collect();
    if parts.len() >= 4 {
        return match parts[0] {
            "sha256" => HashKind::Pbkdf2Sha256,
            "sha1" => HashKind::Pbkdf2Sha1,
            _ => HashKind::Unknown,
        };
    }
    let is_hex = |s: &str| s.bytes().all(|b| b.is_ascii_hexdigit());
    if hash.len() == 41 && hash.starts_with('*') && is_hex(&hash[1..]) {
        HashKind::MysqlPassword
    } else if hash.len() == 16 && is_hex(hash) {
        HashKind::MysqlOldPassword
    } else {
        HashKind::Unknown
    }
}

/// 저장된 g5 해시(`mb_password`, `wr_password` 등)와 평문 비밀번호를 비교한다.
pub fn verify(password: &str, hash: &str) -> Verify {
    match detect(hash) {
        HashKind::Pbkdf2Sha256 | HashKind::Pbkdf2Sha1 => verify_pbkdf2(password, hash),
        HashKind::MysqlPassword => {
            if ct_eq(
                mysql_password(password).as_bytes(),
                hash.to_ascii_uppercase().as_bytes(),
            ) {
                Verify::OkNeedsUpgrade
            } else {
                Verify::Mismatch
            }
        }
        HashKind::MysqlOldPassword | HashKind::Unknown => Verify::Unsupported,
    }
}

fn verify_pbkdf2(password: &str, hash: &str) -> Verify {
    let p: Vec<&str> = hash.split(':').collect();
    let (algo, salt, expected_b64) = (p[0], p[2], p[3]);
    let Ok(iterations) = p[1].parse::<u32>() else {
        return Verify::Unsupported;
    };
    let Ok(expected) = B64.decode(expected_b64) else {
        return Verify::Unsupported;
    };
    if iterations == 0 || expected.is_empty() {
        return Verify::Unsupported;
    }
    let mut out = vec![0u8; expected.len()];
    let derived = match algo {
        "sha256" => {
            pbkdf2::pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt.as_bytes(), iterations, &mut out)
        }
        "sha1" => pbkdf2::pbkdf2::<Hmac<Sha1>>(password.as_bytes(), salt.as_bytes(), iterations, &mut out),
        _ => return Verify::Unsupported,
    };
    if derived.is_err() {
        return Verify::Unsupported;
    }
    if !ct_eq(&out, &expected) {
        Verify::Mismatch
    } else if algo == "sha256" && iterations >= PBKDF2_ITERATIONS {
        Verify::Ok
    } else {
        Verify::OkNeedsUpgrade
    }
}

/// g5 `create_hash()` 와 같은 형식(`sha256:12000:salt:hash`)으로 해시를 만든다.
/// 공존 기간 동안 PHP 쪽에서도 로그인되도록 이 형식으로 저장한다.
/// `salt_raw` 는 24바이트 난수(호출 측에서 OS RNG로 생성).
pub fn create_hash(password: &str, salt_raw: &[u8; PBKDF2_SALT_BYTES]) -> String {
    let salt = B64.encode(salt_raw);
    let mut out = [0u8; PBKDF2_HASH_BYTES];
    // 출력 길이가 고정이고 0이 아니므로 실패하지 않는다.
    let _ = pbkdf2::pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt.as_bytes(), PBKDF2_ITERATIONS, &mut out);
    format!("sha256:{PBKDF2_ITERATIONS}:{salt}:{}", B64.encode(out))
}

/// MySQL `PASSWORD()` = `'*' + UPPER(HEX(SHA1(SHA1(pw))))`
pub fn mysql_password(password: &str) -> String {
    let inner = Sha1::digest(password.as_bytes());
    let outer = Sha1::digest(inner);
    format!("*{}", hex::encode_upper(outer))
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && bool::from(a.ct_eq(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    // 그누보드 5.6.44 lib/pbkdf2.compat.php 의 create_hash() 를 PHP로 실행해 만든 실제 해시.
    const G5_HASHES: &[(&str, &str)] = &[
        (
            "test1234",
            "sha256:12000:z2WJyEE9ecDTA40/4W8y4l9mKw56lOC6:JPhpyZBWkRauawQzF2MiomGvwB4LFjFV",
        ),
        (
            "비밀번호!@#",
            "sha256:12000:cjQUebOEc2XAVmNGLXmrDcs2V3Jgmeqe:FBBaOpokwg6DiOe5Dh/MTlqtDeU0RobW",
        ),
        (
            "p",
            "sha256:12000:GpUlX62y1P7/aveoh55fwx/KOMLmDiKW:iKpYVoY9NG9SXpPNJmlP6cTSTyDYXs9o",
        ),
    ];

    #[test]
    fn verifies_real_g5_pbkdf2_hashes() {
        for (pw, hash) in G5_HASHES {
            assert_eq!(detect(hash), HashKind::Pbkdf2Sha256);
            assert_eq!(verify(pw, hash), Verify::Ok, "{pw}");
            assert_eq!(verify("wrong", hash), Verify::Mismatch);
        }
    }

    #[test]
    fn verifies_g5_php_compat_sha1_hash_and_flags_upgrade() {
        let h = "sha1:2400:WesRTKeZ26T1450FV3MQVlwavVPyCorb:MP1S+YYFFuTEPcjPi39XBrC5vNdUWY/l";
        assert_eq!(verify("test1234", h), Verify::OkNeedsUpgrade);
        assert_eq!(verify("test123", h), Verify::Mismatch);
    }

    #[test]
    fn verifies_mysql_password() {
        // SELECT PASSWORD('test') 의 잘 알려진 값
        let h = "*94BDCEBE19083CE2A1F959FD02F964C7AF4CFC29";
        assert_eq!(mysql_password("test"), h);
        assert_eq!(detect(h), HashKind::MysqlPassword);
        assert_eq!(verify("test", h), Verify::OkNeedsUpgrade);
        assert_eq!(verify("test", &h.to_lowercase()), Verify::OkNeedsUpgrade);
        assert_eq!(verify("tset", h), Verify::Mismatch);
    }

    #[test]
    fn old_password_and_garbage_are_unsupported() {
        assert_eq!(verify("x", "565491d704013245"), Verify::Unsupported);
        assert_eq!(verify("x", ""), Verify::Unsupported);
        assert_eq!(verify("x", "md5:1:a:b"), Verify::Unsupported);
        assert_eq!(verify("x", "sha256:0:a:b"), Verify::Unsupported);
    }

    #[test]
    fn create_hash_roundtrip_matches_g5_format() {
        let h = create_hash("hello", &[7u8; PBKDF2_SALT_BYTES]);
        let parts: Vec<&str> = h.split(':').collect();
        assert_eq!(parts[0], "sha256");
        assert_eq!(parts[1], "12000");
        assert_eq!(parts[2].len(), 32); // base64(24 bytes)
        assert_eq!(parts[3].len(), 32);
        assert_eq!(verify("hello", &h), Verify::Ok);
    }
}
