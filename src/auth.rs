// src/auth.rs

use crate::{config::CONFIG, errors::AppError};
use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::Future; // <-- Tambahkan ini
use std::pin::Pin;       // <-- dan ini
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: Uuid,
    pub roles: Vec<String>,
    pub iat: i64,
    pub exp: i64,
}

// Fungsi auth_middleware (tidak ada perubahan)
pub async fn auth_middleware(mut req: Request<Body>, next: Next) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| auth_value.strip_prefix("Bearer "));

    let token = token.ok_or_else(|| anyhow::anyhow!("Token otentikasi tidak ditemukan"))?;

    let claims = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| anyhow::anyhow!("Token tidak valid atau telah kedaluwarsa"))?
    .claims;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}


// --- PERBAIKAN UTAMA DI SINI ---
// Middleware Factory yang menggunakan pola `Box<dyn Future>` yang stabil
pub fn require_role(
    required_roles: Vec<String>,
) -> impl Fn(Request<Body>, Next) -> Pin<Box<dyn Future<Output = Result<Response, AppError>> + Send>> + Clone {
    move |req: Request<Body>, next: Next| {
        let roles_to_check = required_roles.clone();
        
        // Kita "kotak-kan" async block kita dengan `Box::pin()`
        Box::pin(async move {
            let claims = req.extensions().get::<TokenClaims>()
                .ok_or_else(|| anyhow::anyhow!("Data otentikasi tidak ditemukan di request."))?;

            let has_required_role = claims.roles.iter().any(|user_role| roles_to_check.contains(user_role));

            if has_required_role {
                Ok(next.run(req).await)
            } else {
                Err(AppError::Forbidden)
            }
        })
    }
}