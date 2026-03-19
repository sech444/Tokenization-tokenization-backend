-- Initial schema migration for tokenization platform
-- Version: 001
-- Description: Create core tables for users, projects, tokens, transactions, and compliance

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create custom ENUM types
CREATE TYPE user_role AS ENUM ('admin', 'user', 'investor', 'project_manager', 'compliance_officer', 'moderator', 'developer');
CREATE TYPE user_status AS ENUM ('active', 'inactive', 'suspended', 'pending_verification');
CREATE TYPE project_type AS ENUM ('residential', 'commercial', 'industrial', 'mixed_use', 'land', 'hospitality');
CREATE TYPE project_status AS ENUM ('draft', 'submitted', 'pending_approval', 'approved', 'active', 'funded', 'completed', 'cancelled', 'rejected', 'closed');
CREATE TYPE token_status AS ENUM ('pending', 'active', 'paused', 'cancelled', 'completed');
CREATE TYPE transaction_type AS ENUM ('investment', 'withdrawal', 'transfer', 'dividend', 'fee');
CREATE TYPE transaction_status AS ENUM ('pending', 'processing', 'completed', 'failed', 'cancelled');
CREATE TYPE verification_status AS ENUM ('pending', 'in_progress', 'approved', 'rejected', 'expired', 'requires_review');
CREATE TYPE risk_level AS ENUM ('low', 'medium', 'high', 'critical');
CREATE TYPE risk_rating AS ENUM ('very_low', 'low', 'medium', 'high', 'very_high', 'prohibited');
CREATE TYPE investor_type AS ENUM ('retail', 'accredited', 'qualified_institutional', 'institutional', 'foreign');
CREATE TYPE document_type AS ENUM ('passport', 'driver_license', 'national_id', 'utility_bill', 'bank_statement', 'proof_of_income', 'business_registration', 'tax_document', 'other');
CREATE TYPE document_verification_status AS ENUM ('pending', 'processing', 'verified', 'failed', 'requires_review');
CREATE TYPE aml_screening_type AS ENUM ('pep_check', 'sanctions_check', 'adverse_media_check', 'watchlist', 'enhanced');
CREATE TYPE aml_screening_result AS ENUM ('clear', 'potential_match', 'match', 'requires_review', 'error');
CREATE TYPE aml_match_type AS ENUM ('politically_exposed_person', 'sanctioned_entity', 'adverse_media', 'watchlist', 'relative_or_associate');

-- -- Initial schema migration for tokenization platform
-- -- Version: 001
-- -- Description: Create core tables for users, projects, tokens, transactions, and compliance


-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    phone VARCHAR(20),
    date_of_birth DATE,
    nationality VARCHAR(3), -- ISO country code
    address JSONB,
    username VARCHAR(255),
    wallet_address VARCHAR(42),
    role user_role NOT NULL DEFAULT 'user',
    status user_status NOT NULL DEFAULT 'pending_verification',
    email_verified BOOLEAN DEFAULT FALSE,
    phone_verified BOOLEAN DEFAULT FALSE,
    two_factor_enabled BOOLEAN DEFAULT FALSE,
    two_factor_secret VARCHAR(100),
    is_active BOOLEAN DEFAULT TRUE,
    last_login TIMESTAMP WITH TIME ZONE,
    login_attempts INTEGER DEFAULT 0,
    locked_until TIMESTAMP WITH TIME ZONE,
    reset_token VARCHAR(255),
    reset_token_expires TIMESTAMP WITH TIME ZONE,
    verification_token VARCHAR(255),
    verification_token_expires TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Projects table
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    project_type project_type NOT NULL,
    status project_status NOT NULL DEFAULT 'draft',
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    location VARCHAR(255),
    property_address TEXT,
    total_value BIGINT NOT NULL, -- in cents
    minimum_investment BIGINT NOT NULL, -- in cents
    maximum_investment BIGINT, -- in cents
    funds_raised BIGINT DEFAULT 0, -- in cents
    investor_count INTEGER DEFAULT 0,
    expected_return DECIMAL(5,2), -- percentage
    investment_period_months INTEGER NOT NULL,
    property_details JSONB DEFAULT '{}',
    legal_documents TEXT[] DEFAULT '{}',
    images TEXT[] DEFAULT '{}',
    is_tokenized BOOLEAN DEFAULT FALSE,
    token_contract_address VARCHAR(42), -- Ethereum address
    compliance_verified BOOLEAN DEFAULT FALSE,
    kyc_required BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);


