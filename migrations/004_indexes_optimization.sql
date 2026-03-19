

-- Migration 004 (rewritten): Indexes, constraints, and performance helpers
-- Safe for production: avoids immutability errors and legacy-data blockers

-- ===== Extensions =====
CREATE EXTENSION IF NOT EXISTS pg_trgm;
-- Optional (used by get_slow_queries); may require superuser privileges
-- CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- ===== Immutable helper functions for time bucketing =====
DROP FUNCTION IF EXISTS immutable_year(timestamptz);
CREATE OR REPLACE FUNCTION immutable_year(ts timestamptz)
RETURNS int LANGUAGE sql IMMUTABLE PARALLEL SAFE AS $$
  SELECT EXTRACT(YEAR FROM ts)::int;
$$;

DROP FUNCTION IF EXISTS immutable_month(timestamptz);
CREATE OR REPLACE FUNCTION immutable_month(ts timestamptz)
RETURNS int LANGUAGE sql IMMUTABLE PARALLEL SAFE AS $$
  SELECT EXTRACT(MONTH FROM ts)::int;
$$;

-- ===== User authentication and session management =====
CREATE INDEX IF NOT EXISTS idx_users_email_status_role ON users(email, status, role);
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_token ON user_sessions(user_id, session_token);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at);

-- ===== Project search and filtering optimizations =====
CREATE INDEX IF NOT EXISTS idx_projects_status_type_location ON projects(status, project_type, location);
CREATE INDEX IF NOT EXISTS idx_projects_funding_status ON projects(status, funds_raised, total_value);
CREATE INDEX IF NOT EXISTS idx_projects_tokenized_active ON projects(is_tokenized, status) WHERE status IN ('active', 'approved');
CREATE INDEX IF NOT EXISTS idx_projects_owner_status ON projects(owner_id, status);
CREATE INDEX IF NOT EXISTS idx_projects_created_funding ON projects(created_at DESC, funds_raised DESC);

