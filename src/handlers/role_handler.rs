// src/handlers/role_handler.rs
use crate::{
    db::DbPool,
    errors::AppError,
    models::role_model::RoleResponse,
    repositories::role_repo,
};
use axum::{extract::State, Json};

pub async fn get_all_roles_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<RoleResponse>>, AppError> {
    let roles = role_repo::get_all_roles_repo(&pool).await?;
    Ok(Json(roles))
}