//! 사용법:
//!   bp-migrate check   기존 g5/영카트/이윰보드 DB 점검 (읽기 전용)
//!   bp-migrate up      bp_* 테이블 마이그레이션 적용 (g5 테이블은 건드리지 않음)

use anyhow::{Context, Result, bail};
use bp_g5compat::{G5_CORE_TABLES, G5_SHOP_TABLES, table::Prefix};
use sqlx::{MySqlPool, Row};
use std::collections::BTreeMap;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let cmd = std::env::args().nth(1).unwrap_or_default();
    let url = std::env::var("DATABASE_URL").context("DATABASE_URL 이 필요합니다")?;
    let prefix = Prefix::new(&std::env::var("G5_TABLE_PREFIX").unwrap_or_else(|_| "g5_".into()))?;
    let pool = MySqlPool::connect(&url).await.context("DB 연결 실패")?;

    match cmd.as_str() {
        "check" => check(&pool, &prefix).await,
        "up" => {
            sqlx::migrate!("../../migrations").run(&pool).await?;
            println!("bp_* 마이그레이션 적용 완료");
            Ok(())
        }
        _ => bail!("사용법: bp-migrate <check|up>"),
    }
}

async fn check(pool: &MySqlPool, prefix: &Prefix) -> Result<()> {
    let p = prefix.as_str();
    let rows = sqlx::query(
        "SELECT TABLE_NAME AS name, IFNULL(ENGINE,'') AS engine, IFNULL(TABLE_COLLATION,'') AS coll \
         FROM information_schema.TABLES WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME LIKE CONCAT(?, '%')",
    )
    .bind(p)
    .fetch_all(pool)
    .await?;

    let mut tables: BTreeMap<String, (String, String)> = BTreeMap::new();
    for r in &rows {
        let name: String = r.try_get("name")?;
        tables.insert(
            name.trim_start_matches(p).to_owned(),
            (r.try_get("engine")?, r.try_get("coll")?),
        );
    }

    let missing = |list: &[&str]| -> Vec<String> {
        list.iter()
            .filter(|t| !tables.contains_key(**t))
            .map(|t| format!("{p}{t}"))
            .collect()
    };
    let core_missing = missing(G5_CORE_TABLES);
    let shop_missing = missing(G5_SHOP_TABLES);
    let boards: Vec<&String> = tables.keys().filter(|k| k.starts_with("write_")).collect();
    let eyoom: Vec<&String> = tables.keys().filter(|k| k.starts_with("eyoom")).collect();
    let myisam = tables
        .values()
        .filter(|(e, _)| e.eq_ignore_ascii_case("MyISAM"))
        .count();
    let not_mb4 = tables.values().filter(|(_, c)| !c.starts_with("utf8mb4")).count();

    println!("== beripickboard DB 점검 (접두사 {p}) ==");
    println!("테이블 수: {}", tables.len());
    println!("게시판 글 테이블: {}개", boards.len());
    println!("이윰보드 테이블: {}개 {:?}", eyoom.len(), eyoom);
    println!(
        "그누보드 기본 테이블 누락: {} {:?}",
        core_missing.len(),
        core_missing
    );
    println!("영카트 테이블 누락: {} {:?}", shop_missing.len(), shop_missing);
    println!("MyISAM 테이블: {myisam}개 (InnoDB 변환 대상)");
    println!("utf8mb4 아닌 테이블: {not_mb4}개");

    if tables.contains_key("migrations") {
        let last: Option<String> = sqlx::query_scalar(&format!(
            "SELECT migration_id FROM {} WHERE status='applied' ORDER BY migration_id DESC LIMIT 1",
            prefix.table("migrations")
        ))
        .fetch_optional(pool)
        .await
        .unwrap_or(None);
        println!(
            "g5 마지막 적용 마이그레이션: {}",
            last.as_deref().unwrap_or("(없음)")
        );
    } else {
        println!("g5_migrations 없음 → 5.6.x 이전 버전. 그누보드 최신 패치 적용 후 다시 점검하세요.");
    }
    Ok(())
}
