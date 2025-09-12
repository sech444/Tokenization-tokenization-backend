// // tokenization-backend/src/handlers/projectsold.rs

// use axum::{
//     extract::{State, Path},
//     response::Json,
//     http::StatusCode,
//     Extension,
// };
// use uuid::Uuid;

// use crate::{
//     AppState,
//     database::projects as db_projects,
//     models::{project::{Project, ProjectStatus}, user::{User, UserRole}},
//     utils::errors::{AppError, AppResult},
// };

// use super::projects::{ // reusing your DTOs
//     ProjectResponse, ProjectListQuery, ProjectListResponse, TokenizeProjectRequest, TokenizationResponse,
// };

// pub async fn delete_project(
//     State(state): State<AppState>,
//     Extension(user): Extension<User>,
//     Path(project_id): Path<Uuid>,
// ) -> AppResult<StatusCode> {
//     let project = db_projects::get_project_by_id(&state.db, project_id)
//         .await?
//         .ok_or_else(|| AppError::not_found("Project not found"))?;

//     if project.owner_id != user.id && !matches!(user.role, UserRole::Admin) {
//         return Err(AppError::forbidden("You can only delete your own projects"));
//     }

//     if matches!(project.status, ProjectStatus::Active | ProjectStatus::Funded) {
//         return Err(AppError::bad_request("Cannot delete active or funded projects"));
//     }

//     if project.is_tokenized {
//         return Err(AppError::bad_request("Cannot delete tokenized projects"));
//     }

//     db_projects::delete_project_by_id(&state.db, project_id).await?;
//     Ok(StatusCode::NO_CONTENT)
// }
