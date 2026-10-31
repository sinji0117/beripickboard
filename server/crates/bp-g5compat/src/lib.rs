//! 그누보드5 · 영카트5 호환 계층.
//!
//! g5의 테이블 구조와 값의 의미를 바꾸지 않고 Rust에서 그대로 다루기 위한 헬퍼 모음.
//! 기준: 그누보드 5.6.44 / 영카트 5.4.5.5.1 (doc/03_architecture.md)

pub mod datetime;
pub mod member;
pub mod password;
pub mod point;
pub mod table;

/// 그누보드 5.6.44 `install/gnuboard5.sql` 의 테이블 (접두사 제외).
pub const G5_CORE_TABLES: &[&str] = &[
    "auth",
    "board",
    "board_file",
    "board_good",
    "board_new",
    "config",
    "cert_history",
    "member_cert_history",
    "group",
    "group_member",
    "login",
    "mail",
    "member",
    "member_auto_login",
    "memo",
    "point",
    "poll",
    "poll_etc",
    "popular",
    "scrap",
    "visit",
    "visit_sum",
    "uniqid",
    "autosave",
    "qa_config",
    "qa_content",
    "content",
    "faq",
    "faq_master",
    "member_social_profiles",
    "new_win",
    "menu",
    "migrations",
    "abuse_rate",
];

/// 영카트 5.4.5.5.1 `install/gnuboard5shop.sql` 의 테이블 (접두사 제외).
pub const G5_SHOP_TABLES: &[&str] = &[
    "shop_banner",
    "shop_cart",
    "shop_category",
    "shop_coupon",
    "shop_coupon_log",
    "shop_coupon_zone",
    "shop_default",
    "shop_event",
    "shop_event_item",
    "shop_item",
    "shop_item_option",
    "shop_item_use",
    "shop_item_qa",
    "shop_item_relation",
    "shop_order",
    "shop_order_address",
    "shop_order_data",
    "shop_order_delete",
    "shop_personalpay",
    "shop_sendcost",
    "shop_wish",
    "shop_item_stocksms",
    "shop_order_post_log",
    "shop_inicis_log",
    "shop_order_cancel_log",
    "shop_inicis_pay",
    "shop_inicis_pay_event",
    "shop_kcp_noti",
    "shop_order_access",
    "shop_lg_noti",
    "shop_pg_approval_log",
];
