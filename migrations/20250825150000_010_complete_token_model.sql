-- Migration: Complete Token Model
-- Add missing fields and enums to tokens table

-- Create token_type enum
CREATE TYPE token_type AS ENUM (
    'fungible',
    'non_fungible', 
    'semi_fungible',
    'utility',
    'security',
    'governance'
);

-- Add missing is_active column
ALTER TABLE tokens 
ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT true;

-- Update token_type column from varchar to enum
-- First, set a default value for any NULL token_type values
UPDATE tokens 
SET token_type = 'fungible' 
WHERE token_type IS NULL OR token_type = '';

-- Convert the column to use the enum type
ALTER TABLE tokens 
ALTER COLUMN token_type TYPE token_type 
USING CASE 
    WHEN token_type = 'fungible' THEN 'fungible'::token_type
    WHEN token_type = 'non_fungible' OR token_type = 'nft' THEN 'non_fungible'::token_type
    WHEN token_type = 'semi_fungible' THEN 'semi_fungible'::token_type
    WHEN token_type = 'utility' THEN 'utility'::token_type
    WHEN token_type = 'security' THEN 'security'::token_type
    WHEN token_type = 'governance' THEN 'governance'::token_type
    ELSE 'fungible'::token_type
END;

-- Make token_type NOT NULL with default
ALTER TABLE tokens 
ALTER COLUMN token_type SET NOT NULL,
ALTER COLUMN token_type SET DEFAULT 'fungible'::token_type;

-- Make owner_id NOT NULL (it's already populated based on our check)
ALTER TABLE tokens 
ALTER COLUMN owner_id SET NOT NULL;

-- Create additional indexes for performance
CREATE INDEX IF NOT EXISTS idx_tokens_is_active ON tokens(is_active);
CREATE INDEX IF NOT EXISTS idx_tokens_token_type ON tokens(token_type);
CREATE INDEX IF NOT EXISTS idx_tokens_active_status ON tokens(is_active, status);

-- Add foreign key constraint for owner_id if users table exists
-- (Uncomment if you want to enforce referential integrity)
-- ALTER TABLE tokens 
-- ADD CONSTRAINT fk_tokens_owner_id 
-- FOREIGN KEY (owner_id) REFERENCES users(id);

-- Add foreign key constraint for project_id if not exists
-- (Uncomment if you want to enforce referential integrity)
-- ALTER TABLE tokens 
-- ADD CONSTRAINT fk_tokens_project_id 
-- FOREIGN KEY (project_id) REFERENCES projects(id);