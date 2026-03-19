-- Compliance tables migration
-- Version: 002
-- Description: Create additional compliance and regulatory reporting tables

-- Create additional ENUM types for compliance
CREATE TYPE compliance_action_type AS ENUM (
    'kyc_initiated', 'kyc_approved', 'kyc_rejected', 'aml_screening_performed',
    'risk_rating_updated', 'compliance_flag_added', 'compliance_flag_removed',
    'document_uploaded', 'review_requested', 'manual_override'
);

CREATE TYPE risk_impact AS ENUM ('none', 'low', 'medium', 'high', 'critical');

CREATE TYPE report_type AS ENUM (
    'suspicious_activity', 'large_transactions', 'compliance', 'audit', 'regulatory'
);

CREATE TYPE submission_status AS ENUM (
    'draft', 'pending', 'submitted', 'acknowledged', 'rejected'
);

-- Compliance Audit Log table for detailed compliance tracking
CREATE TABLE compliance_audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action_type compliance_action_type NOT NULL,
    action_description TEXT NOT NULL,
    performed_by UUID NOT NULL REFERENCES users(id),
    ip_address INET,
    user_agent TEXT,
    metadata JSONB DEFAULT '{}',
    risk_impact risk_impact,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Regulatory Reporting table
CREATE TABLE regulatory_reports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    report_type report_type NOT NULL,
    reporting_period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    reporting_period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    jurisdiction VARCHAR(10) NOT NULL, -- Country/region code
    report_data JSONB NOT NULL DEFAULT '{}',
    file_path TEXT,
    submission_status submission_status NOT NULL DEFAULT 'draft',
    submission_date TIMESTAMP WITH TIME ZONE,
    regulatory_reference VARCHAR(255),
    generated_by UUID NOT NULL REFERENCES users(id),
    reviewed_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Suspicious Activity Reports table
CREATE TABLE suspicious_activity_reports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    transaction_id UUID REFERENCES transactions(id) ON DELETE SET NULL,
    report_type VARCHAR(50) NOT NULL,
    suspicion_level risk_level NOT NULL,
    description TEXT NOT NULL,
    indicators TEXT[] DEFAULT '{}',
    investigation_notes TEXT,
    reported_by UUID NOT NULL REFERENCES users(id),
    reviewed_by UUID REFERENCES users(id),
    regulatory_filing_required BOOLEAN DEFAULT FALSE,
    regulatory_filing_date TIMESTAMP WITH TIME ZONE,
    regulatory_reference VARCHAR(255),
    status VARCHAR(50) DEFAULT 'open',
    resolution_notes TEXT,
    resolved_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Transaction Monitoring Rules table
CREATE TABLE transaction_monitoring_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rule_name VARCHAR(255) NOT NULL,
    rule_type VARCHAR(100) NOT NULL,
    description TEXT,
    conditions JSONB NOT NULL DEFAULT '{}',
    thresholds JSONB NOT NULL DEFAULT '{}',
    actions JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN DEFAULT TRUE,
    severity risk_level DEFAULT 'medium',
    created_by UUID NOT NULL REFERENCES users(id),
    last_modified_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Transaction Monitoring Alerts table
CREATE TABLE transaction_monitoring_alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rule_id UUID NOT NULL REFERENCES transaction_monitoring_rules(id) ON DELETE CASCADE,
    transaction_id UUID REFERENCES transactions(id) ON DELETE SET NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    alert_type VARCHAR(100) NOT NULL,
    severity risk_level NOT NULL,
    status VARCHAR(50) DEFAULT 'open',
    description TEXT NOT NULL,
    alert_data JSONB DEFAULT '{}',
    assigned_to UUID REFERENCES users(id),
    reviewed_by UUID REFERENCES users(id),
    review_notes TEXT,
    false_positive BOOLEAN DEFAULT FALSE,
    escalated BOOLEAN DEFAULT FALSE,
    escalated_at TIMESTAMP WITH TIME ZONE,
    resolved_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Compliance Training Records table
