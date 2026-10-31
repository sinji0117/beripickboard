//! 게시판 읽기 (bbs/board.php, list.php, view.php, view_comment.php, download.php)

pub mod handlers;
pub mod viewer;

use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

/// PHP 세션 플래그(ss_view_*, ss_down_*) 대응. 같은 열람자가 일정 시간 안에 다시 보면 조회수·포인트를 중복 처리하지 않는다.
pub struct SeenCache {
    ttl: Duration,
    map: Mutex<HashMap<String, Instant>>,
}

impl SeenCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            map: Mutex::new(HashMap::new()),
        }
    }

    /// 처음 보는 키면 true 를 돌려주고 기록한다.
    pub fn first_time(&self, key: &str) -> bool {
        let Ok(mut m) = self.map.lock() else { return true };
        if m.len() > 200_000 {
            let ttl = self.ttl;
            m.retain(|_, t| t.elapsed() < ttl);
        }
        match m.get(key) {
            Some(t) if t.elapsed() < self.ttl => false,
            _ => {
                m.insert(key.to_owned(), Instant::now());
                true
            }
        }
    }

    /// PHP 가 alert 로 중단하면 세션 플래그가 남지 않는 것과 맞춘다.
    pub fn forget(&self, key: &str) {
        if let Ok(mut m) = self.map.lock() {
            m.remove(key);
        }
    }

    #[cfg(test)]
    pub fn contains(&self, key: &str) -> bool {
        self.map
            .lock()
            .ok()
            .and_then(|m| m.get(key).map(|t| t.elapsed() < self.ttl))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seen_once() {
        let c = SeenCache::new(Duration::from_secs(60));
        assert!(!c.contains("a"));
        assert!(c.first_time("a"));
        assert!(!c.first_time("a"));
        assert!(c.contains("a"));
        assert!(c.first_time("b"));
    }
}
