//! 로그인 실패 제한 (메모리). 다중 인스턴스 운영 시 Redis 로 교체.

use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

pub struct LoginLimiter {
    max_failures: u32,
    window: Duration,
    map: Mutex<HashMap<String, (u32, Instant)>>,
}

impl LoginLimiter {
    pub fn new(max_failures: u32, window: Duration) -> Self {
        Self {
            max_failures,
            window,
            map: Mutex::new(HashMap::new()),
        }
    }

    fn key(ip: &str, mb_id: &str) -> String {
        format!("{ip}|{}", mb_id.to_ascii_lowercase())
    }

    pub fn is_blocked(&self, ip: &str, mb_id: &str) -> bool {
        let Ok(map) = self.map.lock() else { return false };
        map.get(&Self::key(ip, mb_id))
            .is_some_and(|(n, t)| *n >= self.max_failures && t.elapsed() < self.window)
    }

    pub fn fail(&self, ip: &str, mb_id: &str) {
        let Ok(mut map) = self.map.lock() else { return };
        if map.len() > 100_000 {
            let w = self.window;
            map.retain(|_, (_, t)| t.elapsed() < w);
        }
        let e = map.entry(Self::key(ip, mb_id)).or_insert((0, Instant::now()));
        if e.1.elapsed() >= self.window {
            *e = (0, Instant::now());
        }
        e.0 += 1;
    }

    pub fn success(&self, ip: &str, mb_id: &str) {
        if let Ok(mut map) = self.map.lock() {
            map.remove(&Self::key(ip, mb_id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_max() {
        let l = LoginLimiter::new(3, Duration::from_secs(60));
        for _ in 0..3 {
            assert!(!l.is_blocked("1.1.1.1", "Admin"));
            l.fail("1.1.1.1", "admin");
        }
        assert!(l.is_blocked("1.1.1.1", "ADMIN"));
        assert!(!l.is_blocked("2.2.2.2", "admin"));
        l.success("1.1.1.1", "admin");
        assert!(!l.is_blocked("1.1.1.1", "admin"));
    }
}
