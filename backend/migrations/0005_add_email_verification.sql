-- Email verification tokens table for secure email changes
-- ============================================
CREATE TABLE IF NOT EXISTS email_verification_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    new_email VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_email_verification_user ON email_verification_tokens(user_id);
CREATE INDEX idx_email_verification_hash ON email_verification_tokens(token_hash);
CREATE INDEX idx_email_verification_expires ON email_verification_tokens(expires_at);

-- Cleanup old tokens regularly
-- DELETE FROM email_verification_tokens WHERE expires_at < NOW();
