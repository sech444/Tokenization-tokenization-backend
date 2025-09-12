-- Analytics views and functions migration
-- Version: 003
-- Description: Create views, materialized views, and functions for analytics and reporting

-- User Analytics Views
CREATE VIEW user_activity_summary AS
SELECT
    u.id,
    u.email,
    u.role,
    u.status,
    u.created_at as registration_date,
    u.last_login,
    COUNT(DISTINCT t.id) as total_transactions,
    COALESCE(SUM(t.amount), 0) as total_investment_amount,
    COUNT(DISTINCT CASE WHEN t.transaction_type = 'investment' THEN t.project_id END) as projects_invested_in,
    kyc.verification_status as kyc_status,
    kyc.risk_level,
    cp.risk_rating,
    cp.investor_type
FROM users u
LEFT JOIN transactions t ON u.id = t.user_id AND t.status = 'completed'
LEFT JOIN kyc_verifications kyc ON u.id = kyc.user_id
LEFT JOIN compliance_profiles cp ON u.id = cp.user_id
GROUP BY u.id, u.email, u.role, u.status, u.created_at, u.last_login,
         kyc.verification_status, kyc.risk_level, cp.risk_rating, cp.investor_type;

-- Project Performance View
CREATE VIEW project_performance_summary AS
SELECT
    p.id,
    p.name,
    p.project_type,
    p.status,
    p.owner_id,
    p.total_value,
    p.funds_raised,
    p.investor_count,
    p.minimum_investment,
    p.maximum_investment,
    ROUND((p.funds_raised::DECIMAL / p.total_value::DECIMAL) * 100, 2) as funding_percentage,
    p.expected_return,
    p.investment_period_months,
    p.is_tokenized,
    p.created_at,
    COUNT(DISTINCT t.user_id) as actual_investor_count,
    AVG(t.amount) as average_investment_amount,
    MIN(t.created_at) as first_investment_date,
    MAX(t.created_at) as last_investment_date,
    CASE
        WHEN p.funds_raised >= p.total_value THEN 'Fully Funded'
        WHEN p.funds_raised >= p.total_value * 0.8 THEN 'Nearly Funded'
        WHEN p.funds_raised >= p.total_value * 0.5 THEN 'Half Funded'
        WHEN p.funds_raised > 0 THEN 'Partially Funded'
        ELSE 'Not Funded'
    END as funding_status
FROM projects p
LEFT JOIN transactions t ON p.id = t.project_id AND t.status = 'completed' AND t.transaction_type = 'investment'
GROUP BY p.id, p.name, p.project_type, p.status, p.owner_id, p.total_value,
         p.funds_raised, p.investor_count, p.minimum_investment, p.maximum_investment,
         p.expected_return, p.investment_period_months, p.is_tokenized, p.created_at;

-- Transaction Analytics View
CREATE VIEW transaction_analytics AS
SELECT
    t.id,
    t.user_id,
    t.project_id,
    t.token_id,
    t.transaction_type,
    t.amount,
    t.fee,
    t.status,
    t.created_at,
    u.role as user_role,
    u.status as user_status,
    p.project_type,
    p.name as project_name,
    EXTRACT(YEAR FROM t.created_at) as transaction_year,
    EXTRACT(MONTH FROM t.created_at) as transaction_month,
    EXTRACT(DOW FROM t.created_at) as day_of_week,
    EXTRACT(HOUR FROM t.created_at) as hour_of_day,
    DATE_TRUNC('day', t.created_at) as transaction_date,
    DATE_TRUNC('week', t.created_at) as transaction_week,
    DATE_TRUNC('month', t.created_at) as transaction_month_start
FROM transactions t
JOIN users u ON t.user_id = u.id
LEFT JOIN projects p ON t.project_id = p.id;

-- KYC Compliance Analytics View
CREATE VIEW kyc_compliance_analytics AS
SELECT
    u.id as user_id,
    u.email,
    u.role,
    u.created_at as user_registration_date,
    kyc.id as kyc_id,
    kyc.verification_status,
    kyc.risk_level,
    kyc.created_at as kyc_initiated_date,
    kyc.verification_date,
    kyc.documents_verified,
    kyc.identity_verified,
    kyc.address_verified,
    kyc.pep_check,
    kyc.sanctions_check,
    kyc.adverse_media_check,
    kyc.verification_score,
    cp.risk_rating,
    cp.investor_type,
    cp.accredited_investor,
    CASE
        WHEN kyc.verification_date IS NOT NULL
        THEN EXTRACT(DAYS FROM kyc.verification_date - kyc.created_at)
        ELSE EXTRACT(DAYS FROM NOW() - kyc.created_at)
    END as kyc_processing_days,
    COUNT(kd.id) as documents_uploaded,
    COUNT(CASE WHEN kd.verification_status = 'verified' THEN 1 END) as documents_verified_count,
    COUNT(aml.id) as aml_screenings_count,
    COUNT(CASE WHEN aml.screening_result = 'match' THEN 1 END) as aml_matches_count
