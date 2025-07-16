// src/handlers/role_handler.rs
use super::{
    role_model::RoleResponse,
    role_repo as role_repo,
};


use crate::{
    db::DbPool,
    errors::AppError,
};
use axum::{extract::State, Json};

pub async fn get_all_roles_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<RoleResponse>>, AppError> {
    let roles = role_repo::get_all_roles_repo(&pool).await?;
    Ok(Json(roles))
}