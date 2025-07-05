// src/models/role_model.rs
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
}