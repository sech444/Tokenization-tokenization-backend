-- Migration: Add recommended indexes and constraints to users table
-- Date: 2025-09-13

BEGIN;

-- Ensure email is unique (important for login/auth)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND tablename = 'users'
          AND indexname = 'idx_users_email_unique'
    ) THEN
        CREATE UNIQUE INDEX idx_users_email_unique ON users(email);
    END IF;
END$$;

-- Index for role lookups (admin dashboards)
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- Index for status lookups (KYC / verification)
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);

-- Index for admin dashboard ordering
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- Index for last login queries (optional, audit / monitoring)
CREATE INDEX IF NOT EXISTS idx_users_last_login ON users(last_login DESC);

-- Index for searching by username (optional, profile lookups)
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

COMMIT;