FROM users u
LEFT JOIN kyc_verifications kyc ON u.id = kyc.user_id
LEFT JOIN compliance_profiles cp ON u.id = cp.user_id
LEFT JOIN kyc_documents kd ON kyc.id = kd.kyc_verification_id
LEFT JOIN aml_screenings aml ON kyc.id = aml.kyc_verification_id
GROUP BY u.id, u.email, u.role, u.created_at, kyc.id, kyc.verification_status,
         kyc.risk_level, kyc.created_at, kyc.verification_date, kyc.documents_verified,
         kyc.identity_verified, kyc.address_verified, kyc.pep_check, kyc.sanctions_check,
         kyc.adverse_media_check, kyc.verification_score, cp.risk_rating, cp.investor_type,
         cp.accredited_investor;

-- Token Performance View
CREATE VIEW token_performance_analytics AS
SELECT
    tk.id,
    tk.name,
    tk.symbol,
    tk.total_supply,
    tk.circulating_supply,
    tk.current_price,
    tk.initial_price,
    tk.contract_address,
    tk.status,
    tk.created_at,
    p.name as project_name,
    p.project_type,
    p.total_value as project_value,
    p.funds_raised as project_funds_raised,
    ROUND(((tk.current_price - tk.initial_price)::DECIMAL / tk.initial_price::DECIMAL) * 100, 2) as price_change_percentage,
    COUNT(DISTINCT t.user_id) as holder_count,
    SUM(CASE WHEN t.transaction_type = 'investment' THEN t.amount ELSE 0 END) as total_traded_volume,
    AVG(CASE WHEN t.transaction_type = 'investment' THEN t.amount END) as average_trade_size
FROM tokens tk
JOIN projects p ON tk.project_id = p.id
LEFT JOIN transactions t ON tk.id = t.token_id AND t.status = 'completed'
GROUP BY tk.id, tk.name, tk.symbol, tk.total_supply, tk.circulating_supply,
         tk.current_price, tk.initial_price, tk.contract_address, tk.status,
         tk.created_at, p.name, p.project_type, p.total_value, p.funds_raised;

-- Materialized Views for Performance
CREATE MATERIALIZED VIEW daily_platform_metrics AS
SELECT
    DATE(created_at) as metric_date,
    'user_registrations' as metric_type,
    COUNT(*) as metric_value,
    CURRENT_TIMESTAMP as last_updated
FROM users
GROUP BY DATE(created_at)

UNION ALL

SELECT
    DATE(created_at) as metric_date,
    'projects_created' as metric_type,
    COUNT(*) as metric_value,
    CURRENT_TIMESTAMP as last_updated
FROM projects
GROUP BY DATE(created_at)

UNION ALL

SELECT
    DATE(created_at) as metric_date,
    'transactions_volume' as metric_type,
    COALESCE(SUM(amount), 0) as metric_value,
    CURRENT_TIMESTAMP as last_updated
FROM transactions
WHERE status = 'completed'
GROUP BY DATE(created_at)

UNION ALL

SELECT
    DATE(created_at) as metric_date,
    'transactions_count' as metric_type,
    COUNT(*) as metric_value,
    CURRENT_TIMESTAMP as last_updated
FROM transactions
WHERE status = 'completed'
GROUP BY DATE(created_at);

-- Create unique index for materialized view
CREATE UNIQUE INDEX idx_daily_platform_metrics_unique
ON daily_platform_metrics(metric_date, metric_type);

-- Monthly aggregated metrics materialized view
CREATE MATERIALIZED VIEW monthly_platform_metrics AS
SELECT
    DATE_TRUNC('month', metric_date)::DATE as month_start,
    metric_type,
    SUM(metric_value) as total_value,
    AVG(metric_value) as average_daily_value,
    COUNT(*) as days_with_activity,
    CURRENT_TIMESTAMP as last_updated
FROM daily_platform_metrics
GROUP BY DATE_TRUNC('month', metric_date), metric_type;

