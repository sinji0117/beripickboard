//! g5 테이블 접근 (읽기/쓰기는 PHP 그누보드와 같은 규칙으로).

pub mod config;
pub mod member;
pub mod point;

use chrono::{DateTime, FixedOffset, Utc};

/// g5 기준 현재 시각 (`G5_TIME_YMDHIS` 등).
#[derive(Debug, Clone, Copy)]
pub struct Now(pub DateTime<FixedOffset>);

impl Now {
    pub fn new(tz: FixedOffset) -> Self {
        Self(Utc::now().with_timezone(&tz))
    }
    /// `Y-m-d H:i:s`
    pub fn ymdhis(&self) -> String {
        self.0.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    /// `Y-m-d`
    pub fn ymd(&self) -> String {
        self.0.format("%Y-%m-%d").to_string()
    }
    /// `Ymd`
    pub fn ymd_compact(&self) -> String {
        self.0.format("%Y%m%d").to_string()
    }
}
