// src/handlers/user_management_handler.rs
use super::{
    model::{RoleAssignmentPayload, UserWithRoles, UpdateUserPayload,ResetPasswordPayload, CreateUserPayload},
    repo as user_management_repo,
};


use crate::{
    db::DbPool,
    errors::AppError,
    modules::general::model::SuccessResponse, 
};
use axum::{extract::{State, Path}, http::StatusCode, Json};
use uuid::Uuid;

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

pub async fn get_user_by_id_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserWithRoles>, AppError> {
    let user = user_management_repo::get_user_by_id_with_roles_repo(&pool, id).await?;
    Ok(Json(user))
}

pub async fn update_user_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserPayload>,
) -> Result<Json<UserWithRoles>, AppError> {
    let updated_user = user_management_repo::update_user_repo(&pool, id, payload).await?;
    Ok(Json(updated_user))
}

pub async fn delete_user_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    user_management_repo::delete_user_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Handler untuk admin me-reset password user
pub async fn reset_password_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResetPasswordPayload>,
) -> Result<Json<SuccessResponse>, AppError> {
    // Hash password baru yang diterima dari payload
    let hashed_password = bcrypt::hash(payload.new_password, bcrypt::DEFAULT_COST)?;

    // Panggil repository untuk menyimpan hash yang baru
    user_management_repo::reset_user_password_repo(&pool, id, &hashed_password).await?;

    let response = SuccessResponse {
        message: "Password user berhasil direset.".to_string(),
    };

    Ok(Json(response))
}

pub async fn create_user_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<(StatusCode, Json<UserWithRoles>), AppError> {
    let new_user = user_management_repo::create_user_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(new_user)))
}