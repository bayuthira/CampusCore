// src/models/auth_model.rs
use serde::{Deserialize, Serialize};
use crate::modules::user_management::model::UserData;

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

// Struct untuk response saat login berhasil
#[derive(Debug, Serialize)]
pub struct LoginSuccessResponse {
    pub token: String,
    pub user: UserData,
}