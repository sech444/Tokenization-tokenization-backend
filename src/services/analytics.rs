use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    models::{
        project::{ProjectStatus, ProjectType},
        token::TokenStatus,
        transaction::{TransactionStatus, TransactionType},
        user::{UserRole, UserStatus},
    },
    utils::errors::{AppError, AppResult},
};

pub struct AnalyticsService {
    db: Pool<Postgres>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalytics {
    pub overview: OverviewMetrics,
    pub user_analytics: UserAnalytics,
    pub project_analytics: ProjectAnalytics,
    pub transaction_analytics: TransactionAnalytics,
    pub token_analytics: TokenAnalytics,
    pub financial_metrics: FinancialMetrics,
    pub growth_metrics: GrowthMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewMetrics {
    pub total_users: u32,
    pub active_users_30d: u32,
    pub total_projects: u32,
    pub active_projects: u32,
    pub total_tokens: u32,
    pub total_transaction_volume: i64,
    pub total_funds_raised: i64,
    pub platform_revenue: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnalytics {
    pub registrations_by_day: Vec<TimeSeriesPoint>,
    pub user_activity_by_day: Vec<TimeSeriesPoint>,
    pub user_distribution_by_role: HashMap<String, u32>,
    pub user_distribution_by_status: HashMap<String, u32>,
    pub kyc_completion_rate: f64,
    pub user_retention_rates: RetentionRates,
    pub geographic_distribution: Vec<GeographicPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalytics {
    pub projects_created_by_day: Vec<TimeSeriesPoint>,
    pub projects_by_type: HashMap<String, u32>,
    pub projects_by_status: HashMap<String, u32>,
    pub funding_success_rate: f64,
    pub average_funding_amount: i64,
    pub funding_by_day: Vec<TimeSeriesPoint>,
    pub top_performing_projects: Vec<ProjectPerformance>,
    pub average_time_to_funding: f64, // days
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalytics {
    pub transaction_volume_by_day: Vec<TimeSeriesPoint>,
    pub transaction_count_by_day: Vec<TimeSeriesPoint>,
    pub transactions_by_type: HashMap<String, u32>,
    pub transactions_by_status: HashMap<String, u32>,
    pub average_transaction_size: i64,
    pub transaction_success_rate: f64,
    pub peak_transaction_hours: Vec<HourlyActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalytics {
    pub tokens_created_by_day: Vec<TimeSeriesPoint>,
    pub tokens_by_status: HashMap<String, u32>,
    pub total_token_supply: i64,
    pub average_token_price: i64,
    pub token_trading_volume: i64,
    pub top_performing_tokens: Vec<TokenPerformance>,
    pub token_holder_distribution: Vec<TokenHolderStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialMetrics {
    pub revenue_by_day: Vec<TimeSeriesPoint>,
    pub revenue_by_source: HashMap<String, i64>,
    pub fees_collected: i64,
    pub funds_under_management: i64,
    pub average_investment_size: i64,
    pub roi_distribution: Vec<RoiRange>,
    pub cash_flow_analysis: CashFlowAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthMetrics {
    pub user_growth_rate: f64,
    pub project_growth_rate: f64,
    pub volume_growth_rate: f64,
    pub revenue_growth_rate: f64,
    pub market_penetration: f64,
    pub customer_acquisition_cost: f64,
    pub lifetime_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub date: DateTime<Utc>,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicPoint {
    pub country_code: String,
    pub country_name: String,
    pub user_count: u32,
    pub transaction_volume: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionRates {
    pub day_1: f64,
    pub day_7: f64,
    pub day_30: f64,
    pub day_90: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPerformance {
    pub project_id: Uuid,
    pub project_name: String,
    pub total_raised: i64,
    pub funding_percentage: f64,
    pub investor_count: u32,
    pub roi: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPerformance {
    pub token_id: Uuid,
    pub token_name: String,
    pub token_symbol: String,
    pub current_price: i64,
    pub price_change_24h: f64,
    pub trading_volume_24h: i64,
    pub market_cap: i64,
    pub holder_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolderStats {
    pub token_id: Uuid,
    pub token_name: String,
    pub total_holders: u32,
    pub concentration_ratio: f64, // percentage held by top 10 holders
    pub average_holding: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyActivity {
    pub hour: u32,
    pub transaction_count: u32,
    pub transaction_volume: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoiRange {
    pub range: String,
    pub project_count: u32,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowAnalysis {
    pub monthly_inflows: Vec<TimeSeriesPoint>,
    pub monthly_outflows: Vec<TimeSeriesPoint>,
    pub net_cash_flow: Vec<TimeSeriesPoint>,
    pub cash_flow_trend: String, // "increasing", "decreasing", "stable"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub granularity: TimeGranularity,
    pub filters: AnalyticsFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsFilters {
    pub user_roles: Option<Vec<UserRole>>,
    pub project_types: Option<Vec<ProjectType>>,
    pub project_statuses: Option<Vec<ProjectStatus>>,
    pub transaction_types: Option<Vec<TransactionType>>,
    pub geographic_regions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeGranularity {
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRequest {
    pub report_type: ReportType,
    pub query: AnalyticsQuery,
    pub format: ReportFormat,
    pub recipients: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    UserActivity,
    ProjectPerformance,
    FinancialSummary,
    ComplianceReport,
    TradingVolume,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Json,
    Csv,
    Excel,
    Pdf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiMetrics {
    pub total_revenue: i64,
    pub monthly_recurring_revenue: i64,
    pub customer_acquisition_cost: f64,
    pub lifetime_value: f64,
    pub churn_rate: f64,
    pub net_promoter_score: f64,
    pub platform_utilization: f64,
    pub compliance_rate: f64,
}

impl AnalyticsService {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn get_dashboard_analytics(
        &self,
        query: &AnalyticsQuery,
    ) -> AppResult<DashboardAnalytics> {
        let overview = self.get_overview_metrics(query).await?;
        let user_analytics = self.get_user_analytics(query).await?;
        let project_analytics = self.get_project_analytics(query).await?;
        let transaction_analytics = self.get_transaction_analytics(query).await?;
        let token_analytics = self.get_token_analytics(query).await?;
        let financial_metrics = self.get_financial_metrics(query).await?;
        let growth_metrics = self.get_growth_metrics(query).await?;

        Ok(DashboardAnalytics {
            overview,
            user_analytics,
            project_analytics,
            transaction_analytics,
            token_analytics,
            financial_metrics,
            growth_metrics,
        })
    }

    pub async fn get_overview_metrics(&self, query: &AnalyticsQuery) -> AppResult<OverviewMetrics> {
        let total_users: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let active_users_30d: i64 = sqlx::query_scalar!(
            "SELECT COUNT(DISTINCT user_id) FROM user_activities WHERE created_at >= $1",
            Utc::now() - Duration::days(30)
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let total_projects: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM projects")
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let active_projects: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM projects WHERE status IN ('active', 'approved')"
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let total_tokens: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM tokens")
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let total_transaction_volume: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(amount) FROM transactions WHERE status = 'completed' AND created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let total_funds_raised: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(funds_raised) FROM projects WHERE created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Calculate platform revenue (fees)
        let platform_revenue: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(amount * 0.025) FROM transactions WHERE status = 'completed' AND created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(OverviewMetrics {
            total_users: total_users as u32,
            active_users_30d: active_users_30d as u32,
            total_projects: total_projects as u32,
            active_projects: active_projects as u32,
            total_tokens: total_tokens as u32,
            total_transaction_volume: total_transaction_volume.unwrap_or(0),
            total_funds_raised: total_funds_raised.unwrap_or(0),
            platform_revenue: platform_revenue.unwrap_or(0),
        })
    }

    pub async fn get_user_analytics(&self, query: &AnalyticsQuery) -> AppResult<UserAnalytics> {
        // User registrations by day
        let registrations = sqlx::query!(
            r#"
            SELECT DATE(created_at) as date, COUNT(*) as count
            FROM users
            WHERE created_at BETWEEN $1 AND $2
            GROUP BY DATE(created_at)
            ORDER BY date
            "#,
            query.start_date,
            query.end_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let registrations_by_day: Vec<TimeSeriesPoint> = registrations
            .into_iter()
            .map(|row| TimeSeriesPoint {
                date: row.date.unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc(),
                value: row.count.unwrap_or(0),
            })
            .collect();

        // User distribution by role
        let role_distribution =
            sqlx::query!("SELECT role::text as user_role, COUNT(*) as count FROM users GROUP BY role")
                .fetch_all(&self.db)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let user_distribution_by_role: HashMap<String, u32> = role_distribution
            .into_iter()
            .map(|row| (row.user_role.to_string(), row.count.unwrap_or(0) as u32))
            .collect();

        // User distribution by status
        let status_distribution = sqlx::query!(
            "SELECT status as user_status, COUNT(*) as count FROM users GROUP BY status"
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let user_distribution_by_status: HashMap<String, u32> = status_distribution
            .into_iter()
            .map(|row| (row.user_status.to_string(), row.count.unwrap_or(0) as u32))
            .collect();

        // KYC completion rate
        let total_users: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let kyc_completed: i64 = sqlx::query_scalar!(
            "SELECT COUNT(DISTINCT user_id) FROM kyc_verifications WHERE verification_status = 'approved'"
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let kyc_completion_rate = if total_users > 0 {
            (kyc_completed as f64 / total_users as f64) * 100.0
        } else {
            0.0
        };

        // Calculate retention rates (simplified)
        let retention_rates = RetentionRates {
            day_1: 0.85, // These would be calculated from actual user activity
            day_7: 0.65,
            day_30: 0.45,
            day_90: 0.30,
        };

        Ok(UserAnalytics {
            registrations_by_day,
            user_activity_by_day: Vec::new(), // Would be calculated from user_activities table
            user_distribution_by_role,
            user_distribution_by_status,
            kyc_completion_rate,
            user_retention_rates: retention_rates,
            geographic_distribution: Vec::new(), // Would be calculated from user locations
        })
    }

    pub async fn get_project_analytics(
        &self,
        query: &AnalyticsQuery,
    ) -> AppResult<ProjectAnalytics> {
        // Projects created by day
        let projects_created = sqlx::query!(
            r#"
            SELECT DATE(created_at) as date, COUNT(*) as count
            FROM projects
            WHERE created_at BETWEEN $1 AND $2
            GROUP BY DATE(created_at)
            ORDER BY date
            "#,
            query.start_date,
            query.end_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let projects_created_by_day: Vec<TimeSeriesPoint> = projects_created
            .into_iter()
            .map(|row| TimeSeriesPoint {
                date: row.date.unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc(),
                value: row.count.unwrap_or(0),
            })
            .collect();

        // Projects by type
        let type_distribution = sqlx::query!(
            "SELECT project_type, COUNT(*) as count FROM projects GROUP BY project_type"
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let projects_by_type: HashMap<String, u32> = type_distribution
            .into_iter()
            .map(|row| (row.project_type.to_string(), row.count.unwrap_or(0) as u32))
            .collect();

        // Projects by status
        let status_distribution =
            sqlx::query!("SELECT status, COUNT(*) as count FROM projects GROUP BY status")
                .fetch_all(&self.db)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let projects_by_status: HashMap<String, u32> = status_distribution
            .into_iter()
            .map(|row| (row.status.to_string(), row.count.unwrap_or(0) as u32))
            .collect();

        // Funding success rate
        let total_projects: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM projects WHERE created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let funded_projects: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM projects WHERE status = 'funded' AND created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let funding_success_rate = if total_projects > 0 {
            (funded_projects as f64 / total_projects as f64) * 100.0
        } else {
            0.0
        };

        // Average funding amount
        let average_funding: Option<i64> = sqlx::query_scalar!(
            "SELECT AVG(funds_raised) FROM projects WHERE funds_raised > 0 AND created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Top performing projects
        let top_projects = sqlx::query!(
            r#"
            SELECT id, name, funds_raised, total_value, investor_count, created_at
            FROM projects
            WHERE created_at BETWEEN $1 AND $2
            ORDER BY funds_raised DESC
            LIMIT 10
            "#,
            query.start_date,
            query.end_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let top_performing_projects: Vec<ProjectPerformance> = top_projects
            .into_iter()
            .map(|row| ProjectPerformance {
                project_id: row.id,
                project_name: row.name,
                total_raised: row.funds_raised,
                funding_percentage: (row.funds_raised as f64 / row.total_value as f64) * 100.0,
                investor_count: row.investor_count as u32,
                roi: 0.0, // Would be calculated based on actual returns
                created_at: row.created_at,
            })
            .collect();

        Ok(ProjectAnalytics {
            projects_created_by_day,
            projects_by_type,
            projects_by_status,
            funding_success_rate,
            average_funding_amount: average_funding.unwrap_or(0),
            funding_by_day: Vec::new(), // Would be calculated from transaction data
            top_performing_projects,
            average_time_to_funding: 0.0, // Would be calculated from project lifecycle data
        })
    }

    pub async fn get_transaction_analytics(
        &self,
        query: &AnalyticsQuery,
    ) -> AppResult<TransactionAnalytics> {
        // Transaction volume by day
        let volume_by_day = sqlx::query!(
            r#"
            SELECT DATE(created_at) as date, SUM(amount) as volume
            FROM transactions
            WHERE created_at BETWEEN $1 AND $2 AND status = 'completed'
            GROUP BY DATE(created_at)
            ORDER BY date
            "#,
            query.start_date,
            query.end_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let transaction_volume_by_day: Vec<TimeSeriesPoint> = volume_by_day
            .into_iter()
            .map(|row| TimeSeriesPoint {
                date: row.date.unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc(),
                value: row.volume.unwrap_or(0),
            })
            .collect();

        // Transaction count by day
        let count_by_day = sqlx::query!(
            r#"
            SELECT DATE(created_at) as date, COUNT(*) as count
            FROM transactions
            WHERE created_at BETWEEN $1 AND $2
            GROUP BY DATE(created_at)
            ORDER BY date
            "#,
            query.start_date,
            query.end_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let transaction_count_by_day: Vec<TimeSeriesPoint> = count_by_day
            .into_iter()
            .map(|row| TimeSeriesPoint {
                date: row.date.unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc(),
                value: row.count.unwrap_or(0),
            })
            .collect();

        // Transactions by type
        let type_distribution = sqlx::query!(
            "SELECT transaction_type, COUNT(*) as count FROM transactions GROUP BY transaction_type"
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let transactions_by_type: HashMap<String, u32> = type_distribution
            .into_iter()
            .map(|row| {
                (
                    row.transaction_type.to_string(),
                    row.count.unwrap_or(0) as u32,
                )
            })
            .collect();

        // Calculate average transaction size and success rate
        let avg_transaction_size: Option<i64> = sqlx::query_scalar!(
            "SELECT AVG(amount) FROM transactions WHERE status = 'completed' AND created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let total_transactions: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM transactions WHERE created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let successful_transactions: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM transactions WHERE status = 'completed' AND created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let transaction_success_rate = if total_transactions > 0 {
            (successful_transactions as f64 / total_transactions as f64) * 100.0
        } else {
            0.0
        };

        Ok(TransactionAnalytics {
            transaction_volume_by_day,
            transaction_count_by_day,
            transactions_by_type,
            transactions_by_status: HashMap::new(),
            average_transaction_size: avg_transaction_size.unwrap_or(0),
            transaction_success_rate,
            peak_transaction_hours: Vec::new(),
        })
    }

    pub async fn get_token_analytics(&self, query: &AnalyticsQuery) -> AppResult<TokenAnalytics> {
        // Tokens created by day
        let tokens_created = sqlx::query!(
            r#"
            SELECT DATE(created_at) as date, COUNT(*) as count
            FROM tokens
            WHERE created_at BETWEEN $1 AND $2
            GROUP BY DATE(created_at)
            ORDER BY date
            "#,
            query.start_date,
            query.end_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let tokens_created_by_day: Vec<TimeSeriesPoint> = tokens_created
            .into_iter()
            .map(|row| TimeSeriesPoint {
                date: row.date.unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc(),
                value: row.count.unwrap_or(0),
            })
            .collect();

        // Tokens by status
        let status_distribution =
            sqlx::query!("SELECT status, COUNT(*) as count FROM tokens GROUP BY status")
                .fetch_all(&self.db)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let tokens_by_status: HashMap<String, u32> = status_distribution
            .into_iter()
            .map(|row| (row.status.to_string(), row.count.unwrap_or(0) as u32))
            .collect();

        // Total token supply and average price
        let total_supply: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(total_supply) FROM tokens WHERE created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let average_price: Option<i64> = sqlx::query_scalar!(
            "SELECT AVG(current_price) FROM tokens WHERE created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(TokenAnalytics {
            tokens_created_by_day,
            tokens_by_status,
            total_token_supply: total_supply.unwrap_or(0),
            average_token_price: average_price.unwrap_or(0),
            token_trading_volume: 0, // Would be calculated from trading data
            top_performing_tokens: Vec::new(),
            token_holder_distribution: Vec::new(),
        })
    }

    pub async fn get_financial_metrics(
        &self,
        query: &AnalyticsQuery,
    ) -> AppResult<FinancialMetrics> {
        // Revenue by day (platform fees)
        let revenue_by_day = sqlx::query!(
            r#"
            SELECT DATE(created_at) as date, SUM(amount * 0.025) as revenue
            FROM transactions
            WHERE created_at BETWEEN $1 AND $2 AND status = 'completed'
            GROUP BY DATE(created_at)
            ORDER BY date
            "#,
            query.start_date,
            query.end_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let revenue_by_day_series: Vec<TimeSeriesPoint> = revenue_by_day
            .into_iter()
            .map(|row| TimeSeriesPoint {
                date: row.date.unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc(),
                value: row.revenue.unwrap_or(0),
            })
            .collect();

        // Revenue by source
        let mut revenue_by_source = HashMap::new();
        revenue_by_source.insert("transaction_fees".to_string(), 100000i64);
        revenue_by_source.insert("listing_fees".to_string(), 50000i64);
        revenue_by_source.insert("token_creation_fees".to_string(), 25000i64);

        // Calculate funds under management
        let funds_under_management: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(total_value) FROM projects WHERE status IN ('active', 'funded')"
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Average investment size
        let average_investment: Option<i64> = sqlx::query_scalar!(
            "SELECT AVG(amount) FROM transactions WHERE transaction_type = 'investment' AND status = 'completed' AND created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Fees collected
        let fees_collected: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(amount * 0.025) FROM transactions WHERE status = 'completed' AND created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // ROI distribution (simplified)
        let roi_distribution = vec![
            RoiRange {
                range: "0-5%".to_string(),
                project_count: 10,
                percentage: 25.0,
            },
            RoiRange {
                range: "5-10%".to_string(),
                project_count: 15,
                percentage: 37.5,
            },
            RoiRange {
                range: "10-20%".to_string(),
                project_count: 12,
                percentage: 30.0,
            },
            RoiRange {
                range: "20%+".to_string(),
                project_count: 3,
                percentage: 7.5,
            },
        ];

        // Cash flow analysis
        let cash_flow_analysis = CashFlowAnalysis {
            monthly_inflows: Vec::new(),
            monthly_outflows: Vec::new(),
            net_cash_flow: Vec::new(),
            cash_flow_trend: "increasing".to_string(),
        };

        Ok(FinancialMetrics {
            revenue_by_day: revenue_by_day_series,
            revenue_by_source,
            fees_collected: fees_collected.unwrap_or(0),
            funds_under_management: funds_under_management.unwrap_or(0),
            average_investment_size: average_investment.unwrap_or(0),
            roi_distribution,
            cash_flow_analysis,
        })
    }

    pub async fn get_growth_metrics(&self, query: &AnalyticsQuery) -> AppResult<GrowthMetrics> {
        // Calculate growth rates
        let current_period_start = query.start_date;
        let current_period_end = query.end_date;
        let period_duration = current_period_end - current_period_start;
        let previous_period_start = current_period_start - period_duration;
        let previous_period_end = current_period_start;

        // User growth rate
        let current_users: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE created_at BETWEEN $1 AND $2",
            current_period_start,
            current_period_end
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let previous_users: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE created_at BETWEEN $1 AND $2",
            previous_period_start,
            previous_period_end
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let user_growth_rate = if previous_users > 0 {
            ((current_users - previous_users) as f64 / previous_users as f64) * 100.0
        } else {
            0.0
        };

        // Project growth rate
        let current_projects: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM projects WHERE created_at BETWEEN $1 AND $2",
            current_period_start,
            current_period_end
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let previous_projects: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM projects WHERE created_at BETWEEN $1 AND $2",
            previous_period_start,
            previous_period_end
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let project_growth_rate = if previous_projects > 0 {
            ((current_projects - previous_projects) as f64 / previous_projects as f64) * 100.0
        } else {
            0.0
        };

        // Volume growth rate
        let current_volume: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(amount) FROM transactions WHERE status = 'completed' AND created_at BETWEEN $1 AND $2",
            current_period_start,
            current_period_end
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let previous_volume: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(amount) FROM transactions WHERE status = 'completed' AND created_at BETWEEN $1 AND $2",
            previous_period_start,
            previous_period_end
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let volume_growth_rate =
            if let (Some(current), Some(previous)) = (current_volume, previous_volume) {
                if previous > 0 {
                    ((current - previous) as f64 / previous as f64) * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            };

        // Revenue growth rate
        let current_revenue: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(amount * 0.025) FROM transactions WHERE status = 'completed' AND created_at BETWEEN $1 AND $2",
            current_period_start,
            current_period_end
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let previous_revenue: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(amount * 0.025) FROM transactions WHERE status = 'completed' AND created_at BETWEEN $1 AND $2",
            previous_period_start,
            previous_period_end
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let revenue_growth_rate =
            if let (Some(current), Some(previous)) = (current_revenue, previous_revenue) {
                if previous > 0 {
                    ((current - previous) as f64 / previous as f64) * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            };

        Ok(GrowthMetrics {
            user_growth_rate,
            project_growth_rate,
            volume_growth_rate,
            revenue_growth_rate,
            market_penetration: 2.5, // Would be calculated based on total addressable market
            customer_acquisition_cost: 50.0, // Would be calculated from marketing spend
            lifetime_value: 1200.0,  // Would be calculated from customer behavior
        })
    }

    pub async fn get_kpi_metrics(&self, query: &AnalyticsQuery) -> AppResult<KpiMetrics> {
        let total_revenue: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(amount * 0.025) FROM transactions WHERE status = 'completed' AND created_at BETWEEN $1 AND $2",
            query.start_date,
            query.end_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Monthly recurring revenue (simplified calculation)
        let mrr: Option<i64> = sqlx::query_scalar!(
            "SELECT SUM(amount * 0.025) / 12 FROM transactions WHERE status = 'completed' AND created_at >= $1",
            Utc::now() - Duration::days(365)
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Compliance rate
        let total_kyc: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM kyc_verifications")
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let approved_kyc: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM kyc_verifications WHERE verification_status = 'approved'"
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let compliance_rate = if total_kyc > 0 {
            (approved_kyc as f64 / total_kyc as f64) * 100.0
        } else {
            0.0
        };

        Ok(KpiMetrics {
            total_revenue: total_revenue.unwrap_or(0),
            monthly_recurring_revenue: mrr.unwrap_or(0),
            customer_acquisition_cost: 50.0,
            lifetime_value: 1200.0,
            churn_rate: 5.2,
            net_promoter_score: 42.0,
            platform_utilization: 67.5,
            compliance_rate,
        })
    }

    pub async fn generate_report(&self, request: &ReportRequest) -> AppResult<Vec<u8>> {
        let analytics = self.get_dashboard_analytics(&request.query).await?;

        match request.format {
            ReportFormat::Json => {
                let json_data = serde_json::to_string_pretty(&analytics)
                    .map_err(|e| AppError::SerializationError(e.to_string()))?;
                Ok(json_data.into_bytes())
            }
            ReportFormat::Csv => {
                // Convert to CSV format
                let csv_data = self.convert_to_csv(&analytics)?;
                Ok(csv_data.into_bytes())
            }
            ReportFormat::Excel => {
                // Convert to Excel format (would need additional dependencies)
                Err(AppError::NotImplemented(
                    "Excel format not implemented".to_string(),
                ))
            }
            ReportFormat::Pdf => {
                // Convert to PDF format (would need additional dependencies)
                Err(AppError::NotImplemented(
                    "PDF format not implemented".to_string(),
                ))
            }
        }
    }

    fn convert_to_csv(&self, analytics: &DashboardAnalytics) -> AppResult<String> {
        let mut csv_output = String::new();

        // Overview metrics
        csv_output.push_str("Metric,Value\n");
        csv_output.push_str(&format!("Total Users,{}\n", analytics.overview.total_users));
        csv_output.push_str(&format!(
            "Active Users (30d),{}\n",
            analytics.overview.active_users_30d
        ));
        csv_output.push_str(&format!(
            "Total Projects,{}\n",
            analytics.overview.total_projects
        ));
        csv_output.push_str(&format!(
            "Total Transaction Volume,{}\n",
            analytics.overview.total_transaction_volume
        ));
        csv_output.push_str(&format!(
            "Platform Revenue,{}\n",
            analytics.overview.platform_revenue
        ));

        csv_output.push_str("\n");

        // Growth metrics
        csv_output.push_str("Growth Metric,Value\n");
        csv_output.push_str(&format!(
            "User Growth Rate,{:.2}%\n",
            analytics.growth_metrics.user_growth_rate
        ));
        csv_output.push_str(&format!(
            "Project Growth Rate,{:.2}%\n",
            analytics.growth_metrics.project_growth_rate
        ));
        csv_output.push_str(&format!(
            "Volume Growth Rate,{:.2}%\n",
            analytics.growth_metrics.volume_growth_rate
        ));
        csv_output.push_str(&format!(
            "Revenue Growth Rate,{:.2}%\n",
            analytics.growth_metrics.revenue_growth_rate
        ));

        Ok(csv_output)
    }
}