-- Create unique index for monthly metrics
CREATE UNIQUE INDEX idx_monthly_platform_metrics_unique
ON monthly_platform_metrics(month_start, metric_type);

-- Analytics Functions

-- Function to calculate user retention rate
CREATE OR REPLACE FUNCTION calculate_user_retention(
    cohort_start_date DATE,
    cohort_end_date DATE,
    retention_period_days INTEGER
)
RETURNS TABLE (
    cohort_period TEXT,
    cohort_size BIGINT,
    retained_users BIGINT,
    retention_rate DECIMAL(5,2)
) AS $$
BEGIN
    RETURN QUERY
    WITH cohort_users AS (
        SELECT
            id,
            DATE_TRUNC('month', created_at) as cohort_month
        FROM users
        WHERE DATE(created_at) BETWEEN cohort_start_date AND cohort_end_date
    ),
    user_activity AS (
        SELECT
            u.id,
            u.cohort_month,
            MAX(t.created_at) as last_activity
        FROM cohort_users u
        LEFT JOIN transactions t ON u.id = t.user_id
        GROUP BY u.id, u.cohort_month
    )
    SELECT
        TO_CHAR(ua.cohort_month, 'YYYY-MM') as cohort_period,
        COUNT(*) as cohort_size,
        COUNT(CASE
            WHEN ua.last_activity >= (ua.cohort_month + INTERVAL '1 month' * retention_period_days / 30)
            THEN 1
        END) as retained_users,
        ROUND(
            (COUNT(CASE
                WHEN ua.last_activity >= (ua.cohort_month + INTERVAL '1 month' * retention_period_days / 30)
                THEN 1
            END)::DECIMAL / COUNT(*)::DECIMAL) * 100,
            2
        ) as retention_rate
    FROM user_activity ua
    GROUP BY ua.cohort_month
    ORDER BY ua.cohort_month;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate project funding velocity
