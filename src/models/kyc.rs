// tokenization-backend/src/models/kyc.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::str::FromStr;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KycVerification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub verification_status: VerificationStatus,
    pub risk_level: RiskLevel,
    pub verification_provider: String,
    pub provider_reference_id: Option<String>,
    pub documents_verified: bool,
    pub identity_verified: bool,
    pub address_verified: bool,
    pub phone_verified: bool,
    pub email_verified: bool,
    pub pep_check: bool,
    pub sanctions_check: bool,
    pub adverse_media_check: bool,
    pub verification_score: Option<f32>,
    pub verification_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub rejection_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KycDocument {
    pub id: Uuid,
    pub kyc_verification_id: Uuid,
    pub document_type: DocumentType,
    pub document_number: Option<String>,
    pub issuing_country: Option<String>,
    pub issuing_authority: Option<String>,
    pub issue_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub file_path: String,
    pub file_hash: String,
    pub verification_status: DocumentVerificationStatus,
    pub extracted_data: Option<serde_json::Value>,
    pub confidence_score: Option<f32>,
    pub verification_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AmlScreening {
    pub id: Uuid,
    pub kyc_verification_id: Uuid,
    pub screening_provider: String,
    pub screening_reference_id: String,
    pub screening_type: AmlScreeningType,
    pub screening_result: AmlScreeningResult,
    pub risk_score: Option<f32>,
    pub matches_found: i32,
    pub screening_data: serde_json::Value,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
    pub false_positive: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AmlMatch {
    pub id: Uuid,
    pub aml_screening_id: Uuid,
    pub match_type: AmlMatchType,
    pub entity_name: String,
    pub entity_type: Option<String>,
    pub match_score: f32,
    pub list_source: String,
    pub list_type: String,
    pub description: Option<String>,
    pub countries: Option<Vec<String>>,
    pub aliases: Option<Vec<String>>,
    pub birth_date: Option<DateTime<Utc>>,
    pub nationality: Option<String>,
    pub additional_info: Option<serde_json::Value>,
    pub reviewed: bool,
    pub false_positive: bool,
    pub review_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub risk_rating: RiskRating,
    pub investor_type: InvestorType,
    pub accredited_investor: bool,
    pub accreditation_verified: bool,
    pub accreditation_documents: Option<Vec<Uuid>>,
    pub investment_limit: Option<i64>,
    pub geographic_restrictions: Option<Vec<String>>,
    pub compliance_flags: Vec<String>,
    pub last_review_date: Option<DateTime<Utc>>,
    pub next_review_date: Option<DateTime<Utc>>,
    pub compliance_officer_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceAuditLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action_type: ComplianceActionType,
    pub action_description: String,
    pub performed_by: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub risk_impact: Option<RiskImpact>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RegulatoryReporting {
    pub id: Uuid,
    pub report_type: ReportType,
    pub reporting_period_start: DateTime<Utc>,
    pub reporting_period_end: DateTime<Utc>,
    pub jurisdiction: String,
    pub report_data: serde_json::Value,
    pub file_path: Option<String>,
    pub submission_status: SubmissionStatus,
    pub submission_date: Option<DateTime<Utc>>,
    pub regulatory_reference: Option<String>,
    pub generated_by: Uuid,
    pub reviewed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Enums
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "verification_status", rename_all = "lowercase")]
pub enum VerificationStatus {
    Pending,
    InProgress,
    Approved,
    Rejected,
    Expired,
    RequiresReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "risk_level", rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_type", rename_all = "lowercase")]
pub enum DocumentType {
    Passport,
    DriverLicense,
    NationalId,
    UtilityBill,
    BankStatement,
    ProofOfIncome,
    BusinessRegistration,
    TaxDocument,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_verification_status", rename_all = "lowercase")]
pub enum DocumentVerificationStatus {
    Pending,
    Processing,
    Verified,
    Failed,
    RequiresReview,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "aml_screening_type", rename_all = "lowercase")]
pub enum AmlScreeningType {
    PepCheck,
    SanctionsCheck,
    AdverseMediaCheck,
    Watchlist,
    Enhanced,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "aml_screening_result", rename_all = "lowercase")]
pub enum AmlScreeningResult {
    Clear,
    PotentialMatch,
    Match,
    RequiresReview,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "aml_match_type", rename_all = "lowercase")]
pub enum AmlMatchType {
    PoliticallyExposedPerson,
    SanctionedEntity,
    AdverseMedia,
    Watchlist,
    RelativeOrAssociate,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "risk_rating", rename_all = "lowercase")]
