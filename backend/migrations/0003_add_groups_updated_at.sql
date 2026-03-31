-- Add updated_at column to groups table (M-22)
-- Tracks when group information was last modified

ALTER TABLE groups
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;

-- Set initial value for existing rows
UPDATE groups SET updated_at = created_at WHERE updated_at IS NULL;

-- Add trigger to automatically update updated_at on modification
CREATE OR REPLACE FUNCTION update_groups_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_groups_updated_at ON groups;

CREATE TRIGGER trigger_groups_updated_at
    BEFORE UPDATE ON groups
    FOR EACH ROW
    EXECUTE FUNCTION update_groups_updated_at();
