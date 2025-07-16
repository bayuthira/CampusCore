// src/repositories/role_repo.rs
use super::{
    role_model::RoleResponse,
};


use crate::{
    db::DbPool,
    errors::AppError,
};

pub async fn get_all_roles_repo(pool: &DbPool) -> Result<Vec<RoleResponse>, AppError> {
    let roles = sqlx::query_as!(
        RoleResponse,
        "SELECT id, name FROM roles ORDER BY name ASC"
    )
    .fetch_all(pool)
    .await?;
    Ok(roles)
}