pub enum RiskRating {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
    Prohibited,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "investor_type", rename_all = "lowercase")]
pub enum InvestorType {
    Retail,
    Accredited,
    QualifiedInstitutional,
    Institutional,
    Foreign,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "compliance_action_type", rename_all = "lowercase")]
pub enum ComplianceActionType {
    KycInitiated,
    KycApproved,
    KycRejected,
    AmlScreeningPerformed,
    RiskRatingUpdated,
    ComplianceFlagAdded,
    ComplianceFlagRemoved,
    DocumentUploaded,
    ReviewRequested,
    ManualOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "risk_impact", rename_all = "lowercase")]
pub enum RiskImpact {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "report_type", rename_all = "lowercase")]
pub enum ReportType {
    SuspiciousActivity,
    LargeTransactions,
    Compliance,
    Audit,
    Regulatory,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "submission_status", rename_all = "lowercase")]
pub enum SubmissionStatus {
    Draft,
    Pending,
    Submitted,
    Acknowledged,
    Rejected,
}

// Request/Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct InitiateKycRequest {
    pub user_id: Uuid,
    pub verification_provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KycDocumentUpload {
    pub document_type: DocumentType,
    pub file_name: String,
    pub file_content: Vec<u8>,
    pub document_number: Option<String>,
    pub issuing_country: Option<String>,
    pub expiry_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateKycStatusRequest {
    pub verification_status: VerificationStatus,
    pub rejection_reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AmlScreeningRequest {
    pub kyc_verification_id: Uuid,
    pub screening_types: Vec<AmlScreeningType>,
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReviewRequest {
    pub user_id: Uuid,
    pub risk_rating: Option<RiskRating>,
    pub compliance_flags: Option<Vec<String>>,
    pub notes: Option<String>,
}

// Implementation blocks
impl KycVerification {
    pub fn new(user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            verification_status: VerificationStatus::Pending,
            risk_level: RiskLevel::Medium,
            verification_provider: "internal".to_string(),
            provider_reference_id: None,
            documents_verified: false,
            identity_verified: false,
            address_verified: false,
            phone_verified: false,
            email_verified: false,
            pep_check: false,
            sanctions_check: false,
            adverse_media_check: false,
            verification_score: None,
            verification_date: None,
            expiry_date: None,
            notes: None,
            rejection_reason: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_verified(&self) -> bool {
        matches!(self.verification_status, VerificationStatus::Approved)
            && self.documents_verified
            && self.identity_verified
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiry_date) = self.expiry_date {
            Utc::now() > expiry_date
        } else {
            false
        }
    }

    pub fn calculate_risk_score(&self) -> f32 {
        let mut score = 0.0;

        if self.documents_verified {
            score += 20.0;
        }
        if self.identity_verified {
            score += 25.0;
        }
        if self.address_verified {
            score += 15.0;
        }
        if self.phone_verified {
            score += 10.0;
        }
        if self.email_verified {
            score += 10.0;
        }
        if !self.pep_check {
            score += 10.0;
        }
        if !self.sanctions_check {
            score += 10.0;
        }

        score
    }
}

impl ComplianceProfile {
    pub fn new(user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            risk_rating: RiskRating::Medium,
            investor_type: InvestorType::Retail,
            accredited_investor: false,
            accreditation_verified: false,
            accreditation_documents: None,
            investment_limit: None,
            geographic_restrictions: None,
            compliance_flags: Vec::new(),
            last_review_date: None,
            next_review_date: None,
            compliance_officer_id: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_high_risk(&self) -> bool {
        matches!(
            self.risk_rating,
            RiskRating::High | RiskRating::VeryHigh | RiskRating::Prohibited
        )
    }

    pub fn can_invest(&self, amount: i64) -> bool {
        if matches!(self.risk_rating, RiskRating::Prohibited) {
            return false;
        }

        if let Some(limit) = self.investment_limit {
            amount <= limit
        } else {
            true
        }
    }

    pub fn requires_enhanced_due_diligence(&self) -> bool {
        self.is_high_risk() || self.compliance_flags.contains(&"enhanced_dd".to_string())
    }
}

impl AmlScreening {
    pub fn new(kyc_verification_id: Uuid, screening_type: AmlScreeningType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            kyc_verification_id,
            screening_provider: "internal".to_string(),
            screening_reference_id: Uuid::new_v4().to_string(),
            screening_type,
            screening_result: AmlScreeningResult::Clear,
            risk_score: None,
            matches_found: 0,
            screening_data: serde_json::Value::Null,
            reviewed_by: None,
            review_notes: None,
            false_positive: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn has_matches(&self) -> bool {
        self.matches_found > 0
    }

    pub fn requires_review(&self) -> bool {
        matches!(
            self.screening_result,
            AmlScreeningResult::PotentialMatch
                | AmlScreeningResult::Match
                | AmlScreeningResult::RequiresReview
        )
    }
}


// Add these implementations for VerificationStatus
impl Display for VerificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            VerificationStatus::Pending => "pending",
            VerificationStatus::InProgress => "in_progress",
            VerificationStatus::Approved => "approved",
            VerificationStatus::Rejected => "rejected",
            VerificationStatus::Expired => "expired",
            VerificationStatus::RequiresReview => "requires_review",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for VerificationStatus {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(VerificationStatus::Pending),
            "in_progress" => Ok(VerificationStatus::InProgress),
            "approved" => Ok(VerificationStatus::Approved),
            "rejected" => Ok(VerificationStatus::Rejected),
            "expired" => Ok(VerificationStatus::Expired),
            _ => Err(format!("Invalid verification status: {}", s)),
        }
    }
}

// Add these implementations for RiskLevel
impl Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for RiskLevel {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(RiskLevel::Low),
            "medium" => Ok(RiskLevel::Medium),
            "high" => Ok(RiskLevel::High),
            _ => Err(format!("Invalid risk level: {}", s)),
        }
    }
}

// Add these implementations for DocumentType
impl Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DocumentType::Passport => "passport",
            DocumentType::DriverLicense => "drivers_license",
            DocumentType::NationalId => "national_id",
            DocumentType::UtilityBill => "utility_bill",
            DocumentType::BankStatement => "bank_statement",
            DocumentType::Other => "other",
            DocumentType::ProofOfIncome => "proof_of_income",
            DocumentType::BusinessRegistration => "business_registration",
            DocumentType::TaxDocument => "tax_document",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for DocumentType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "passport" => Ok(DocumentType::Passport),
            "drivers_license" => Ok(DocumentType::DriverLicense),
            "national_id" => Ok(DocumentType::NationalId),
            "utility_bill" => Ok(DocumentType::UtilityBill),
            "bank_statement" => Ok(DocumentType::BankStatement),
            "other" => Ok(DocumentType::Other),
            _ => Err(format!("Invalid document type: {}", s)),
        }
    }
}

