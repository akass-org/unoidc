-- 生成测试审计日志数据脚本
-- 这个脚本会创建一些测试审计日志记录用于测试审计日志功能

-- 首先获取或创建测试用户和客户端
-- 插入一些测试审计日志

-- 记录登录成功事件
INSERT INTO audit_logs (
    id, actor_user_id, client_id, correlation_id, action,
    target_type, target_id, outcome, reason_code, metadata,
    ip_address, user_agent, created_at
) VALUES (
    gen_random_uuid(),
    (SELECT id FROM users LIMIT 1),
    NULL,
    'corr-' || gen_random_uuid()::text,
    'login',
    'user_session',
    'test-session-1',
    'success',
    NULL,
    '{"event": "login_success"}'::jsonb,
    '192.168.1.100',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
    NOW() - INTERVAL '5 minutes'
);

-- 记录登录成功事件2
INSERT INTO audit_logs (
    id, actor_user_id, client_id, correlation_id, action,
    target_type, target_id, outcome, reason_code, metadata,
    ip_address, user_agent, created_at
) VALUES (
    gen_random_uuid(),
    (SELECT id FROM users LIMIT 1),
    NULL,
    'corr-' || gen_random_uuid()::text,
    'login',
    'user_session',
    'test-session-2',
    'success',
    NULL,
    '{"event": "login_success"}'::jsonb,
    '192.168.1.101',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)',
    NOW() - INTERVAL '10 minutes'
);

-- 记录登录失败事件
INSERT INTO audit_logs (
    id, actor_user_id, client_id, correlation_id, action,
    target_type, target_id, outcome, reason_code, metadata,
    ip_address, user_agent, created_at
) VALUES (
    gen_random_uuid(),
    NULL,
    NULL,
    'corr-' || gen_random_uuid()::text,
    'login',
    'user_session',
    'unknown-user',
    'failure',
    'invalid_credentials',
    '{"event": "login_failure", "username": "testuser"}'::jsonb,
    '192.168.1.102',
    'Mozilla/5.0 (X11; Linux x86_64)',
    NOW() - INTERVAL '15 minutes'
);

-- 记录令牌发放事件
INSERT INTO audit_logs (
    id, actor_user_id, client_id, correlation_id, action,
    target_type, target_id, outcome, reason_code, metadata,
    ip_address, user_agent, created_at
) VALUES (
    gen_random_uuid(),
    (SELECT id FROM users LIMIT 1),
    (SELECT id FROM clients LIMIT 1),
    'corr-' || gen_random_uuid()::text,
    'token_issued',
    'id_token',
    gen_random_uuid()::text,
    'success',
    NULL,
    '{"event": "token_issued", "token_type": "id_token"}'::jsonb,
    '192.168.1.100',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
    NOW() - INTERVAL '3 minutes'
);

-- 记录登出事件
INSERT INTO audit_logs (
    id, actor_user_id, client_id, correlation_id, action,
    target_type, target_id, outcome, reason_code, metadata,
    ip_address, user_agent, created_at
) VALUES (
    gen_random_uuid(),
    (SELECT id FROM users LIMIT 1),
    NULL,
    'corr-' || gen_random_uuid()::text,
    'logout',
    'user_session',
    'test-session-1',
    'success',
    NULL,
    '{"event": "logout"}'::jsonb,
    '192.168.1.100',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
    NOW() - INTERVAL '1 minute'
);

-- 记录用户创建事件
INSERT INTO audit_logs (
    id, actor_user_id, client_id, correlation_id, action,
    target_type, target_id, outcome, reason_code, metadata,
    ip_address, user_agent, created_at
) VALUES (
    gen_random_uuid(),
    (SELECT id FROM users WHERE username != 'admin' LIMIT 1),
    NULL,
    'corr-' || gen_random_uuid()::text,
    'user_created',
    'user_account',
    gen_random_uuid()::text,
    'success',
    NULL,
    '{"event": "user_created", "username": "newuser"}'::jsonb,
    '192.168.1.200',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
    NOW() - INTERVAL '30 minutes'
);

-- 记录密码重置事件
INSERT INTO audit_logs (
    id, actor_user_id, client_id, correlation_id, action,
    target_type, target_id, outcome, reason_code, metadata,
    ip_address, user_agent, created_at
) VALUES (
    gen_random_uuid(),
    (SELECT id FROM users WHERE username != 'admin' LIMIT 1),
    NULL,
    'corr-' || gen_random_uuid()::text,
    'password_reset',
    'user_account',
    (SELECT id FROM users LIMIT 1 OFFSET 1)::text,
    'success',
    NULL,
    '{"event": "password_reset"}'::jsonb,
    '192.168.1.200',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
    NOW() - INTERVAL '20 minutes'
);

-- 记录授权同意事件
INSERT INTO audit_logs (
    id, actor_user_id, client_id, correlation_id, action,
    target_type, target_id, outcome, reason_code, metadata,
    ip_address, user_agent, created_at
) VALUES (
    gen_random_uuid(),
    (SELECT id FROM users LIMIT 1),
    (SELECT id FROM clients LIMIT 1),
    'corr-' || gen_random_uuid()::text,
    'consent_granted',
    'user_consent',
    (SELECT id FROM users LIMIT 1)::text || ':' || (SELECT id FROM clients LIMIT 1)::text,
    'success',
    NULL,
    '{"event": "consent_granted", "scopes": ["openid", "profile", "email"]}'::jsonb,
    '192.168.1.100',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
    NOW() - INTERVAL '25 minutes'
);

SELECT COUNT(*) as audit_logs_created FROM audit_logs WHERE created_at > NOW() - INTERVAL '1 hour';
