-- Migration: Add password reset fields to users table
-- Description: Add reset_token and reset_token_expires columns to users table

-- Add reset token columns if they don't exist
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS reset_token TEXT,
ADD COLUMN IF NOT EXISTS reset_token_expires TIMESTAMP WITH TIME ZONE;

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_users_reset_token ON users(reset_token) WHERE reset_token IS NOT NULL;

-- Add comments
COMMENT ON COLUMN users.reset_token IS 'Token for password reset functionality';
COMMENT ON COLUMN users.reset_token_expires IS 'Expiration timestamp for reset token';