CREATE TABLE compliance_training_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    training_module VARCHAR(255) NOT NULL,
    training_version VARCHAR(50) NOT NULL,
    completion_date TIMESTAMP WITH TIME ZONE NOT NULL,
    score DECIMAL(5,2),
    certificate_url TEXT,
    expiry_date TIMESTAMP WITH TIME ZONE,
    renewed_from UUID REFERENCES compliance_training_records(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Risk Assessment Templates table
CREATE TABLE risk_assessment_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    template_name VARCHAR(255) NOT NULL,
    template_version VARCHAR(50) NOT NULL,
    description TEXT,
    assessment_criteria JSONB NOT NULL DEFAULT '{}',
    scoring_matrix JSONB NOT NULL DEFAULT '{}',
    risk_thresholds JSONB NOT NULL DEFAULT '{}',
    applicable_jurisdictions TEXT[] DEFAULT '{}',
    applicable_investor_types investor_type[] DEFAULT '{}',
    created_by UUID NOT NULL REFERENCES users(id),
    approved_by UUID REFERENCES users(id),
    effective_date TIMESTAMP WITH TIME ZONE NOT NULL,
    expiry_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- User Risk Assessments table
CREATE TABLE user_risk_assessments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    template_id UUID NOT NULL REFERENCES risk_assessment_templates(id),
    assessment_date TIMESTAMP WITH TIME ZONE NOT NULL,
    assessor_id UUID NOT NULL REFERENCES users(id),
    risk_score DECIMAL(8,2) NOT NULL,
    risk_rating risk_rating NOT NULL,
    risk_factors JSONB DEFAULT '{}',
    mitigation_measures TEXT,
    next_review_date TIMESTAMP WITH TIME ZONE,
    approved_by UUID REFERENCES users(id),
    approval_date TIMESTAMP WITH TIME ZONE,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Compliance Policies table
CREATE TABLE compliance_policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    policy_name VARCHAR(255) NOT NULL,
    policy_type VARCHAR(100) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    policy_content TEXT NOT NULL,
    applicable_jurisdictions TEXT[] DEFAULT '{}',
    applicable_roles user_role[] DEFAULT '{}',
    effective_date TIMESTAMP WITH TIME ZONE NOT NULL,
    review_date TIMESTAMP WITH TIME ZONE,
    expiry_date TIMESTAMP WITH TIME ZONE,
    created_by UUID NOT NULL REFERENCES users(id),
    approved_by UUID REFERENCES users(id),
    approval_date TIMESTAMP WITH TIME ZONE,
    supersedes UUID REFERENCES compliance_policies(id),
    status VARCHAR(50) DEFAULT 'draft',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Policy Acknowledgments table
CREATE TABLE policy_acknowledgments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    policy_id UUID NOT NULL REFERENCES compliance_policies(id) ON DELETE CASCADE,
    acknowledged_date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ip_address INET,
    user_agent TEXT,
    digital_signature TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, policy_id)
);

-- Compliance Exceptions table
CREATE TABLE compliance_exceptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    transaction_id UUID REFERENCES transactions(id) ON DELETE SET NULL,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    exception_type VARCHAR(100) NOT NULL,
    rule_violated VARCHAR(255),
    business_justification TEXT NOT NULL,
    risk_assessment TEXT,
    temporary_exception BOOLEAN DEFAULT FALSE,
    expiry_date TIMESTAMP WITH TIME ZONE,
    requested_by UUID NOT NULL REFERENCES users(id),
    approved_by UUID REFERENCES users(id),
    approval_date TIMESTAMP WITH TIME ZONE,
    rejection_reason TEXT,
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Watchlist Entries table
CREATE TABLE watchlist_entries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entry_type VARCHAR(50) NOT NULL, -- 'individual', 'entity', 'address', 'document'
    identifier_type VARCHAR(50) NOT NULL, -- 'name', 'email', 'passport', 'company_name', etc.
    identifier_value TEXT NOT NULL,
    list_source VARCHAR(255) NOT NULL,
    list_type VARCHAR(100) NOT NULL,
    risk_level risk_level NOT NULL DEFAULT 'medium',
    description TEXT,
    effective_date TIMESTAMP WITH TIME ZONE NOT NULL,
    expiry_date TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}',
    added_by UUID NOT NULL REFERENCES users(id),
    reviewed_by UUID REFERENCES users(id),
    last_review_date TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Watchlist Matches table
