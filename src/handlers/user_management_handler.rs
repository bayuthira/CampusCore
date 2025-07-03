// src/handlers/user_management_handler.rs
use crate::{
    db::DbPool,
    errors::AppError,
    models::{general_model::SuccessResponse, user_model::{RoleAssignmentPayload, UserWithRoles}},
    repositories::user_management_repo,
};
use axum::{extract::State, http::StatusCode, Json};

pub async fn list_users_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<UserWithRoles>>, AppError> {
    let users = user_management_repo::get_all_users_with_roles_repo(&pool).await?;
    Ok(Json(users))
}

pub async fn assign_role_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<RoleAssignmentPayload>,
) -> Result<(StatusCode, Json<SuccessResponse>), AppError> {
    user_management_repo::assign_role_to_user_repo(&pool, payload).await?;
    let response = SuccessResponse {
        message: "Peran berhasil diberikan.".to_string(),
    };
    Ok((StatusCode::OK, Json(response)))
}

pub async fn revoke_role_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<RoleAssignmentPayload>,
) -> Result<(StatusCode, Json<SuccessResponse>), AppError> {
    user_management_repo::revoke_role_from_user_repo(&pool, payload).await?;
    let response = SuccessResponse {
        message: "Peran berhasil dicabut.".to_string(),
    };
    Ok((StatusCode::OK, Json(response)))
}