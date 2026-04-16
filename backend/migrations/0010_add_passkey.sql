-- Passkey credentials table
CREATE TABLE IF NOT EXISTS passkey_credentials (
    id TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    public_key BYTEA NOT NULL,
    counter BIGINT NOT NULL DEFAULT 0,
    device_type VARCHAR(50),
    backed_up BOOLEAN DEFAULT false,
    transports TEXT[],
    display_name VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_passkey_credentials_user_id ON passkey_credentials(user_id);

-- WebAuthn challenges table (temporary state, no new middleware needed)
CREATE TABLE IF NOT EXISTS webauthn_challenges (
    challenge_hash BYTEA PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    purpose VARCHAR(20) NOT NULL,
    state_data BYTEA NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
