-- 系统设置表
-- 存储全局系统配置，如品牌、外观、安全设置等

CREATE TABLE IF NOT EXISTS system_settings (
    key VARCHAR(64) PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 插入默认设置
INSERT INTO system_settings (key, value) VALUES
    ('brand_name', 'UNOIDC'),
    ('logo_url', ''),
    ('login_background_url', ''),
    ('login_layout', 'split-left'),
    ('session_timeout', '24'),
    ('max_login_attempts', '5')
ON CONFLICT (key) DO NOTHING;
