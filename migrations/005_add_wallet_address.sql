-- Add wallet_address column to users table (only if not exists)
DO $$ BEGIN
    BEGIN
        ALTER TABLE users ADD COLUMN wallet_address VARCHAR(42);
    EXCEPTION
        WHEN duplicate_column THEN NULL;
    END;
END $$;

-- Add index for wallet_address lookups (only if not exists)
CREATE INDEX IF NOT EXISTS idx_users_wallet_address ON users(wallet_address);

-- Add comment for documentation
COMMENT ON COLUMN users.wallet_address IS 'Ethereum wallet address (42 characters including 0x prefix)';

-- Add constraint for wallet address format validation (only if not exists)
DO $$ BEGIN
    BEGIN
        ALTER TABLE users
        ADD CONSTRAINT wallet_address_format
        CHECK (wallet_address ~ '^0x[a-fA-F0-9]{40}$') NOT VALID;
    EXCEPTION
        WHEN duplicate_object THEN NULL;
    END;
END $$;