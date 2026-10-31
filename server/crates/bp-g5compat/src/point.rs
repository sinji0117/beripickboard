//! 포인트 규칙 (lib/common.lib.php `insert_point`).

use chrono::{Days, NaiveDate};

/// 무기한 포인트의 만료일.
pub const NO_EXPIRE: &str = "9999-12-31";

/// 지급 포인트의 `po_expire_date` 를 계산한다.
/// - `cf_point_term <= 0` → `9999-12-31`
/// - 양수 지급 → 오늘 + (기간 − 1)일 (`expire_days` 지정 시 그것을 우선)
/// - 음수(사용) → 오늘
pub fn expire_date(today: NaiveDate, point: i32, cf_point_term: i32, expire_days: i32) -> String {
    if point < 0 {
        return today.format("%Y-%m-%d").to_string();
    }
    if cf_point_term <= 0 {
        return NO_EXPIRE.to_owned();
    }
    let days = if expire_days > 0 {
        expire_days
    } else {
        cf_point_term
    };
    let add = u64::try_from(days - 1).unwrap_or(0);
    today
        .checked_add_days(Days::new(add))
        .map_or_else(|| NO_EXPIRE.to_owned(), |d| d.format("%Y-%m-%d").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap_or_default()
    }

    #[test]
    fn expire_rules() {
        let t = d("2026-10-31");
        assert_eq!(expire_date(t, 100, 0, 0), "9999-12-31");
        assert_eq!(expire_date(t, 100, 1, 0), "2026-10-31");
        assert_eq!(expire_date(t, 100, 30, 0), "2026-11-29");
        assert_eq!(expire_date(t, 100, 30, 7), "2026-11-06");
        assert_eq!(expire_date(t, -50, 30, 0), "2026-10-31");
    }
}