-- Tokens table
CREATE TABLE tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    total_supply BIGINT NOT NULL,
    description TEXT,
    circulating_supply BIGINT DEFAULT 0,
    current_price BIGINT NOT NULL, -- in cents
    initial_price BIGINT NOT NULL, -- in cents
    contract_address VARCHAR(42) NOT NULL UNIQUE, -- Ethereum address
    decimals INTEGER DEFAULT 18,
    status token_status NOT NULL DEFAULT 'pending',
    metadata_uri TEXT,
    compliance_rules JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Transactions table
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    token_id UUID REFERENCES tokens(id) ON DELETE SET NULL,
    transaction_type transaction_type NOT NULL,
    amount BIGINT NOT NULL, -- in cents
    fee BIGINT DEFAULT 0, -- in cents
    status transaction_status NOT NULL DEFAULT 'pending',
    payment_method VARCHAR(50),
    payment_reference VARCHAR(255),
    blockchain_tx_hash VARCHAR(66), -- Ethereum transaction hash
    blockchain_confirmations INTEGER DEFAULT 0,
    description TEXT,
    metadata JSONB DEFAULT '{}',
    processed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- KYC Verifications table
CREATE TABLE kyc_verifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    verification_status verification_status NOT NULL DEFAULT 'pending',
    risk_level risk_level NOT NULL DEFAULT 'medium',
    verification_provider VARCHAR(100) NOT NULL DEFAULT 'internal',
    provider_reference_id VARCHAR(255),
    documents_verified BOOLEAN DEFAULT FALSE,
    identity_verified BOOLEAN DEFAULT FALSE,
    address_verified BOOLEAN DEFAULT FALSE,
    phone_verified BOOLEAN DEFAULT FALSE,
    email_verified BOOLEAN DEFAULT FALSE,
    pep_check BOOLEAN DEFAULT FALSE,
    sanctions_check BOOLEAN DEFAULT FALSE,
    adverse_media_check BOOLEAN DEFAULT FALSE,
    verification_score DECIMAL(5,2),
    verification_date TIMESTAMP WITH TIME ZONE,
    expiry_date TIMESTAMP WITH TIME ZONE,
    notes TEXT,
    rejection_reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- KYC Documents table
CREATE TABLE kyc_documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    kyc_verification_id UUID NOT NULL REFERENCES kyc_verifications(id) ON DELETE CASCADE,
    document_type document_type NOT NULL,
    document_number VARCHAR(100),
    issuing_country VARCHAR(3), -- ISO country code
    issuing_authority VARCHAR(255),
    issue_date DATE,
    expiry_date DATE,
    file_path TEXT NOT NULL,
    file_hash VARCHAR(64) NOT NULL,
    verification_status document_verification_status NOT NULL DEFAULT 'pending',
    extracted_data JSONB DEFAULT '{}',
    confidence_score DECIMAL(5,2),
    verification_notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- AML Screenings table
CREATE TABLE aml_screenings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    kyc_verification_id UUID NOT NULL REFERENCES kyc_verifications(id) ON DELETE CASCADE,
    screening_provider VARCHAR(100) NOT NULL,
    screening_reference_id VARCHAR(255) NOT NULL,
    screening_type aml_screening_type NOT NULL,
    screening_result aml_screening_result NOT NULL DEFAULT 'clear',
    risk_score DECIMAL(5,2),
    matches_found INTEGER DEFAULT 0,
    screening_data JSONB DEFAULT '{}',
    reviewed_by UUID REFERENCES users(id),
    review_notes TEXT,
    false_positive BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- AML Matches table
