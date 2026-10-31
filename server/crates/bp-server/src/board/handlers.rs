use super::viewer::{Admin, Viewer, check_list, check_read};
use crate::{
    auth::{
        extract::{ClientMeta, OptionalMember},
        sign,
    },
    error::{AppError, AppResult},
    g5::{
        Now,
        board::{self as g5board, Board, Group},
        config::{self as g5config, G5Config},
        point::{self, PointInput},
        write::{self, FileRow, WriteRow},
    },
    state::AppState,
};
use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
};
use bp_g5compat::{
    board::{self as rules, SqlPart, content_mode, has_option, mask_ip},
    password::{self, Verify},
    table::Prefix,
};
use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::MySqlPool;

/// 다운로드·이미지 링크 유효시간
const FILE_TOKEN_SECS: i64 = 3600;
/// 비밀글 열람 허가 유효시간 (PHP 세션 수명 정도)
const SECRET_GRANT_SECS: i64 = 3 * 3600;
pub const SECRET_GRANT_HEADER: &str = "x-secret-grant";

struct Ctx {
    db: MySqlPool,
    p: Prefix,
    cfg: G5Config,
    board: Board,
    group: Group,
    viewer: Viewer,
    admin: Option<Admin>,
    table: String,
    now: Now,
}

async fn ctx(
    s: &AppState,
    bo_table: &str,
    claims: Option<&crate::auth::jwt::Claims>,
    ip: &str,
) -> AppResult<Ctx> {
    let db =
        s.0.db
            .clone()
            .ok_or_else(|| AppError::unavailable("DB_DISABLED", "DB가 설정되지 않았습니다."))?;
    let p = s.0.config.prefix.clone();
    let not_found = || AppError::not_found("BOARD_NOT_FOUND", "존재하지 않는 게시판입니다.");
    let table = p.write_table(bo_table).map_err(|_| not_found())?;
    let board = g5board::find(&db, &p, bo_table).await?.ok_or_else(not_found)?;
    let cfg = g5config::load(&db, &p).await?;
    let group = g5board::group(&db, &p, &board.gr_id).await?;
    let now = Now::new(s.0.config.tz);
    let viewer = Viewer::load(&db, &p, claims, ip, &now.ymd_compact()).await?;
    let admin = viewer.admin(&cfg, &group, &board);
    Ok(Ctx {
        db,
        p,
        cfg,
        board,
        group,
        viewer,
        admin,
        table,
        now,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BoardInfo<'a> {
    bo_table: &'a str,
    gr_id: &'a str,
    gr_subject: &'a str,
    subject: &'a str,
    skin: &'a str,
    use_category: bool,
    categories: Vec<&'a str>,
    use_good: bool,
    use_nogood: bool,
    use_secret: i64,
    gallery_cols: i64,
    page_rows: i64,
    subject_len: i64,
    can_write: bool,
    admin: Option<&'static str>,
}

fn board_info(c: &Ctx) -> BoardInfo<'_> {
    let b = &c.board;
    BoardInfo {
        bo_table: &b.bo_table,
        gr_id: &b.gr_id,
        gr_subject: &c.group.gr_subject,
        subject: &b.bo_subject,
        skin: &b.bo_skin,
        use_category: b.bo_use_category != 0,
        categories: b
            .bo_category_list
            .split('|')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect(),
        use_good: b.bo_use_good != 0,
        use_nogood: b.bo_use_nogood != 0,
        use_secret: b.bo_use_secret,
        gallery_cols: b.bo_gallery_cols.max(1),
        page_rows: b.bo_page_rows,
        subject_len: b.bo_subject_len,
        can_write: c.viewer.level >= b.bo_write_level,
        admin: c.admin.map(Admin::as_str),
    }
}

/// GET /api/v1/boards — 목록 권한이 있는 게시판
pub async fn list_boards(
    State(s): State<AppState>,
    OptionalMember(claims): OptionalMember,
    meta: ClientMeta,
) -> AppResult<Json<serde_json::Value>> {
    let db =
        s.0.db
            .as_ref()
            .ok_or_else(|| AppError::unavailable("DB_DISABLED", "DB가 설정되지 않았습니다."))?;
    let p = &s.0.config.prefix;
    let now = Now::new(s.0.config.tz);
    let v = Viewer::load(db, p, claims.as_ref(), &meta.ip, &now.ymd_compact()).await?;
    let items: Vec<_> = g5board::list_all(db, p)
        .await?
        .into_iter()
        .filter(|b| v.level >= b.bo_list_level)
        .map(|b| json!({ "boTable": b.bo_table, "grId": b.gr_id, "subject": b.bo_subject, "countWrite": b.bo_count_write, "order": b.bo_order }))
        .collect();
    Ok(Json(json!({ "data": items })))
}

#[derive(Deserialize, Default)]
pub struct ListQuery {
    page: Option<i64>,
    #[serde(default)]
    sca: String,
    #[serde(default)]
    sfl: String,
    #[serde(default)]
    stx: String,
    #[serde(default)]
    sop: String,
    spt: Option<i64>,
    #[serde(default)]
    sst: String,
    #[serde(default)]
    sod: String,
}

impl ListQuery {
    fn search(&self) -> Option<SqlPart> {
        let sfl = if self.sfl.is_empty() {
            "wr_subject||wr_content"
        } else {
            &self.sfl
        };
        rules::search_sql(&self.sca, sfl, &self.stx, &self.sop)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListItem {
    wr_id: i64,
    num: i64,
    is_notice: bool,
    is_secret: bool,
    depth: usize,
    ca_name: String,
    subject: String,
    content: Option<String>,
    content_mode: &'static str,
    name: String,
    mb_id: String,
    datetime: String,
    /// 마지막 활동(댓글 포함) 일시
    last: String,
    hit: i64,
    good: i64,
    nogood: i64,
    comment: i64,
    file: i64,
    has_link: bool,
    is_new: bool,
    is_hot: bool,
}

fn is_new(wr_datetime: &str, bo_new: i64, now: Now) -> bool {
    if bo_new <= 0 {
        return false;
    }
    let Ok(dt) = NaiveDateTime::parse_from_str(wr_datetime, "%Y-%m-%d %H:%M:%S") else {
        return false;
    };
    dt >= now.0.naive_local() - Duration::hours(bo_new)
}

fn list_item(c: &Ctx, w: WriteRow, num: i64, is_notice: bool) -> ListItem {
    let secret = has_option(&w.wr_option, "secret");
    let show_content = c.board.bo_use_list_content != 0 && !secret;
    ListItem {
        wr_id: w.wr_id,
        num,
        is_notice,
        is_secret: secret,
        depth: w.wr_reply.len(),
        ca_name: w.ca_name,
        subject: w.wr_subject,
        content: show_content.then_some(w.wr_content),
        content_mode: content_mode(&w.wr_option),
        name: w.wr_name,
        mb_id: w.mb_id,
        is_new: is_new(&w.wr_datetime, c.board.bo_new, c.now),
        datetime: w.wr_datetime,
        last: w.wr_last,
        hit: w.wr_hit,
        good: w.wr_good,
        nogood: w.wr_nogood,
        comment: w.wr_comment,
        file: w.wr_file,
        has_link: !w.wr_link1.is_empty() || !w.wr_link2.is_empty(),
        is_hot: c.board.bo_hot > 0 && w.wr_hit >= c.board.bo_hot,
    }
}

/// GET /api/v1/boards/{bo_table}/posts — list.php
pub async fn list_posts(
    State(s): State<AppState>,
    Path(bo_table): Path<String>,
    OptionalMember(claims): OptionalMember,
    meta: ClientMeta,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let c = ctx(&s, &bo_table, claims.as_ref(), &meta.ip).await?;
    check_list(&c.viewer, &c.cfg, &c.board, c.admin)?;

    let page = q.page.unwrap_or(1).max(1);
    let page_rows = c.board.bo_page_rows.max(1);
    let order = rules::order_by(&q.sst, &q.sod, &c.board.bo_sort_field);
    let search = q.search();
    let mut items = Vec::new();
    let mut search_info = serde_json::Value::Null;

    if let Some(sq) = &search {
        // 검색: wr_num 구간(cf_search_part) 단위로 나눠 검색
        let part = c.cfg.cf_search_part.max(1);
        let min_spt = write::min_wr_num(&c.db, &c.table).await?;
        let spt = q.spt.unwrap_or(min_spt);
        let total = write::search_count(&c.db, &c.table, sq, spt, part).await?;
        let pg = rules::paging(page, page_rows, total, 0, 0);
        let rows =
            write::search_list(&c.db, &c.table, sq, spt, part, &order, pg.from_record, pg.rows).await?;
        for (k, w) in rows.into_iter().enumerate() {
            let num = rules::list_num(total, page, page_rows, 0, i64::try_from(k).unwrap_or(0));
            items.push(list_item(&c, w, num, false));
        }
        let prev = spt - part;
        let next = spt + part;
        search_info = json!({
            "spt": spt,
            "prevSpt": (prev >= min_spt).then_some(prev),
            "nextSpt": (next < 0).then_some(next),
        });
        write::insert_popular(
            &c.db,
            &c.p,
            &rules::popular_words(&q.sfl, &q.stx),
            &c.now.ymd(),
            &meta.ip,
        )
        .await;
        return Ok(Json(json!({ "data": {
            "board": board_info(&c), "items": items, "page": page, "totalCount": total,
            "totalPage": pg.total_page, "pageRows": page_rows, "search": search_info,
        }})));
    }

    // 일반 목록: 공지 → 나머지
    let total = c.board.bo_count_write;
    let notice_ids = rules::notice_ids(&c.board.bo_notice);
    let notices = write::get_many(&c.db, &c.table, &notice_ids).await?;
    let notice_total = i64::try_from(notices.len()).unwrap_or(0);
    let notice_from = usize::try_from((page - 1) * page_rows).unwrap_or(0);
    let exclude: Vec<i64> = notices.iter().map(|n| n.wr_id).collect();
    let mut shown = 0i64;
    for (idx, n) in notices.into_iter().enumerate() {
        if idx < notice_from {
            continue;
        }
        items.push(list_item(&c, n, 0, true));
        shown += 1;
        if shown >= page_rows {
            break;
        }
    }
    let pg = rules::paging(page, page_rows, total, notice_total, shown);
    if pg.rows > 0 {
        let rows = write::list_plain(&c.db, &c.table, &exclude, &order, pg.from_record, pg.rows).await?;
        for (k, w) in rows.into_iter().enumerate() {
            let num = rules::list_num(total, page, page_rows, shown, i64::try_from(k).unwrap_or(0));
            items.push(list_item(&c, w, num, false));
        }
    }
    Ok(Json(json!({ "data": {
        "board": board_info(&c), "items": items, "page": page, "totalCount": total,
        "totalPage": pg.total_page, "pageRows": page_rows, "search": search_info,
    }})))
}

/// 비밀글/비밀댓글 열람 허가 토큰들 (헤더에 쉼표로 여러 개). PHP 의 ss_secret_*, ss_secret_comment_* 세션 대응.
fn grants(s: &AppState, headers: &HeaderMap) -> Vec<Vec<String>> {
    let now = chrono::Utc::now().timestamp();
    headers
        .get(SECRET_GRANT_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            v.split(',')
                .filter_map(|t| sign::verify(&s.0.config.jwt_secret, t.trim(), now))
                .collect()
        })
        .unwrap_or_default()
}

fn has_grant(grants: &[Vec<String>], kind: &str, bo_table: &str, id: i64) -> bool {
    grants
        .iter()
        .any(|f| f.len() == 3 && f[0] == kind && f[1] == bo_table && f[2] == id.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileView {
    no: i64,
    source: String,
    size: i64,
    download: i64,
    content: String,
    is_image: bool,
    width: i64,
    height: i64,
    datetime: String,
    download_url: String,
    image_url: Option<String>,
}

fn is_image(f: &FileRow) -> bool {
    // getimagesize 타입: 1 GIF, 2 JPG, 3 PNG, 18 WEBP
    matches!(f.bf_type, 1 | 2 | 3 | 18)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CommentView {
    wr_id: i64,
    depth: usize,
    name: String,
    mb_id: String,
    is_secret: bool,
    /// 비밀 댓글을 볼 수 없으면 None
    content: Option<String>,
    datetime: String,
    ip: Option<String>,
    can_edit: bool,
    can_reply: bool,
}

fn ip_view(c: &Ctx, ip: &str) -> Option<String> {
    if c.admin.is_some() {
        Some(ip.to_owned())
    } else if c.board.bo_use_ip_view != 0 {
        Some(mask_ip(ip))
    } else {
        None
    }
}

/// GET /api/v1/boards/{bo_table}/posts/{wr_id} — board.php(글읽기 분기) + view.php + view_comment.php
pub async fn view_post(
    State(s): State<AppState>,
    Path((bo_table, wr_id)): Path<(String, i64)>,
    OptionalMember(claims): OptionalMember,
    meta: ClientMeta,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let c = ctx(&s, &bo_table, claims.as_ref(), &meta.ip).await?;
    let not_found = || {
        AppError::not_found(
            "POST_NOT_FOUND",
            "글이 존재하지 않습니다. 글이 삭제되었거나 이동된 경우입니다.",
        )
    };
    let mut w = write::get(&c.db, &c.table, wr_id).await?.ok_or_else(not_found)?;
    if w.wr_is_comment != 0 {
        // 댓글 주소로 들어오면 원글을 보여준다 (PHP 는 원글로 이동)
        w = write::get(&c.db, &c.table, w.wr_parent)
            .await?
            .ok_or_else(not_found)?;
    }
    check_read(&c.db, &c.p, &c.viewer, &c.cfg, &c.group, &c.board, c.admin).await?;

    let v = &c.viewer;
    let is_author = v.is_member() && w.mb_id == v.mb_id;
    if !is_author && c.admin.is_none() && has_option(&w.wr_option, "secret") {
        let mut is_owner = false;
        if !w.wr_reply.is_empty() && v.is_member() {
            is_owner = write::root_author(&c.db, &c.table, w.wr_num)
                .await?
                .is_some_and(|m| m == v.mb_id);
        }
        if !is_owner && !has_grant(&grants(&s, &headers), "secret", &bo_table, w.wr_num) {
            return Err(AppError::forbidden(
                "POST_SECRET",
                "비밀글입니다. 비밀번호를 입력해 주세요.",
            ));
        }
    }

    // 조회수 + 읽기 포인트 (세션당 1회)
    let seen_key = format!("view:{bo_table}:{}:{}", w.wr_id, v.seen_key());
    if s.0.seen.first_time(&seen_key) {
        write::increment_hit(&c.db, &c.table, w.wr_id).await?;
        w.wr_hit += 1;
        let own_guest_post = !v.is_member() && c.board.bo_read_level == 1 && w.wr_ip == v.ip;
        if !is_author && !own_guest_post {
            let rp = c.board.bo_read_point;
            if c.cfg.cf_use_point != 0 && rp != 0 && v.point + rp < 0 {
                s.0.seen.forget(&seen_key);
                return Err(AppError::forbidden(
                    "POINT_NOT_ENOUGH",
                    format!(
                        "보유하신 포인트({})가 없거나 모자라서 글읽기({})가 불가합니다. 포인트를 모으신 후 다시 글읽기 해 주십시오.",
                        v.point, rp
                    ),
                ));
            }
            point::insert_point(
                &c.db,
                &c.p,
                &c.cfg,
                c.now,
                PointInput {
                    mb_id: &v.mb_id,
                    point: rp,
                    content: &format!("{} {} 글읽기", c.board.bo_subject, w.wr_id),
                    rel_table: &bo_table,
                    rel_id: &w.wr_id.to_string(),
                    rel_action: "읽기",
                },
            )
            .await?;
        }
    }

    // 첨부파일 (다운로드/이미지 링크는 짧은 수명의 서명 토큰 포함)
    let exp = chrono::Utc::now().timestamp() + FILE_TOKEN_SECS;
    let wid = w.wr_id.to_string();
    let token = sign::sign(
        &s.0.config.jwt_secret,
        &["dl", &bo_table, &wid, &v.mb_id, &v.ip],
        exp,
    );
    let files: Vec<FileView> = write::files(&c.db, &c.p, &bo_table, w.wr_id)
        .await?
        .into_iter()
        .map(|f| {
            let base = format!("/api/v1/boards/{bo_table}/posts/{}/files/{}", w.wr_id, f.bf_no);
            FileView {
                no: f.bf_no,
                is_image: is_image(&f),
                image_url: is_image(&f).then(|| format!("{base}/inline?t={token}")),
                download_url: format!("{base}?t={token}"),
                source: f.bf_source,
                size: f.bf_filesize,
                download: f.bf_download,
                content: f.bf_content,
                width: f.bf_width,
                height: f.bf_height,
                datetime: f.bf_datetime,
            }
        })
        .collect();

    // 댓글 (view_comment.php 비밀댓글 규칙)
    let grants = grants(&s, &headers);
    let can_comment = v.level >= c.board.bo_comment_level;
    let comments: Vec<CommentView> = write::comments(&c.db, &c.table, w.wr_id)
        .await?
        .into_iter()
        .map(|cm| {
            let visible = !has_option(&cm.wr_option, "secret")
                || c.admin.is_some()
                || is_author
                || (v.is_member() && cm.mb_id == v.mb_id)
                || has_grant(&grants, "secretc", &bo_table, cm.wr_id);
            CommentView {
                wr_id: cm.wr_id,
                depth: cm.wr_comment_reply.len(),
                is_secret: has_option(&cm.wr_option, "secret"),
                content: visible.then(|| cm.wr_content.clone()),
                ip: ip_view(&c, &cm.wr_ip),
                can_edit: v.is_member() && (cm.mb_id == v.mb_id || c.admin.is_some()),
                can_reply: (can_comment || c.admin.is_some()) && cm.wr_comment_reply.len() < 5,
                name: cm.wr_name,
                mb_id: cm.mb_id,
                datetime: cm.wr_datetime,
            }
        })
        .collect();

    let (prev, next) = if c.board.bo_use_list_view == 0 {
        write::neighbors(&c.db, &c.table, &w, q.search().as_ref()).await?
    } else {
        (None, None)
    };

    let can_manage = is_author || c.admin.is_some();
    let links: Vec<&String> = [&w.wr_link1, &w.wr_link2]
        .into_iter()
        .filter(|l| !l.is_empty())
        .collect();
    let secret = has_option(&w.wr_option, "secret");
    Ok(Json(json!({ "data": {
        "board": board_info(&c),
        "post": {
            "wrId": w.wr_id, "wrNum": w.wr_num, "depth": w.wr_reply.len(), "caName": w.ca_name,
            "subject": w.wr_subject, "content": w.wr_content, "contentMode": content_mode(&w.wr_option),
            "isSecret": secret, "name": w.wr_name, "mbId": w.mb_id, "datetime": w.wr_datetime,
            "hit": w.wr_hit, "good": w.wr_good, "nogood": w.wr_nogood, "commentCount": w.wr_comment,
            "links": links,
            "ip": ip_view(&c, &w.wr_ip),
        },
        "files": files,
        "comments": comments,
        "prev": prev,
        "next": next,
        "downloadPoint": c.board.bo_download_point,
        "perm": {
            "canReply": v.level >= c.board.bo_reply_level,
            "canEdit": can_manage,
            "canDelete": can_manage,
            "guestPassword": !v.is_member() && w.mb_id.is_empty(),
            "canComment": can_comment,
            "canGood": v.is_member() && c.board.bo_use_good != 0,
            "canNogood": v.is_member() && c.board.bo_use_nogood != 0,
            "canMove": w.wr_reply.is_empty() && matches!(c.admin, Some(Admin::Super | Admin::Group)),
        },
    }})))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockReq {
    password: String,
    /// 비밀댓글 확인일 때 댓글 wr_id (password.php?w=sc)
    comment_id: Option<i64>,
}

/// POST /api/v1/boards/{bo_table}/posts/{wr_id}/unlock — password_check.php (w=s 비밀글, w=sc 비밀댓글)
pub async fn unlock_post(
    State(s): State<AppState>,
    Path((bo_table, wr_id)): Path<(String, i64)>,
    OptionalMember(claims): OptionalMember,
    meta: ClientMeta,
    Json(req): Json<UnlockReq>,
) -> AppResult<Json<serde_json::Value>> {
    let c = ctx(&s, &bo_table, claims.as_ref(), &meta.ip).await?;
    if s.0
        .limiter
        .is_blocked(&meta.ip, &format!("secret:{bo_table}:{wr_id}"))
    {
        return Err(AppError::too_many(
            "AUTH_TOO_MANY",
            "시도가 너무 많습니다. 잠시 후 다시 시도하세요.",
        ));
    }
    let target = req.comment_id.unwrap_or(wr_id);
    let w = write::get(&c.db, &c.table, target)
        .await?
        .filter(|w| req.comment_id.is_none() || (w.wr_is_comment != 0 && w.wr_parent == wr_id))
        .ok_or_else(|| AppError::not_found("POST_NOT_FOUND", "글이 존재하지 않습니다."))?;
    // 회원 글은 wr_password 가 비어 있으므로 작성자 회원 비밀번호로 확인 (password_check.php 와 동일)
    let mut hash = w.wr_password.clone();
    if hash.is_empty() && !w.mb_id.is_empty() {
        hash = crate::g5::member::find(&c.db, &c.p, &w.mb_id)
            .await?
            .map(|m| m.mb_password)
            .unwrap_or_default();
    }
    let ok = !hash.is_empty()
        && matches!(
            password::verify(req.password.trim(), &hash),
            Verify::Ok | Verify::OkNeedsUpgrade
        );
    if !ok {
        s.0.limiter.fail(&meta.ip, &format!("secret:{bo_table}:{wr_id}"));
        return Err(AppError::forbidden("POST_PASSWORD_WRONG", "비밀번호가 틀립니다."));
    }
    let (kind, id) = if req.comment_id.is_some() {
        ("secretc", w.wr_id)
    } else {
        ("secret", w.wr_num)
    };
    let grant = sign::sign(
        &s.0.config.jwt_secret,
        &[kind, &bo_table, &id.to_string()],
        chrono::Utc::now().timestamp() + SECRET_GRANT_SECS,
    );
    Ok(Json(
        json!({ "data": { "grant": grant, "expiresIn": SECRET_GRANT_SECS } }),
    ))
}

#[derive(Deserialize)]
pub struct FileQuery {
    t: String,
}

/// 다운로드 토큰 확인 → (mb_id, ip)
fn file_token(s: &AppState, bo_table: &str, wr_id: i64, t: &str) -> AppResult<(String, String)> {
    let f = sign::verify(&s.0.config.jwt_secret, t, chrono::Utc::now().timestamp())
        .filter(|f| f.len() == 5 && f[0] == "dl" && f[1] == bo_table && f[2] == wr_id.to_string())
        .ok_or_else(|| {
            AppError::forbidden(
                "FILE_TOKEN_INVALID",
                "토큰 유효시간이 지났거나 토큰이 유효하지 않습니다. 새로고침 후 다시 시도해 주세요.",
            )
        })?;
    Ok((f[3].clone(), f[4].clone()))
}

async fn open_file(s: &AppState, bo_table: &str, f: &FileRow) -> AppResult<(tokio::fs::File, u64)> {
    let name = &f.bf_file;
    if name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.') {
        return Err(AppError::not_found("FILE_NOT_FOUND", "파일이 존재하지 않습니다."));
    }
    let path = std::path::Path::new(&s.0.config.data_path)
        .join("file")
        .join(bo_table)
        .join(name);
    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|_| AppError::not_found("FILE_NOT_FOUND", "파일이 존재하지 않습니다."))?;
    let len = file.metadata().await.map(|m| m.len()).unwrap_or(0);
    Ok((file, len))
}

/// RFC 5987 파일명 인코딩 (rawurlencode)
fn rawurlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn mime_of(name: &str) -> &'static str {
    match name.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}

fn stream(file: tokio::fs::File) -> Body {
    Body::from_stream(tokio_util::io::ReaderStream::new(file))
}

async fn file_ctx(
    s: &AppState,
    bo_table: &str,
    wr_id: i64,
    no: i64,
    t: &str,
) -> AppResult<(Ctx, WriteRow, FileRow)> {
    let (mb_id, ip) = file_token(s, bo_table, wr_id, t)?;
    let claims = (!mb_id.is_empty()).then_some(crate::auth::jwt::Claims {
        sub: mb_id,
        lvl: 0,
        adm: None,
        iat: 0,
        exp: 0,
    });
    let c = ctx(s, bo_table, claims.as_ref(), &ip).await?;
    let w = write::get(&c.db, &c.table, wr_id)
        .await?
        .ok_or_else(|| AppError::not_found("POST_NOT_FOUND", "글이 존재하지 않습니다."))?;
    let f = write::file(&c.db, &c.p, bo_table, wr_id, no)
        .await?
        .filter(|f| !f.bf_file.is_empty())
        .ok_or_else(|| AppError::not_found("FILE_NOT_FOUND", "파일 정보가 존재하지 않습니다."))?;
    Ok((c, w, f))
}

/// GET /api/v1/boards/{bo_table}/posts/{wr_id}/files/{no}?t= — download.php
pub async fn download(
    State(s): State<AppState>,
    Path((bo_table, wr_id, no)): Path<(String, i64, i64)>,
    Query(q): Query<FileQuery>,
) -> AppResult<Response> {
    let (c, w, f) = file_ctx(&s, &bo_table, wr_id, no, &q.t).await?;
    let v = &c.viewer;
    let dp = c.board.bo_download_point;
    if dp < 0 && !v.is_member() {
        return Err(AppError::unauthorized(
            "AUTH_REQUIRED",
            "다운로드 권한이 없습니다. 회원이시라면 로그인 후 이용해 보십시오.",
        ));
    }
    if v.level < c.board.bo_download_level {
        return Err(AppError::forbidden(
            "FILE_NO_PERMISSION",
            "다운로드 권한이 없습니다.",
        ));
    }
    let (file, len) = open_file(&s, &bo_table, &f).await?;

    let down_key = format!("down:{bo_table}:{wr_id}:{}", v.seen_key());
    if s.0.seen.first_time(&down_key) {
        let is_author = v.is_member() && w.mb_id == v.mb_id;
        if !(is_author || c.admin.is_some()) {
            if v.point + dp < 0 {
                s.0.seen.forget(&down_key);
                return Err(AppError::forbidden(
                    "POINT_NOT_ENOUGH",
                    format!(
                        "보유하신 포인트({})가 없거나 모자라서 다운로드({dp})가 불가합니다.",
                        v.point
                    ),
                ));
            }
            point::insert_point(
                &c.db,
                &c.p,
                &c.cfg,
                c.now,
                PointInput {
                    mb_id: &v.mb_id,
                    point: dp,
                    content: &format!("{} {wr_id} 파일 다운로드", c.board.bo_subject),
                    rel_table: &bo_table,
                    rel_id: &wr_id.to_string(),
                    rel_action: "다운로드",
                },
            )
            .await?;
        }
    }
    if s.0.seen.first_time(&format!("{down_key}:{no}")) {
        write::increment_download(&c.db, &c.p, &bo_table, wr_id, no).await?;
    }

    let disp = format!(
        "attachment; filename=\"{0}\"; filename*=UTF-8''{0}",
        rawurlencode(&f.bf_source)
    );
    let mut res = stream(file).into_response();
    let h = res.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    h.insert(header::CONTENT_LENGTH, HeaderValue::from(len));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if let Ok(v) = HeaderValue::from_str(&disp) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    Ok(res)
}

/// GET .../files/{no}/inline?t= — 본문 이미지 표시 (포인트·다운로드 수 없음)
pub async fn inline_image(
    State(s): State<AppState>,
    Path((bo_table, wr_id, no)): Path<(String, i64, i64)>,
    Query(q): Query<FileQuery>,
) -> AppResult<Response> {
    let (_c, _w, f) = file_ctx(&s, &bo_table, wr_id, no, &q.t).await?;
    if !is_image(&f) {
        return Err(AppError::not_found("FILE_NOT_FOUND", "이미지가 아닙니다."));
    }
    let (file, len) = open_file(&s, &bo_table, &f).await?;
    let mut res = stream(file).into_response();
    let h = res.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(mime_of(&f.bf_file)),
    );
    h.insert(header::CONTENT_LENGTH, HeaderValue::from(len));
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=3600"),
    );
    h.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_filename() {
        assert_eq!(rawurlencode("a b.txt"), "a%20b.txt");
        assert_eq!(rawurlencode("한.png"), "%ED%95%9C.png");
        assert_eq!(rawurlencode("x\"y"), "x%22y");
    }
}
