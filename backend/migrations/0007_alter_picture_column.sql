-- 将 users.picture 列从 VARCHAR(512) 改为 TEXT，以支持 base64 编码的图片数据
ALTER TABLE users ALTER COLUMN picture TYPE TEXT;
