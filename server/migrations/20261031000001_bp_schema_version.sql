-- beripickboard 자체 스키마 버전 (g5 테이블과 분리, doc/03_architecture.md §4.2)
CREATE TABLE IF NOT EXISTS bp_schema_version (
  id          INT UNSIGNED NOT NULL AUTO_INCREMENT,
  app_version VARCHAR(20)  NOT NULL,
  applied_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO bp_schema_version (app_version) VALUES ('0.0.1');
