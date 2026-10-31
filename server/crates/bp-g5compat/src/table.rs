//! g5 테이블 이름 헬퍼. 동적 테이블(`g5_write_{bo_table}`)은 반드시 여기서만 조립한다.

use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TableError {
    #[error("잘못된 게시판 아이디: {0}")]
    InvalidBoTable(String),
    #[error("잘못된 테이블 접두사: {0}")]
    InvalidPrefix(String),
}

fn ident_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // g5 bo_table 은 varchar(20), 영문·숫자·밑줄만 허용 (adm/board_form_update.php)
    RE.get_or_init(|| Regex::new(r"^[A-Za-z0-9_]{1,20}$").unwrap_or_else(|_| unreachable!()))
}

/// 테이블 접두사 (`G5_TABLE_PREFIX`, 기본 `g5_`).
#[derive(Debug, Clone)]
pub struct Prefix(String);

impl Prefix {
    pub fn new(prefix: &str) -> Result<Self, TableError> {
        let ok = !prefix.is_empty()
            && prefix.len() <= 20
            && prefix.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
        if ok {
            Ok(Self(prefix.to_owned()))
        } else {
            Err(TableError::InvalidPrefix(prefix.to_owned()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 고정 테이블 이름 (`member` → `` `g5_member` ``). 백틱 포함.
    pub fn table(&self, name: &str) -> String {
        format!("`{}{}`", self.0, name)
    }

    /// 게시판 글 테이블 (`free` → `` `g5_write_free` ``).
    /// 호출 측은 `bo_table` 이 `g5_board` 에 실제로 존재하는지도 확인해야 한다.
    pub fn write_table(&self, bo_table: &str) -> Result<String, TableError> {
        if ident_re().is_match(bo_table) {
            Ok(format!("`{}write_{}`", self.0, bo_table))
        } else {
            Err(TableError::InvalidBoTable(bo_table.to_owned()))
        }
    }
}

impl Default for Prefix {
    fn default() -> Self {
        Self("g5_".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_names() {
        let p = Prefix::default();
        assert_eq!(p.table("member"), "`g5_member`");
        assert_eq!(p.write_table("free"), Ok("`g5_write_free`".to_owned()));
    }

    #[test]
    fn rejects_injection() {
        let p = Prefix::default();
        for bad in ["", "free`; DROP TABLE x", "a b", "한글", "abcdefghijklmnopqrstu"] {
            assert!(p.write_table(bad).is_err(), "{bad}");
        }
        assert!(Prefix::new("g5`").is_err());
        assert!(Prefix::new("").is_err());
    }
}
