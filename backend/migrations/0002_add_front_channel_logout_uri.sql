-- Migration: Add front_channel_logout_uri to clients table
-- 
-- 用于支持 OIDC Front-Channel Logout 规范

ALTER TABLE clients
ADD COLUMN IF NOT EXISTS front_channel_logout_uri VARCHAR(512);