// Add these implementations for DocumentVerificationStatus
impl Display for DocumentVerificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DocumentVerificationStatus::Pending => "pending",
            DocumentVerificationStatus::Processing => "processing",
            DocumentVerificationStatus::Approved => "approved",
            DocumentVerificationStatus::Rejected => "rejected",
            DocumentVerificationStatus::Failed => "failed",
            DocumentVerificationStatus::RequiresReview => "requires_review",
            DocumentVerificationStatus::Verified => "verified",
            
        };
        write!(f, "{}", s)
    }
}

impl FromStr for DocumentVerificationStatus {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(DocumentVerificationStatus::Pending),
            "processing" => Ok(DocumentVerificationStatus::Processing),
            "approved" => Ok(DocumentVerificationStatus::Approved),
            "rejected" => Ok(DocumentVerificationStatus::Rejected),
            _ => Err(format!("Invalid document verification status: {}", s)),
        }
    }
}

pub struct KycListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<String>,
    pub risk_level: Option<String>,
    pub requires_review: Option<bool>,   // <-- add this
}

// #[derive(Debug, Clone, Serialize, Deserialize)]

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKycParams {
    pub user_id: Uuid,
    pub approved: bool,
    pub notes: Option<String>,
    pub approved_by: Uuid,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct KycListItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub verification_status: Option<String>,  // <- text-safe
    pub risk_level: Option<String>,           // <- text-safe
    pub created_at: DateTime<Utc>,
}