-- ===== Transaction processing and reporting =====
CREATE INDEX IF NOT EXISTS idx_transactions_user_date_type ON transactions(user_id, created_at DESC, transaction_type);
CREATE INDEX IF NOT EXISTS idx_transactions_project_status_amount ON transactions(project_id, status, amount DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_date_status_type ON transactions(created_at DESC, status, transaction_type);
CREATE INDEX IF NOT EXISTS idx_transactions_blockchain_status ON transactions(blockchain_tx_hash, status) WHERE blockchain_tx_hash IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_transactions_payment_reference ON transactions(payment_reference) WHERE payment_reference IS NOT NULL;

-- ===== Token trading and analytics =====
CREATE INDEX IF NOT EXISTS idx_tokens_status_price ON tokens(status, current_price DESC);
CREATE INDEX IF NOT EXISTS idx_tokens_project_status ON tokens(project_id, status);
CREATE INDEX IF NOT EXISTS idx_tokens_symbol_status ON tokens(symbol, status);

-- ===== KYC and compliance optimization =====
CREATE INDEX IF NOT EXISTS idx_kyc_user_status_risk ON kyc_verifications(user_id, verification_status, risk_level);
CREATE INDEX IF NOT EXISTS idx_kyc_status_created_provider ON kyc_verifications(verification_status, created_at DESC, verification_provider);
CREATE INDEX IF NOT EXISTS idx_kyc_expiry_status ON kyc_verifications(expiry_date, verification_status) WHERE expiry_date IS NOT NULL;

-- ===== KYC documents optimization =====
CREATE INDEX IF NOT EXISTS idx_kyc_documents_verification_type ON kyc_documents(kyc_verification_id, document_type, verification_status);
CREATE INDEX IF NOT EXISTS idx_kyc_documents_hash ON kyc_documents(file_hash);

-- ===== AML screening optimization =====
CREATE INDEX IF NOT EXISTS idx_aml_screening_kyc_result ON aml_screenings(kyc_verification_id, screening_result, screening_type);
CREATE INDEX IF NOT EXISTS idx_aml_screening_provider_ref ON aml_screenings(screening_provider, screening_reference_id);

-- ===== AML matches optimization =====
CREATE INDEX IF NOT EXISTS idx_aml_matches_screening_reviewed ON aml_matches(aml_screening_id, reviewed, false_positive);
CREATE INDEX IF NOT EXISTS idx_aml_matches_score_type ON aml_matches(match_score DESC, match_type);

-- ===== Compliance profiles =====
CREATE INDEX IF NOT EXISTS idx_compliance_user_rating_type ON compliance_profiles(user_id, risk_rating, investor_type);
CREATE INDEX IF NOT EXISTS idx_compliance_review_dates ON compliance_profiles(next_review_date, last_review_date);
CREATE INDEX IF NOT EXISTS idx_compliance_accredited ON compliance_profiles(accredited_investor, accreditation_verified) WHERE accredited_investor = true;

-- ===== Compliance audit logs =====
CREATE INDEX IF NOT EXISTS idx_compliance_audit_user_action_date ON compliance_audit_logs(user_id, action_type, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_compliance_audit_performer_date ON compliance_audit_logs(performed_by, created_at DESC);

-- ===== Regulatory reporting =====
CREATE INDEX IF NOT EXISTS idx_regulatory_reports_type_jurisdiction ON regulatory_reports(report_type, jurisdiction, reporting_period_start);
CREATE INDEX IF NOT EXISTS idx_regulatory_reports_status_date ON regulatory_reports(submission_status, created_at DESC);

-- ===== Suspicious activity reports =====
CREATE INDEX IF NOT EXISTS idx_sar_user_level_status ON suspicious_activity_reports(user_id, suspicion_level, status);
CREATE INDEX IF NOT EXISTS idx_sar_transaction_status ON suspicious_activity_reports(transaction_id, status);

-- ===== Transaction monitoring =====
CREATE INDEX IF NOT EXISTS idx_monitoring_alerts_rule_severity ON transaction_monitoring_alerts(rule_id, severity, status);
CREATE INDEX IF NOT EXISTS idx_monitoring_alerts_user_status ON transaction_monitoring_alerts(user_id, status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_monitoring_alerts_assigned ON transaction_monitoring_alerts(assigned_to, status) WHERE assigned_to IS NOT NULL;

-- ===== Audit logs optimization =====
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_action_date ON audit_logs(user_id, action, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource_date ON audit_logs(resource_type, resource_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_ip_date ON audit_logs(ip_address, created_at DESC) WHERE ip_address IS NOT NULL;

-- ===== Watchlist optimization =====
CREATE INDEX IF NOT EXISTS idx_watchlist_identifier ON watchlist_entries(identifier_type, identifier_value, status);
CREATE INDEX IF NOT EXISTS idx_watchlist_source_type ON watchlist_entries(list_source, list_type, effective_date);
CREATE INDEX IF NOT EXISTS idx_watchlist_matches_status ON watchlist_matches(status, match_score DESC);

-- ===== Partial indexes for active records only =====
CREATE INDEX IF NOT EXISTS idx_active_users_email ON users(email) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_active_projects_funding ON projects(funds_raised, total_value) WHERE status IN ('active', 'approved');
CREATE INDEX IF NOT EXISTS idx_pending_transactions ON transactions(created_at DESC) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_pending_kyc ON kyc_verifications(created_at DESC) WHERE verification_status = 'pending';

-- ===== Text search indexes =====
CREATE INDEX IF NOT EXISTS idx_projects_name_trgm ON projects USING gin(name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_projects_description_trgm ON projects USING gin(description gin_trgm_ops);

-- ===== Time bucketing indexes (immutable wrappers) =====
CREATE INDEX IF NOT EXISTS idx_transactions_monthly
  ON transactions(immutable_year(created_at), immutable_month(created_at), status);
CREATE INDEX IF NOT EXISTS idx_audit_logs_monthly
  ON audit_logs(immutable_year(created_at), immutable_month(created_at));

-- ===== Covering indexes =====
CREATE INDEX IF NOT EXISTS idx_transactions_user_summary
ON transactions(user_id, transaction_type)
INCLUDE (amount, fee, status, created_at)
WHERE status = 'completed';

CREATE INDEX IF NOT EXISTS idx_projects_listing
ON projects(status, project_type)
INCLUDE (name, total_value, funds_raised, minimum_investment, created_at)
WHERE status IN ('active', 'approved');

-- ===== Monitoring & maintenance functions =====
CREATE OR REPLACE FUNCTION get_table_sizes()
RETURNS TABLE (
    schema_name TEXT,
    table_name TEXT,
    row_count BIGINT,
    total_size TEXT,
    index_size TEXT,
    table_size TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        schemaname::TEXT,
        tablename::TEXT,
        n_tup_ins - n_tup_del as row_count,
        pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename))::TEXT as total_size,
        pg_size_pretty(pg_indexes_size(schemaname||'.'||tablename))::TEXT as index_size,
        pg_size_pretty(pg_relation_size(schemaname||'.'||tablename))::TEXT as table_size
    FROM pg_stat_user_tables
    ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_index_usage_stats()
RETURNS TABLE (
    schema_name TEXT,
    table_name TEXT,
    index_name TEXT,
    index_size TEXT,
    index_scans BIGINT,
    tuples_read BIGINT,
    tuples_fetched BIGINT,
    usage_ratio NUMERIC
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        schemaname::TEXT,
        tablename::TEXT,
        indexname::TEXT,
        pg_size_pretty(pg_relation_size(indexrelid))::TEXT,
        idx_scan,
        idx_tup_read,
        idx_tup_fetch,
        CASE
            WHEN idx_tup_read > 0 THEN round(idx_tup_fetch::numeric / idx_tup_read::numeric * 100, 2)
            ELSE 0
        END as usage_ratio
    FROM pg_stat_user_indexes
    ORDER BY idx_scan DESC;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_slow_queries(min_duration_ms INTEGER DEFAULT 1000)
RETURNS TABLE (
    query TEXT,
    calls BIGINT,
    total_time DOUBLE PRECISION,
    mean_time DOUBLE PRECISION,
    max_time DOUBLE PRECISION,
    stddev_time DOUBLE PRECISION
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        pg_stat_statements.query,
        pg_stat_statements.calls,
        pg_stat_statements.total_time,
        pg_stat_statements.mean_time,
        pg_stat_statements.max_time,
        pg_stat_statements.stddev_time
    FROM pg_stat_statements
    WHERE pg_stat_statements.mean_time > min_duration_ms
    ORDER BY pg_stat_statements.mean_time DESC
    LIMIT 50;
EXCEPTION
    WHEN undefined_table THEN
        RAISE NOTICE 'pg_stat_statements extension not available';
        RETURN;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION analyze_all_tables()
RETURNS void AS $$
DECLARE
    table_record RECORD;
BEGIN
    FOR table_record IN
        SELECT schemaname, tablename
        FROM pg_tables
        WHERE schemaname NOT IN ('information_schema', 'pg_catalog')
    LOOP
        EXECUTE 'ANALYZE ' || quote_ident(table_record.schemaname) || '.' || quote_ident(table_record.tablename);
    END LOOP;

    RAISE NOTICE 'All tables analyzed successfully';
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION vacuum_maintenance()
RETURNS void AS $$
BEGIN
    VACUUM ANALYZE;

    INSERT INTO audit_logs (
        user_id,
        action,
        resource_type,
        resource_id,
        new_values,
        created_at
    ) VALUES (
        NULL,
        'vacuum_maintenance',
        'database',
        NULL,
        jsonb_build_object('vacuumed_at', NOW(), 'type', 'full_vacuum_analyze'),
        NOW()
    );

    RAISE NOTICE 'Vacuum maintenance completed';
END;
$$ LANGUAGE plpgsql;

-- ===== Constraints (ALL marked NOT VALID to avoid blocking on legacy rows) =====
ALTER TABLE users ADD CONSTRAINT users_email_format CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$') NOT VALID;
ALTER TABLE users ADD CONSTRAINT users_phone_format CHECK (phone IS NULL OR phone ~* '^\+?[1-9]\d{1,14}$') NOT VALID;
ALTER TABLE projects ADD CONSTRAINT projects_value_positive CHECK (total_value > 0) NOT VALID;
ALTER TABLE projects ADD CONSTRAINT projects_min_investment_positive CHECK (minimum_investment > 0) NOT VALID;
ALTER TABLE projects ADD CONSTRAINT projects_max_investment_valid CHECK (maximum_investment IS NULL OR maximum_investment >= minimum_investment) NOT VALID;
ALTER TABLE projects ADD CONSTRAINT projects_funds_raised_valid CHECK (funds_raised >= 0 AND funds_raised <= total_value) NOT VALID;
ALTER TABLE transactions ADD CONSTRAINT transactions_amount_positive CHECK (amount > 0) NOT VALID;
ALTER TABLE transactions ADD CONSTRAINT transactions_fee_non_negative CHECK (fee >= 0) NOT VALID;
ALTER TABLE tokens ADD CONSTRAINT tokens_supply_positive CHECK (total_supply > 0) NOT VALID;
ALTER TABLE tokens ADD CONSTRAINT tokens_circulating_valid CHECK (circulating_supply >= 0 AND circulating_supply <= total_supply) NOT VALID;
ALTER TABLE tokens ADD CONSTRAINT tokens_price_positive CHECK (current_price > 0 AND initial_price > 0) NOT VALID;
ALTER TABLE kyc_verifications ADD CONSTRAINT kyc_score_range CHECK (verification_score IS NULL OR (verification_score >= 0 AND verification_score <= 100)) NOT VALID;

-- ===== Maintenance: unused index finder =====
CREATE OR REPLACE FUNCTION check_unused_indexes()
RETURNS TABLE (
    schema_name TEXT,
    table_name TEXT,
    index_name TEXT,
    index_size TEXT,
    index_scans BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        schemaname::TEXT,
        tablename::TEXT,
        indexname::TEXT,
        pg_size_pretty(pg_relation_size(indexrelid))::TEXT,
        idx_scan
    FROM pg_stat_user_indexes
    WHERE idx_scan < 100
        AND pg_relation_size(indexrelid) > 1024 * 1024
        AND indexname NOT LIKE '%_pkey'
        AND indexname NOT LIKE '%_key'
    ORDER BY pg_relation_size(indexrelid) DESC;
END;
$$ LANGUAGE plpgsql;

-- ===== Maintenance scheduler table + seed =====
CREATE TABLE IF NOT EXISTS maintenance_schedule (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    task_name VARCHAR(255) NOT NULL,
    task_type VARCHAR(100) NOT NULL,
    frequency_hours INTEGER NOT NULL,
    last_run TIMESTAMPTZ,
    next_run TIMESTAMPTZ NOT NULL,
    enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO maintenance_schedule (task_name, task_type, frequency_hours, next_run)
VALUES
('Refresh Analytics Views', 'refresh_views', 6, NOW() + INTERVAL '6 hours'),
('Update Table Statistics', 'analyze_tables', 24, NOW() + INTERVAL '24 hours'),
('Vacuum Maintenance', 'vacuum', 168, NOW() + INTERVAL '168 hours'),
('Check Index Usage', 'index_check', 72, NOW() + INTERVAL '72 hours')
ON CONFLICT DO NOTHING;

-- ===== Performance summary view =====
CREATE OR REPLACE VIEW performance_summary AS
SELECT 'Active Users' AS metric, COUNT(*) AS value, 'users' AS unit
FROM users WHERE status = 'active'
UNION ALL
SELECT 'Active Projects', COUNT(*), 'projects'
FROM projects WHERE status IN ('active', 'approved')
UNION ALL
SELECT 'Pending Transactions', COUNT(*), 'transactions'
FROM transactions WHERE status = 'pending'
UNION ALL
SELECT 'Pending KYC', COUNT(*), 'verifications'
FROM kyc_verifications WHERE verification_status = 'pending'
UNION ALL
SELECT 'Database Size', pg_database_size(current_database()), 'bytes'
UNION ALL
SELECT 'Total Tables', COUNT(*), 'tables'
FROM information_schema.tables
WHERE table_schema NOT IN ('information_schema', 'pg_catalog');

-- ===== Comments =====
COMMENT ON FUNCTION get_table_sizes() IS 'Returns table sizes and row counts for database monitoring';
COMMENT ON FUNCTION get_index_usage_stats() IS 'Returns index usage statistics for performance optimization';
COMMENT ON FUNCTION get_slow_queries(INTEGER) IS 'Returns slow queries above specified duration threshold';
COMMENT ON FUNCTION analyze_all_tables() IS 'Analyzes all user tables to update query planner statistics';
COMMENT ON FUNCTION vacuum_maintenance() IS 'Performs vacuum analyze on all tables';
COMMENT ON FUNCTION check_unused_indexes() IS 'Identifies potentially unused indexes that consume space';
COMMENT ON VIEW performance_summary IS 'High-level database performance metrics';

-- ===== Post-migration notes =====
-- To validate constraints after cleaning legacy rows, run:
--   ALTER TABLE users            VALIDATE CONSTRAINT users_email_format;
--   ALTER TABLE users            VALIDATE CONSTRAINT users_phone_format;
--   ALTER TABLE projects         VALIDATE CONSTRAINT projects_value_positive;
--   ALTER TABLE projects         VALIDATE CONSTRAINT projects_min_investment_positive;
--   ALTER TABLE projects         VALIDATE CONSTRAINT projects_max_investment_valid;
--   ALTER TABLE projects         VALIDATE CONSTRAINT projects_funds_raised_valid;
--   ALTER TABLE transactions     VALIDATE CONSTRAINT transactions_amount_positive;
--   ALTER TABLE transactions     VALIDATE CONSTRAINT transactions_fee_non_negative;
--   ALTER TABLE tokens           VALIDATE CONSTRAINT tokens_supply_positive;
--   ALTER TABLE tokens           VALIDATE CONSTRAINT tokens_circulating_valid;
--   ALTER TABLE tokens           VALIDATE CONSTRAINT tokens_price_positive;
--   ALTER TABLE kyc_verifications VALIDATE CONSTRAINT kyc_score_range;
