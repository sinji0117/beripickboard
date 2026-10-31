//! 회원 상태 판정 (bbs/login_check.php 와 동일한 규칙).

/// `mb_intercept_date`, `mb_leave_date` 는 `varchar(8)` 의 `Ymd` 문자열.
/// 값이 있고 오늘(`today_ymd`, 예: `20261031`) 이하이면 적용된 것으로 본다.
pub fn ymd_reached(value: &str, today_ymd: &str) -> bool {
    let v = value.trim();
    !v.is_empty() && v <= today_ymd
}

/// `mb_email_certify` (datetime) 에 1~9 숫자가 하나라도 있으면 인증된 것.
/// (`preg_match("/[1-9]/", $mb['mb_email_certify'])`)
pub fn email_certified(mb_email_certify: &str) -> bool {
    mb_email_certify.bytes().any(|b| (b'1'..=b'9').contains(&b))
}

/// g5 관리자 등급. 게시판/그룹 관리자는 해당 게시판·그룹 문맥에서만 판정한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminKind {
    Super,
    Group,
    Board,
}

impl AdminKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Super => "super",
            Self::Group => "group",
            Self::Board => "board",
        }
    }
}

/// `is_admin()` 중 최고관리자 판정 (`cf_admin == mb_id`).
pub fn is_super_admin(cf_admin: &str, mb_id: &str) -> bool {
    !mb_id.is_empty() && cf_admin == mb_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intercept_and_leave_dates() {
        assert!(!ymd_reached("", "20261031"));
        assert!(ymd_reached("20261031", "20261031"));
        assert!(ymd_reached("20250101", "20261031"));
        assert!(!ymd_reached("20261101", "20261031")); // 예약된 차단은 아직 아님
    }

    #[test]
    fn email_certify() {
        assert!(!email_certified("0000-00-00 00:00:00"));
        assert!(email_certified("2026-10-31 12:00:00"));
    }

    #[test]
    fn super_admin() {
        assert!(is_super_admin("admin", "admin"));
        assert!(!is_super_admin("admin", "Admin"));
        assert!(!is_super_admin("", ""));
    }
}
