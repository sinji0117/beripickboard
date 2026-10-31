//! 게시판 목록 규칙 (bbs/list.php, lib/common.lib.php `get_sql_search`, `get_board_sort_fields`).
//! SQL 조각은 값이 모두 바인드 파라미터로 분리되어 있어 인젝션이 불가능하다.

/// 바인드 값
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bind {
    Str(String),
    Int(i64),
}

/// WHERE 조각 + 바인드 값
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SqlPart {
    pub sql: String,
    pub binds: Vec<Bind>,
}

/// 검색 허용 필드. 그 외 필드명은 g5 처럼 `wr_subject` 로 대체.
const TEXT_FIELDS: &[&str] = &[
    "wr_subject",
    "wr_content",
    "wr_1",
    "wr_2",
    "wr_3",
    "wr_4",
    "wr_5",
    "wr_6",
    "wr_7",
    "wr_8",
    "wr_9",
    "wr_10",
];

/// `get_sql_search($sca, $sfl, $stx, $sop)` 이식.
/// - `sfl` 예: `wr_subject`, `wr_subject||wr_content`, `mb_id,1`(원글만), `wr_name,0`(댓글만)
/// - 검색어는 공백으로 나눠 `sop`(and/or) 로 결합
/// - 반환 `None` = 검색 아님(g5 의 `'0'`)
pub fn search_sql(sca: &str, sfl: &str, stx: &str, sop: &str) -> Option<SqlPart> {
    let mut part = SqlPart::default();
    let sca = sca.trim();
    if !sca.is_empty() {
        part.sql.push_str("ca_name = ?");
        part.binds.push(Bind::Str(sca.to_owned()));
    }
    let stx = strip_tags(stx).trim().to_owned();
    if stx.is_empty() {
        return if sca.is_empty() { None } else { Some(part) };
    }
    if !part.sql.is_empty() {
        part.sql.push_str(" AND ");
    }
    let sop = if sop.eq_ignore_ascii_case("or") {
        " OR "
    } else {
        " AND "
    };
    let (fields_part, not_comment) = match sfl.trim().split_once(',') {
        Some((f, n)) => (f, Some(n.trim())),
        None => (sfl.trim(), None),
    };
    let fields: Vec<String> = fields_part
        .split("||")
        .map(|f| f.trim().to_ascii_lowercase())
        .collect();

    part.sql.push('(');
    let mut first_word = true;
    for word in stx.split(' ').map(str::trim).filter(|w| !w.is_empty()) {
        if !first_word {
            part.sql.push_str(sop);
        }
        first_word = false;
        part.sql.push('(');
        for (k, f) in fields.iter().enumerate() {
            if k > 0 {
                part.sql.push_str(" OR ");
            }
            match f.as_str() {
                "mb_id" | "wr_name" => {
                    part.sql.push_str(&format!("{f} = ?"));
                    part.binds.push(Bind::Str(word.to_owned()));
                }
                "wr_hit" | "wr_good" | "wr_nogood" => {
                    part.sql.push_str(&format!("{f} >= ?"));
                    part.binds.push(Bind::Int(word.parse().unwrap_or(0)));
                }
                "wr_num" => {
                    part.sql.push_str("wr_num = ?");
                    part.binds.push(Bind::Int(-word.parse::<i64>().unwrap_or(0)));
                }
                "wr_ip" | "wr_password" => part.sql.push_str("1=0"),
                other => {
                    let col = if TEXT_FIELDS.contains(&other) {
                        other
                    } else {
                        "wr_subject"
                    };
                    if word.bytes().any(|b| b.is_ascii_alphabetic()) {
                        part.sql.push_str(&format!("INSTR(LOWER({col}), LOWER(?))"));
                    } else {
                        part.sql.push_str(&format!("INSTR({col}, ?)"));
                    }
                    part.binds.push(Bind::Str(word.to_owned()));
                }
            }
        }
        part.sql.push(')');
    }
    part.sql.push(')');
    match not_comment {
        Some("1") => part.sql.push_str(" AND wr_is_comment = '0'"),
        Some("0") => part.sql.push_str(" AND wr_is_comment = '1'"),
        _ => {}
    }
    Some(part)
}

