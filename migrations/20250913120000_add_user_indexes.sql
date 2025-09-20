-- Forward migration: add useful indexes for performance

-- Index on role for admin filtering
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- Index on status for KYC/verification filtering
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);

-- Index on created_at for dashboards / sorting
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);
