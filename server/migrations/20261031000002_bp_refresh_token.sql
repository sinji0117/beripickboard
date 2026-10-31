-- 리프레시 토큰 (doc/09_api.md §2). 원문 토큰은 쿠키에만, DB에는 SHA-256 해시만 저장.
CREATE TABLE IF NOT EXISTS bp_refresh_token (
  token_hash  CHAR(64)     NOT NULL,
  family_id   CHAR(32)     NOT NULL,
  mb_id       VARCHAR(20)  NOT NULL,
  auto_login  TINYINT(1)   NOT NULL DEFAULT 0,
  user_agent  VARCHAR(255) NOT NULL DEFAULT '',
  ip          VARCHAR(45)  NOT NULL DEFAULT '',
  created_at  DATETIME     NOT NULL,
  expires_at  DATETIME     NOT NULL,
  revoked_at  DATETIME     NULL,
  PRIMARY KEY (token_hash),
  KEY idx_family (family_id),
  KEY idx_mb_id (mb_id),
  KEY idx_expires (expires_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO bp_schema_version (app_version) VALUES ('0.0.3');