/// 인기검색어(g5_popular)에 넣을 단어 목록 (`insert_popular`: mb_id 검색은 제외)
pub fn popular_words(sfl: &str, stx: &str) -> Vec<String> {
    let fields = sfl.split(',').next().unwrap_or_default();
    if fields.split("||").any(|f| f.trim().eq_ignore_ascii_case("mb_id")) {
        return vec![];
    }
    strip_tags(stx)
        .split(' ')
        .map(str::trim)
        .filter(|w| !w.is_empty())
        .map(|w| w.chars().take(50).collect())
        .collect()
}

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// `get_board_sort_fields()` 기본 목록
pub const SORT_FIELDS: &[&str] = &[
    "wr_num, wr_reply",
    "wr_datetime asc",
    "wr_datetime desc",
    "wr_hit asc, wr_num, wr_reply",
    "wr_hit desc, wr_num, wr_reply",
    "wr_last asc",
    "wr_last desc",
    "wr_comment asc, wr_num, wr_reply",
    "wr_comment desc, wr_num, wr_reply",
    "wr_good asc, wr_num, wr_reply",
    "wr_good desc, wr_num, wr_reply",
    "wr_nogood asc, wr_num, wr_reply",
    "wr_nogood desc, wr_num, wr_reply",
    "wr_subject asc, wr_num, wr_reply",
    "wr_subject desc, wr_num, wr_reply",
    "wr_name asc, wr_num, wr_reply",
    "wr_name desc, wr_num, wr_reply",
    "ca_name asc, wr_num, wr_reply",
    "ca_name desc, wr_num, wr_reply",
];

/// list.php 의 정렬 결정. 반환값은 화이트리스트 문자열이므로 SQL 에 그대로 넣어도 안전.
pub fn order_by(sst: &str, sod: &str, bo_sort_field: &str) -> String {
    let sst = sst.trim();
    let sod = sod.trim().to_ascii_lowercase();
    if sst.is_empty() {
        return SORT_FIELDS
            .iter()
            .find(|f| **f == bo_sort_field)
            .unwrap_or(&SORT_FIELDS[0])
            .to_string();
    }
    if sod.is_empty() {
        // 키 형식: 공백 제거 (예: "wr_hitdesc,wr_num,wr_reply")
        if let Some(f) = SORT_FIELDS.iter().find(|f| f.replace(' ', "") == sst) {
            return (*f).to_owned();
        }
    }
    let col = match sst {
        "wr_datetime" | "wr_hit" | "wr_good" | "wr_nogood" => sst,
        _ => return SORT_FIELDS[0].to_owned(),
    };
    match sod.as_str() {
        "asc" | "desc" => format!("{col} {sod}"),
        _ => col.to_owned(),
    }
}

/// 목록 페이지 계산 결과 (공지 포함 규칙)
#[derive(Debug, PartialEq, Eq)]
pub struct Paging {
    /// 이 페이지에 보일 공지의 인덱스 시작 (`bo_notice` 순서 기준)
    pub notice_from: usize,
    pub from_record: i64,
    pub rows: i64,
    pub total_page: i64,
}

/// list.php 의 공지·페이지 계산.
/// - `notice_total`: 실제 존재하는 공지 글 수
/// - `notice_shown`: 이 페이지에 표시되는 공지 수
pub fn paging(page: i64, page_rows: i64, total_count: i64, notice_total: i64, notice_shown: i64) -> Paging {
    let page = page.max(1);
    let total_page = if page_rows > 0 {
        (total_count + page_rows - 1) / page_rows
    } else {
        0
    };
    let mut from_record = (page - 1) * page_rows;
    let mut rows = page_rows;
    if notice_total > 0 {
        from_record = (from_record - notice_total).max(0);
        if notice_shown > 0 {
            rows -= notice_shown;
        }
        if rows < 0 {
            rows = page_rows;
        }
    }
    Paging {
        notice_from: usize::try_from((page - 1) * page_rows).unwrap_or(0),
        from_record,
        rows,
        total_page,
    }
}

/// 목록 번호: `total - (page-1)*page_rows - notice_count - k`
pub fn list_num(total_count: i64, page: i64, page_rows: i64, notice_shown: i64, k: i64) -> i64 {
    total_count - (page.max(1) - 1) * page_rows - notice_shown - k
}

/// `G5_IP_DISPLAY` = `\1.♡.\3.\4`
pub fn mask_ip(ip: &str) -> String {
    let p: Vec<&str> = ip.split('.').collect();
    if p.len() == 4
        && p.iter()
            .all(|x| !x.is_empty() && x.bytes().all(|b| b.is_ascii_digit()))
    {
        format!("{}.♡.{}.{}", p[0], p[2], p[3])
    } else {
        ip.to_owned()
    }
}