CREATE TABLE watchlist_matches (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    watchlist_entry_id UUID NOT NULL REFERENCES watchlist_entries(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    transaction_id UUID REFERENCES transactions(id) ON DELETE SET NULL,
    match_score DECIMAL(5,2) NOT NULL,
    match_type VARCHAR(50) NOT NULL,
    match_details JSONB DEFAULT '{}',
    false_positive BOOLEAN DEFAULT FALSE,
    reviewed_by UUID REFERENCES users(id),
    review_date TIMESTAMP WITH TIME ZONE,
    review_notes TEXT,
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX idx_compliance_audit_logs_user_id ON compliance_audit_logs(user_id);
CREATE INDEX idx_compliance_audit_logs_action_type ON compliance_audit_logs(action_type);
CREATE INDEX idx_compliance_audit_logs_performed_by ON compliance_audit_logs(performed_by);
CREATE INDEX idx_compliance_audit_logs_created_at ON compliance_audit_logs(created_at);

CREATE INDEX idx_regulatory_reports_report_type ON regulatory_reports(report_type);
CREATE INDEX idx_regulatory_reports_jurisdiction ON regulatory_reports(jurisdiction);
CREATE INDEX idx_regulatory_reports_submission_status ON regulatory_reports(submission_status);
CREATE INDEX idx_regulatory_reports_period ON regulatory_reports(reporting_period_start, reporting_period_end);

CREATE INDEX idx_suspicious_activity_reports_user_id ON suspicious_activity_reports(user_id);
CREATE INDEX idx_suspicious_activity_reports_status ON suspicious_activity_reports(status);
CREATE INDEX idx_suspicious_activity_reports_suspicion_level ON suspicious_activity_reports(suspicion_level);

CREATE INDEX idx_transaction_monitoring_alerts_rule_id ON transaction_monitoring_alerts(rule_id);
CREATE INDEX idx_transaction_monitoring_alerts_user_id ON transaction_monitoring_alerts(user_id);
CREATE INDEX idx_transaction_monitoring_alerts_status ON transaction_monitoring_alerts(status);
CREATE INDEX idx_transaction_monitoring_alerts_severity ON transaction_monitoring_alerts(severity);

CREATE INDEX idx_compliance_training_records_user_id ON compliance_training_records(user_id);
CREATE INDEX idx_compliance_training_records_expiry_date ON compliance_training_records(expiry_date);

CREATE INDEX idx_user_risk_assessments_user_id ON user_risk_assessments(user_id);
CREATE INDEX idx_user_risk_assessments_risk_rating ON user_risk_assessments(risk_rating);
CREATE INDEX idx_user_risk_assessments_next_review_date ON user_risk_assessments(next_review_date);

CREATE INDEX idx_policy_acknowledgments_user_id ON policy_acknowledgments(user_id);
CREATE INDEX idx_policy_acknowledgments_policy_id ON policy_acknowledgments(policy_id);

CREATE INDEX idx_compliance_exceptions_user_id ON compliance_exceptions(user_id);
CREATE INDEX idx_compliance_exceptions_status ON compliance_exceptions(status);
CREATE INDEX idx_compliance_exceptions_expiry_date ON compliance_exceptions(expiry_date);

CREATE INDEX idx_watchlist_entries_identifier_type ON watchlist_entries(identifier_type);
CREATE INDEX idx_watchlist_entries_identifier_value ON watchlist_entries(identifier_value);
CREATE INDEX idx_watchlist_entries_list_source ON watchlist_entries(list_source);
CREATE INDEX idx_watchlist_entries_status ON watchlist_entries(status);

CREATE INDEX idx_watchlist_matches_watchlist_entry_id ON watchlist_matches(watchlist_entry_id);
CREATE INDEX idx_watchlist_matches_user_id ON watchlist_matches(user_id);
CREATE INDEX idx_watchlist_matches_status ON watchlist_matches(status);

-- Add triggers for automatic updated_at timestamps
CREATE TRIGGER update_regulatory_reports_updated_at BEFORE UPDATE ON regulatory_reports FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_suspicious_activity_reports_updated_at BEFORE UPDATE ON suspicious_activity_reports FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_transaction_monitoring_rules_updated_at BEFORE UPDATE ON transaction_monitoring_rules FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_transaction_monitoring_alerts_updated_at BEFORE UPDATE ON transaction_monitoring_alerts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_risk_assessment_templates_updated_at BEFORE UPDATE ON risk_assessment_templates FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_risk_assessments_updated_at BEFORE UPDATE ON user_risk_assessments FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_compliance_policies_updated_at BEFORE UPDATE ON compliance_policies FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_compliance_exceptions_updated_at BEFORE UPDATE ON compliance_exceptions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_watchlist_entries_updated_at BEFORE UPDATE ON watchlist_entries FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_watchlist_matches_updated_at BEFORE UPDATE ON watchlist_matches FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default transaction monitoring rules
INSERT INTO transaction_monitoring_rules (
    id,
    rule_name,
    rule_type,
    description,
    conditions,
    thresholds,
    actions,
    severity,
    created_by
) VALUES
(
    uuid_generate_v4(),
    'Large Transaction Alert',
    'transaction_amount',
    'Alert for transactions above threshold',
    '{"transaction_type": "investment"}',
    '{"amount": 1000000}', -- $10,000 in cents
    '{"create_alert": true, "require_review": true}',
    'medium',
    (SELECT id FROM users WHERE role = 'admin' LIMIT 1)
),
(
    uuid_generate_v4(),
    'Rapid Transaction Velocity',
    'transaction_velocity',
    'Alert for multiple transactions in short period',
    '{"time_window_hours": 1}',
    '{"transaction_count": 5, "total_amount": 500000}', -- $5,000 in cents
    '{"create_alert": true, "flag_for_review": true}',
    'high',
    (SELECT id FROM users WHERE role = 'admin' LIMIT 1)
),
(
    uuid_generate_v4(),
    'Cross-Border Transaction',
    'geographic',
    'Alert for transactions from high-risk jurisdictions',
    '{"check_user_location": true}',
    '{"high_risk_countries": ["XX", "YY"]}',
    '{"create_alert": true, "enhanced_screening": true}',
    'high',
    (SELECT id FROM users WHERE role = 'admin' LIMIT 1)
);

-- Insert default risk assessment template
INSERT INTO risk_assessment_templates (
    id,
    template_name,
    template_version,
    description,
    assessment_criteria,
    scoring_matrix,
    risk_thresholds,
    created_by,
    effective_date
) VALUES (
    uuid_generate_v4(),
    'Standard Individual Risk Assessment',
    '1.0',
    'Standard risk assessment template for individual investors',
    '{
        "kyc_status": {"weight": 30, "criteria": ["approved", "pending", "rejected"]},
        "geographic_risk": {"weight": 20, "criteria": ["low", "medium", "high"]},
        "transaction_history": {"weight": 25, "criteria": ["clean", "suspicious", "flagged"]},
        "aml_screening": {"weight": 25, "criteria": ["clear", "potential_match", "match"]}
    }',
    '{
        "kyc_status": {"approved": 0, "pending": 50, "rejected": 100},
        "geographic_risk": {"low": 0, "medium": 30, "high": 70},
        "transaction_history": {"clean": 0, "suspicious": 40, "flagged": 80},
        "aml_screening": {"clear": 0, "potential_match": 60, "match": 100}
    }',
    '{
        "very_low": {"min": 0, "max": 15},
        "low": {"min": 16, "max": 30},
        "medium": {"min": 31, "max": 50},
        "high": {"min": 51, "max": 75},
        "very_high": {"min": 76, "max": 95},
        "prohibited": {"min": 96, "max": 100}
    }',
    (SELECT id FROM users WHERE role = 'admin' LIMIT 1),
    NOW()
);