CREATE OR REPLACE FUNCTION calculate_project_funding_velocity(project_uuid UUID)
RETURNS TABLE (
    project_id UUID,
    project_name VARCHAR(255),
    total_value BIGINT,
    current_funding BIGINT,
    funding_percentage DECIMAL(5,2),
    days_since_launch INTEGER,
    daily_funding_rate DECIMAL(15,2),
    estimated_days_to_completion INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        p.id,
        p.name,
        p.total_value,
        p.funds_raised,
        ROUND((p.funds_raised::DECIMAL / p.total_value::DECIMAL) * 100, 2) as funding_percentage,
        EXTRACT(DAYS FROM NOW() - p.created_at)::INTEGER as days_since_launch,
        CASE
            WHEN EXTRACT(DAYS FROM NOW() - p.created_at) > 0
            THEN p.funds_raised::DECIMAL / EXTRACT(DAYS FROM NOW() - p.created_at)::DECIMAL
            ELSE 0
        END as daily_funding_rate,
        CASE
            WHEN p.funds_raised > 0 AND EXTRACT(DAYS FROM NOW() - p.created_at) > 0
            THEN ((p.total_value - p.funds_raised)::DECIMAL /
                  (p.funds_raised::DECIMAL / EXTRACT(DAYS FROM NOW() - p.created_at)::DECIMAL))::INTEGER
            ELSE NULL
        END as estimated_days_to_completion
    FROM projects p
    WHERE p.id = project_uuid;
END;
$$ LANGUAGE plpgsql;

-- Function to get top investors by volume
CREATE OR REPLACE FUNCTION get_top_investors(limit_count INTEGER DEFAULT 10)
RETURNS TABLE (
    user_id UUID,
    email VARCHAR(255),
    total_invested BIGINT,
    number_of_investments BIGINT,
    average_investment DECIMAL(15,2),
    first_investment_date TIMESTAMP WITH TIME ZONE,
    last_investment_date TIMESTAMP WITH TIME ZONE,
    unique_projects_count BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.email,
        COALESCE(SUM(t.amount), 0) as total_invested,
        COUNT(t.id) as number_of_investments,
        COALESCE(AVG(t.amount), 0) as average_investment,
        MIN(t.created_at) as first_investment_date,
        MAX(t.created_at) as last_investment_date,
        COUNT(DISTINCT t.project_id) as unique_projects_count
    FROM users u
    LEFT JOIN transactions t ON u.id = t.user_id
        AND t.transaction_type = 'investment'
        AND t.status = 'completed'
    GROUP BY u.id, u.email
    HAVING COUNT(t.id) > 0
    ORDER BY total_invested DESC
    LIMIT limit_count;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate platform revenue metrics
CREATE OR REPLACE FUNCTION calculate_platform_revenue(
    start_date DATE,
    end_date DATE
)
RETURNS TABLE (
    period_start DATE,
    period_end DATE,
    total_transaction_volume BIGINT,
    total_fees_collected BIGINT,
    number_of_transactions BIGINT,
    average_transaction_size DECIMAL(15,2),
    average_fee_percentage DECIMAL(5,4)
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        start_date as period_start,
        end_date as period_end,
        COALESCE(SUM(t.amount), 0) as total_transaction_volume,
        COALESCE(SUM(t.fee), 0) as total_fees_collected,
        COUNT(t.id) as number_of_transactions,
        COALESCE(AVG(t.amount), 0) as average_transaction_size,
        CASE
            WHEN SUM(t.amount) > 0
            THEN (SUM(t.fee)::DECIMAL / SUM(t.amount)::DECIMAL) * 100
            ELSE 0
        END as average_fee_percentage
    FROM transactions t
    WHERE DATE(t.created_at) BETWEEN start_date AND end_date
        AND t.status = 'completed';
END;
$$ LANGUAGE plpgsql;

-- Function to refresh materialized views
CREATE OR REPLACE FUNCTION refresh_analytics_views()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY daily_platform_metrics;
    REFRESH MATERIALIZED VIEW CONCURRENTLY monthly_platform_metrics;

    -- Log the refresh
    INSERT INTO audit_logs (
        user_id,
        action,
        resource_type,
        resource_id,
        new_values,
        created_at
    ) VALUES (
        NULL,
        'refresh_analytics_views',
        'materialized_view',
        NULL,
        jsonb_build_object('refreshed_at', NOW()),
        NOW()
    );
END;
$$ LANGUAGE plpgsql;

-- Risk scoring function
CREATE OR REPLACE FUNCTION calculate_user_risk_score(user_uuid UUID)
RETURNS TABLE (
    user_id UUID,
    risk_score DECIMAL(8,2),
    risk_factors JSONB
) AS $$
DECLARE
    base_score DECIMAL(8,2) := 50.0;
    kyc_score DECIMAL(8,2) := 0.0;
    transaction_score DECIMAL(8,2) := 0.0;
    compliance_score DECIMAL(8,2) := 0.0;
    final_score DECIMAL(8,2);
    factors JSONB := '{}'::jsonb;
BEGIN
    -- KYC scoring
    SELECT
        CASE
            WHEN kyc.verification_status = 'approved' THEN -20.0
            WHEN kyc.verification_status = 'rejected' THEN 30.0
            WHEN kyc.verification_status = 'pending' THEN 10.0
            ELSE 15.0
        END +
        CASE
            WHEN kyc.risk_level = 'low' THEN -10.0
            WHEN kyc.risk_level = 'high' THEN 20.0
            WHEN kyc.risk_level = 'critical' THEN 40.0
            ELSE 0.0
        END
    INTO kyc_score
    FROM kyc_verifications kyc
    WHERE kyc.user_id = user_uuid
    ORDER BY kyc.created_at DESC
    LIMIT 1;

    kyc_score := COALESCE(kyc_score, 15.0);

    -- Transaction pattern scoring
    WITH transaction_stats AS (
        SELECT
            COUNT(*) as tx_count,
            AVG(amount) as avg_amount,
            STDDEV(amount) as amount_stddev,
            COUNT(DISTINCT project_id) as unique_projects
        FROM transactions
        WHERE user_id = user_uuid
            AND status = 'completed'
            AND created_at >= NOW() - INTERVAL '90 days'
    )
    SELECT
        CASE
            WHEN tx_count > 50 THEN 15.0
            WHEN tx_count > 20 THEN 5.0
            WHEN tx_count < 3 THEN -5.0
            ELSE 0.0
        END +
        CASE
            WHEN amount_stddev > avg_amount * 2 THEN 10.0
            ELSE 0.0
        END
    INTO transaction_score
    FROM transaction_stats;

    transaction_score := COALESCE(transaction_score, 0.0);

    -- Compliance scoring
    SELECT
        CASE
            WHEN COUNT(CASE WHEN screening_result = 'match' THEN 1 END) > 0 THEN 35.0
            WHEN COUNT(CASE WHEN screening_result = 'potential_match' THEN 1 END) > 0 THEN 15.0
            ELSE -5.0
        END
    INTO compliance_score
    FROM aml_screenings aml
    JOIN kyc_verifications kyc ON aml.kyc_verification_id = kyc.id
    WHERE kyc.user_id = user_uuid;

    compliance_score := COALESCE(compliance_score, 0.0);

    final_score := base_score + kyc_score + transaction_score + compliance_score;
    final_score := GREATEST(0.0, LEAST(100.0, final_score));

    factors := jsonb_build_object(
        'base_score', base_score,
        'kyc_adjustment', kyc_score,
        'transaction_adjustment', transaction_score,
        'compliance_adjustment', compliance_score
    );

    RETURN QUERY SELECT user_uuid, final_score, factors;
END;
$$ LANGUAGE plpgsql;

-- Create indexes for analytics performance
CREATE INDEX IF NOT EXISTS idx_transactions_created_at_status ON transactions(created_at, status);
CREATE INDEX IF NOT EXISTS idx_transactions_user_id_type ON transactions(user_id, transaction_type);
CREATE INDEX IF NOT EXISTS idx_projects_created_at_status ON projects(created_at, status);
CREATE INDEX IF NOT EXISTS idx_users_created_at_role ON users(created_at, role);
CREATE INDEX IF NOT EXISTS idx_kyc_verifications_created_at_status ON kyc_verifications(created_at, verification_status);

-- Function to generate analytics report
CREATE OR REPLACE FUNCTION generate_analytics_report(
    start_date DATE DEFAULT CURRENT_DATE - INTERVAL '30 days',
    end_date DATE DEFAULT CURRENT_DATE
)
RETURNS JSONB AS $$
DECLARE
    result JSONB;
BEGIN
    WITH platform_stats AS (
        SELECT
            COUNT(DISTINCT u.id) as total_users,
            COUNT(DISTINCT CASE WHEN u.created_at BETWEEN start_date AND end_date THEN u.id END) as new_users,
            COUNT(DISTINCT p.id) as total_projects,
            COUNT(DISTINCT CASE WHEN p.created_at BETWEEN start_date AND end_date THEN p.id END) as new_projects,
            COUNT(DISTINCT t.id) as total_transactions,
            COUNT(DISTINCT CASE WHEN t.created_at BETWEEN start_date AND end_date THEN t.id END) as period_transactions,
            COALESCE(SUM(CASE WHEN t.created_at BETWEEN start_date AND end_date AND t.status = 'completed' THEN t.amount END), 0) as period_volume,
            COALESCE(SUM(CASE WHEN t.created_at BETWEEN start_date AND end_date AND t.status = 'completed' THEN t.fee END), 0) as period_fees
        FROM users u
        CROSS JOIN projects p
        CROSS JOIN transactions t
    ),
    kyc_stats AS (
        SELECT
            COUNT(*) as total_kyc,
            COUNT(CASE WHEN verification_status = 'approved' THEN 1 END) as approved_kyc,
            COUNT(CASE WHEN verification_status = 'pending' THEN 1 END) as pending_kyc,
            COUNT(CASE WHEN created_at BETWEEN start_date AND end_date THEN 1 END) as period_kyc
        FROM kyc_verifications
    )
    SELECT jsonb_build_object(
        'period', jsonb_build_object(
            'start_date', start_date,
            'end_date', end_date
        ),
        'users', jsonb_build_object(
            'total', ps.total_users,
            'new_in_period', ps.new_users
        ),
        'projects', jsonb_build_object(
            'total', ps.total_projects,
            'new_in_period', ps.new_projects
        ),
        'transactions', jsonb_build_object(
            'total_count', ps.total_transactions,
            'period_count', ps.period_transactions,
            'period_volume', ps.period_volume,
            'period_fees', ps.period_fees
        ),
        'compliance', jsonb_build_object(
            'total_kyc', ks.total_kyc,
            'approved_kyc', ks.approved_kyc,
            'pending_kyc', ks.pending_kyc,
            'kyc_approval_rate',
            CASE WHEN ks.total_kyc > 0
                THEN ROUND((ks.approved_kyc::DECIMAL / ks.total_kyc::DECIMAL) * 100, 2)
                ELSE 0
            END
        ),
        'generated_at', CURRENT_TIMESTAMP
    ) INTO result
    FROM platform_stats ps
    CROSS JOIN kyc_stats ks;

    RETURN result;
END;
$$ LANGUAGE plpgsql;
