//! g5의 `0000-00-00 00:00:00` / `0000-00-00` 기본값 처리.

use chrono::{NaiveDate, NaiveDateTime};

pub const ZERO_DATETIME: &str = "0000-00-00 00:00:00";
pub const ZERO_DATE: &str = "0000-00-00";

/// g5 datetime 문자열 → `None`(0일자/빈값) 또는 날짜.
/// DB에서는 `CAST(col AS CHAR)` 로 읽어 이 함수로 변환한다(드라이버의 0일자 오류 회피).
pub fn parse_datetime(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    if s.is_empty() || s.starts_with("0000-00-00") {
        return None;
    }
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
}

pub fn parse_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.is_empty() || s == ZERO_DATE {
        return None;
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// 저장 시: `None` → g5 기본값 문자열.
pub fn format_datetime(v: Option<NaiveDateTime>) -> String {
    v.map_or_else(
        || ZERO_DATETIME.to_owned(),
        |d| d.format("%Y-%m-%d %H:%M:%S").to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_dates_are_none() {
        assert_eq!(parse_datetime(ZERO_DATETIME), None);
        assert_eq!(parse_datetime(""), None);
        assert_eq!(parse_date(ZERO_DATE), None);
        assert_eq!(format_datetime(None), ZERO_DATETIME);
    }

    #[test]
    fn roundtrip() {
        let d = parse_datetime("2026-10-31 20:58:00");
        assert!(d.is_some());
        assert_eq!(format_datetime(d), "2026-10-31 20:58:00");
        assert!(parse_date("2026-02-30").is_none());
    }
}
