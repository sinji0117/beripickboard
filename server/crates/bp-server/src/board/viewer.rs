//! 열람자 (회원 또는 비회원)와 게시판별 권한 판정

use crate::{
    auth::jwt::Claims,
    error::{AppError, AppResult},
    g5::{
        board::{self as g5board, Board, Group},
        config::G5Config,
        member,
    },
};
use bp_g5compat::{member::is_super_admin, table::Prefix};
use sqlx::MySqlPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admin {
    Super,
    Group,
    Board,
}

impl Admin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Super => "super",
            Self::Group => "group",
            Self::Board => "board",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Viewer {
    /// 비회원이면 빈 문자열
    pub mb_id: String,
    pub level: i64,
    pub point: i64,
    pub certify: String,
    pub adult: bool,
    pub ip: String,
}

impl Viewer {
    pub fn is_member(&self) -> bool {
        !self.mb_id.is_empty()
    }

    /// 조회수·다운로드 중복 방지 키 (회원은 아이디, 비회원은 IP)
    pub fn seen_key(&self) -> String {
        if self.is_member() {
            format!("m:{}", self.mb_id)
        } else {
            format!("ip:{}", self.ip)
        }
    }

    /// 토큰의 회원을 DB 에서 다시 읽는다(레벨·포인트 최신값). 탈퇴·차단 회원은 비회원 취급.
    pub async fn load(
        db: &MySqlPool,
        p: &Prefix,
        claims: Option<&Claims>,
        ip: &str,
        today_ymd: &str,
    ) -> AppResult<Self> {
        let guest = Self {
            mb_id: String::new(),
            level: 1,
            point: 0,
            certify: String::new(),
            adult: false,
            ip: ip.to_owned(),
        };
        let Some(c) = claims else { return Ok(guest) };
        let Some(m) = member::find(db, p, &c.sub).await? else {
            return Ok(guest);
        };
        use bp_g5compat::member::ymd_reached;
        if ymd_reached(&m.mb_leave_date, today_ymd) || ymd_reached(&m.mb_intercept_date, today_ymd) {
            return Ok(guest);
        }
        Ok(Self {
            mb_id: m.mb_id,
            level: m.mb_level,
            point: m.mb_point,
            certify: m.mb_certify,
            adult: m.mb_adult != 0,
            ip: ip.to_owned(),
        })
    }

    /// common.php `is_admin()` (게시판·그룹 문맥 포함)
    pub fn admin(&self, cfg: &G5Config, group: &Group, board: &Board) -> Option<Admin> {
        if !self.is_member() {
            None
        } else if is_super_admin(&cfg.cf_admin, &self.mb_id) {
            Some(Admin::Super)
        } else if !group.gr_admin.is_empty() && group.gr_admin == self.mb_id {
            Some(Admin::Group)
        } else if !board.bo_admin.is_empty() && board.bo_admin == self.mb_id {
            Some(Admin::Board)
        } else {
            None
        }
    }
}

fn need_login(v: &Viewer, msg: &str) -> AppError {
    if v.is_member() {
        AppError::forbidden("BOARD_NO_PERMISSION", msg)
    } else {
        AppError::unauthorized(
            "AUTH_REQUIRED",
            format!("{msg} 회원이시라면 로그인 후 이용해 보십시오."),
        )
    }
}

/// board.php 본인확인 게시판 검사
pub fn check_cert(v: &Viewer, cfg: &G5Config, b: &Board, admin: Option<Admin>) -> AppResult<()> {
    if b.bo_use_cert.is_empty() || cfg.cf_cert_use == 0 || admin.is_some() {
        return Ok(());
    }
    if !v.is_member() {
        return Err(need_login(
            v,
            "이 게시판은 본인확인 하신 회원님만 글읽기가 가능합니다.",
        ));
    }
    if b.bo_use_cert == "cert" && v.certify.is_empty() {
        return Err(AppError::forbidden(
            "BOARD_CERT_REQUIRED",
            "이 게시판은 본인확인 하신 회원님만 글읽기가 가능합니다.",
        ));
    }
    if b.bo_use_cert == "adult" && !v.adult {
        return Err(AppError::forbidden(
            "BOARD_ADULT_REQUIRED",
            "이 게시판은 본인확인으로 성인인증 된 회원님만 글읽기가 가능합니다.",
        ));
    }
    Ok(())
}

/// 목록 권한 (board.php else 분기)
pub fn check_list(v: &Viewer, cfg: &G5Config, b: &Board, admin: Option<Admin>) -> AppResult<()> {
    if v.level < b.bo_list_level {
        return Err(need_login(v, "목록을 볼 권한이 없습니다."));
    }
    check_cert(v, cfg, b, admin)
}

/// 글읽기 권한 (그룹 접근 + bo_read_level + 본인확인)
pub async fn check_read(
    db: &MySqlPool,
    p: &Prefix,
    v: &Viewer,
    cfg: &G5Config,
    g: &Group,
    b: &Board,
    admin: Option<Admin>,
) -> AppResult<()> {
    if g.gr_use_access != 0 {
        if !v.is_member() {
            return Err(need_login(v, "비회원은 이 게시판에 접근할 권한이 없습니다."));
        }
        if !matches!(admin, Some(Admin::Super | Admin::Group))
            && !g5board::is_group_member(db, p, &g.gr_id, &v.mb_id).await?
        {
            return Err(AppError::forbidden(
                "BOARD_GROUP_DENIED",
                "접근 권한이 없으므로 글읽기가 불가합니다.",
            ));
        }
    }
    if v.level < b.bo_read_level {
        return Err(need_login(v, "글을 읽을 권한이 없습니다."));
    }
    check_cert(v, cfg, b, admin)
}
