use uuid::Uuid;
use sqlx::FromRow;
use crate::models::project::Project;
use crate::utils::errors::{AppError, AppResult};

pub struct ProjectService<'a> {
    db: &'a sqlx::PgPool,
}

impl<'a> ProjectService<'a> {
    pub fn new(db: &'a sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn get_project_by_id(&self, project_id: Uuid) -> AppResult<Project> {
        let project = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE id = $1"
        )
        .bind(project_id)
        .fetch_one(self.db)
        .await
        .map_err(|_| AppError::NotFound("Project not found".into()))?;

        Ok(project)
    }
}
