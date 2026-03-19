BEGIN;

-- Ensure id is primary key (skip if already set)
ALTER TABLE users ADD PRIMARY KEY (id);

-- Index for fast role-based lookups
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- Index for fast status-based lookups
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);

-- Index for sorting/filtering by created_at (used in admin queries)
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- Unique index on email (important for login lookups)
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email);

COMMIT;