CREATE TABLE aml_matches (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    aml_screening_id UUID NOT NULL REFERENCES aml_screenings(id) ON DELETE CASCADE,
    match_type aml_match_type NOT NULL,
    entity_name VARCHAR(255) NOT NULL,
    entity_type VARCHAR(100),
    match_score DECIMAL(5,2) NOT NULL,
    list_source VARCHAR(255) NOT NULL,
    list_type VARCHAR(100) NOT NULL,
    description TEXT,
    countries TEXT[],
    aliases TEXT[],
    birth_date DATE,
    nationality VARCHAR(3), -- ISO country code
    additional_info JSONB DEFAULT '{}',
    reviewed BOOLEAN DEFAULT FALSE,
    false_positive BOOLEAN DEFAULT FALSE,
    review_notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Compliance Profiles table
CREATE TABLE compliance_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    risk_rating risk_rating NOT NULL DEFAULT 'medium',
    investor_type investor_type NOT NULL DEFAULT 'retail',
    accredited_investor BOOLEAN DEFAULT FALSE,
    accreditation_verified BOOLEAN DEFAULT FALSE,
    accreditation_documents UUID[],
    investment_limit BIGINT, -- in cents
    geographic_restrictions TEXT[],
    compliance_flags TEXT[] DEFAULT '{}',
    last_review_date TIMESTAMP WITH TIME ZONE,
    next_review_date TIMESTAMP WITH TIME ZONE,
    compliance_officer_id UUID REFERENCES users(id),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- User Sessions table (for tracking active sessions)
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token VARCHAR(255) NOT NULL UNIQUE,
    refresh_token VARCHAR(255) NOT NULL UNIQUE,
    device_info JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_accessed TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Audit Log table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for better performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_created_at ON users(created_at);

CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_projects_owner_id ON projects(owner_id);
CREATE INDEX idx_projects_project_type ON projects(project_type);
CREATE INDEX idx_projects_created_at ON projects(created_at);
CREATE INDEX idx_projects_is_tokenized ON projects(is_tokenized);

CREATE INDEX idx_tokens_project_id ON tokens(project_id);
CREATE INDEX idx_tokens_contract_address ON tokens(contract_address);
CREATE INDEX idx_tokens_status ON tokens(status);
CREATE INDEX idx_tokens_created_at ON tokens(created_at);

CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transactions_project_id ON transactions(project_id);
CREATE INDEX idx_transactions_token_id ON transactions(token_id);
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_transaction_type ON transactions(transaction_type);
CREATE INDEX idx_transactions_created_at ON transactions(created_at);
CREATE INDEX idx_transactions_blockchain_tx_hash ON transactions(blockchain_tx_hash);

CREATE INDEX idx_kyc_verifications_user_id ON kyc_verifications(user_id);
CREATE INDEX idx_kyc_verifications_status ON kyc_verifications(verification_status);
CREATE INDEX idx_kyc_verifications_risk_level ON kyc_verifications(risk_level);
CREATE INDEX idx_kyc_verifications_created_at ON kyc_verifications(created_at);

CREATE INDEX idx_kyc_documents_kyc_verification_id ON kyc_documents(kyc_verification_id);
CREATE INDEX idx_kyc_documents_document_type ON kyc_documents(document_type);
CREATE INDEX idx_kyc_documents_verification_status ON kyc_documents(verification_status);

CREATE INDEX idx_aml_screenings_kyc_verification_id ON aml_screenings(kyc_verification_id);
CREATE INDEX idx_aml_screenings_screening_type ON aml_screenings(screening_type);
CREATE INDEX idx_aml_screenings_screening_result ON aml_screenings(screening_result);

CREATE INDEX idx_aml_matches_aml_screening_id ON aml_matches(aml_screening_id);
CREATE INDEX idx_aml_matches_match_type ON aml_matches(match_type);
CREATE INDEX idx_aml_matches_reviewed ON aml_matches(reviewed);

CREATE INDEX idx_compliance_profiles_user_id ON compliance_profiles(user_id);
CREATE INDEX idx_compliance_profiles_risk_rating ON compliance_profiles(risk_rating);
CREATE INDEX idx_compliance_profiles_investor_type ON compliance_profiles(investor_type);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_session_token ON user_sessions(session_token);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);

CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_resource_type ON audit_logs(resource_type);
CREATE INDEX idx_audit_logs_resource_id ON audit_logs(resource_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- Functions for updating timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for automatic updated_at timestamps
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_projects_updated_at BEFORE UPDATE ON projects FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_tokens_updated_at BEFORE UPDATE ON tokens FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_transactions_updated_at BEFORE UPDATE ON transactions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_kyc_verifications_updated_at BEFORE UPDATE ON kyc_verifications FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_kyc_documents_updated_at BEFORE UPDATE ON kyc_documents FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_aml_screenings_updated_at BEFORE UPDATE ON aml_screenings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_aml_matches_updated_at BEFORE UPDATE ON aml_matches FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_compliance_profiles_updated_at BEFORE UPDATE ON compliance_profiles FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default admin user (password should be changed on first login)
-- Password hash for 'admin123' (should be changed in production)
INSERT INTO users (
    id,
    email,
    password_hash,
    first_name,
    last_name,
    role,
    status,
    email_verified
) VALUES (
    uuid_generate_v4(),
    'admin@tokenization.com',
    '$2b$12$BjnGBVDBxPQ5MmvP4FSzBOR8XWOGEbb/HkLgICTPon1YSEmkSrVtu', -- admin123
    'System',
    'Administrator',
    'admin',
    'active',
    true
);
