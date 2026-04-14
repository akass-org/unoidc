-- Add enable_silent_authorize column to clients table
-- 允许每个应用单独配置是否启用"无感授权"

ALTER TABLE clients ADD COLUMN IF NOT EXISTS enable_silent_authorize BOOLEAN NOT NULL DEFAULT false;

-- Create index for faster queries
CREATE INDEX IF NOT EXISTS idx_clients_enable_silent_authorize ON clients(enable_silent_authorize);
