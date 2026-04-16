-- Make password_hash nullable to support passkey-only accounts
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;
