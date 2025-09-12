-- migrations/0007_fix_tokens_table_columns.sql

-- Add token_type column
ALTER TABLE tokens ADD COLUMN IF NOT EXISTS token_type VARCHAR(50);

-- Add owner_id column
ALTER TABLE tokens ADD COLUMN IF NOT EXISTS owner_id UUID REFERENCES users(id);

-- Optional: Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_tokens_token_type ON tokens(token_type);
CREATE INDEX IF NOT EXISTS idx_tokens_owner_id ON tokens(owner_id);