/// 내용 표시 방식 (`wr_option` 의 html1/html2, conv_content)
pub fn content_mode(wr_option: &str) -> &'static str {
    if wr_option.contains("html1") {
        "html"
    } else if wr_option.contains("html2") {
        "html_br"
    } else {
        "text"
    }
}

pub fn has_option(wr_option: &str, opt: &str) -> bool {
    wr_option.split(',').any(|o| o.trim() == opt)
}

/// bo_notice ("12,5,") → 정수 id 목록 (순서 유지)
pub fn notice_ids(bo_notice: &str) -> Vec<i64> {
    bo_notice
        .split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .filter(|n| *n > 0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_basic() {
        let p = search_sql("", "wr_subject||wr_content", "hello 세계", "and").unwrap_or_default();
        assert_eq!(
            p.sql,
            "((INSTR(LOWER(wr_subject), LOWER(?)) OR INSTR(LOWER(wr_content), LOWER(?))) AND (INSTR(wr_subject, ?) OR INSTR(wr_content, ?)))"
        );
        assert_eq!(p.binds.len(), 4);
    }

    #[test]
    fn search_category_only_and_none() {
        let p = search_sql("공지", "wr_subject", "", "and");
        assert_eq!(p.map(|p| p.sql), Some("ca_name = ?".to_owned()));
        assert!(search_sql("", "wr_subject", "  ", "and").is_none());
    }

    #[test]
    fn search_member_and_comment_flags() {
        let p = search_sql("", "mb_id,1", "admin", "or").unwrap_or_default();
        assert_eq!(p.sql, "((mb_id = ?)) AND wr_is_comment = '0'");
        let p = search_sql("", "wr_name,0", "홍길동", "and").unwrap_or_default();
        assert!(p.sql.ends_with("wr_is_comment = '1'"));
    }

    #[test]
    fn search_rejects_injection_fields() {
        let p = search_sql("", "wr_password", "x", "and").unwrap_or_default();
        assert_eq!(p.sql, "((1=0))");
        let p = search_sql("", "(select 1)", "x", "and").unwrap_or_default();
        assert_eq!(p.sql, "((INSTR(LOWER(wr_subject), LOWER(?))))");
        assert_eq!(popular_words("mb_id", "admin"), Vec::<String>::new());
        assert_eq!(popular_words("wr_subject", "<b>a</b> b"), vec!["a", "b"]);
    }

    #[test]
    fn sorting() {
        assert_eq!(order_by("", "", ""), "wr_num, wr_reply");
        assert_eq!(order_by("", "", "wr_datetime desc"), "wr_datetime desc");
        assert_eq!(order_by("", "", "1; drop table"), "wr_num, wr_reply");
        assert_eq!(
            order_by("wr_hitdesc,wr_num,wr_reply", "", ""),
            "wr_hit desc, wr_num, wr_reply"
        );
        assert_eq!(order_by("wr_hit", "DESC", ""), "wr_hit desc");
        assert_eq!(order_by("wr_password", "asc", ""), "wr_num, wr_reply");
    }

    #[test]
    fn paging_with_notices() {
        // 15개씩, 전체 40, 공지 2개 → 1페이지: 일반글 0부터 13개
        assert_eq!(
            paging(1, 15, 40, 2, 2),
            Paging {
                notice_from: 0,
                from_record: 0,
                rows: 13,
                total_page: 3
            }
        );
        // 2페이지: 공지 미표시, from = 15-2 = 13
        assert_eq!(
            paging(2, 15, 40, 2, 0),
            Paging {
                notice_from: 15,
                from_record: 13,
                rows: 15,
                total_page: 3
            }
        );
        assert_eq!(list_num(40, 1, 15, 2, 0), 38);
        assert_eq!(list_num(40, 2, 15, 0, 0), 25);
    }

    #[test]
    fn misc() {
        assert_eq!(mask_ip("123.45.67.89"), "123.♡.67.89");
        assert_eq!(mask_ip("::1"), "::1");
        assert_eq!(content_mode("html1"), "html");
        assert_eq!(content_mode("html2,secret"), "html_br");
        assert_eq!(content_mode(""), "text");
        assert!(has_option("html1,secret", "secret"));
        assert_eq!(notice_ids("12, 5,,x,0"), vec![12, 5]);
    }
}
