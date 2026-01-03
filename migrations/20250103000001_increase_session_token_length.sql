-- Increase session token column length to TEXT to accommodate JWTs
-- JWT tokens can be 300-500+ characters, VARCHAR(255) is too small
ALTER TABLE sessions
ALTER COLUMN token TYPE TEXT;
