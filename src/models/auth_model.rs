// src/models/auth_model.rs
use serde::{Deserialize, Serialize};
use crate::models::user_model::UserData;

#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    pub full_name: String,
    pub username: String, // Ini bisa NIDN, NIM, dll.
    pub email: String,
    pub password: String,
}

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