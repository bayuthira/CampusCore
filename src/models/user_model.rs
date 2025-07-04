// src/models/user_model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

// Struct untuk menampilkan detail user beserta perannya
#[derive(Debug, Serialize, FromRow)]
pub struct UserWithRoles {
    pub id: Uuid,
    pub full_name: String,
    pub username: String,
    pub email: Option<String>,
    pub is_active: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    // `roles` akan kita isi secara manual setelah query
    pub roles: Vec<String>,
}

// Struct untuk payload saat memberi/mencabut peran
#[derive(Debug, Deserialize)]
pub struct RoleAssignmentPayload {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct UserData {
    pub id: Uuid,
    pub username: String,
    pub full_name: String,
    pub roles: Vec<String>,
}