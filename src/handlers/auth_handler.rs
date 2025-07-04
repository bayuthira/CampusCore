// src/handlers/auth_handler.rs
use crate::auth::TokenClaims;
use crate::{
    config::CONFIG,
    db::DbPool,
    errors::AppError,
    models::auth_model::{LoginPayload, RegisterPayload, LoginSuccessResponse},
    models::user_model::UserData,
};

use axum::{Json, extract::State, http::StatusCode};
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{EncodingKey, Header, encode};

// Handler untuk registrasi user baru
pub async fn register_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<RegisterPayload>,
) -> Result<StatusCode, AppError> {
    // Hash password dengan bcrypt
    let hashed_password = hash(payload.password, DEFAULT_COST)?;

    // Simpan user baru ke database
    sqlx::query!(
        "INSERT INTO users (full_name, username, email, password_hash) VALUES ($1, $2, $3, $4)",
        payload.full_name,
        payload.username,
        payload.email,
        hashed_password,
    )
    .execute(&pool)
    .await?;

    // Di sini nanti Anda bisa otomatis memberikan role default ke user baru
    // dengan INSERT ke tabel `user_roles`.

    Ok(StatusCode::CREATED)
}

// Handler untuk login
// Handler untuk login
pub async fn login_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginSuccessResponse>, AppError> {
    // 1. Cari user berdasarkan username, ambil juga data `full_name`
    let user = sqlx::query!(
        "SELECT id, password_hash, full_name, username FROM users WHERE username = $1 AND is_active = true",
        payload.username
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(anyhow::anyhow!("Username atau password salah"))?;

    // 2. Verifikasi password
    let is_password_valid = verify(payload.password, &user.password_hash)?;
    if !is_password_valid {
        return Err(anyhow::anyhow!("Username atau password salah").into());
    }

    // 3. Ambil semua peran (roles) user dengan anotasi tipe eksplisit
    let user_roles: Vec<String> = sqlx::query!( // <-- PERBAIKAN DI SINI
        r#"
        SELECT r.name FROM roles r
        INNER JOIN user_roles ur ON r.id = ur.role_id
        WHERE ur.user_id = $1
        "#,
        user.id
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|row| row.name)
    .collect();

    // 4. Buat JWT Claims
    let now = time::OffsetDateTime::now_utc();
    let claims = TokenClaims {
        sub: user.id,
        roles: user_roles.clone(),
        iat: now.unix_timestamp(),
        exp: (now + time::Duration::seconds(CONFIG.jwt_expires_in)).unix_timestamp(),
    };

    // 5. Encode token
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
    )?;

    // 6. Buat objek respons yang baru
    let login_response = LoginSuccessResponse {
        token,
        user: UserData {
            id: user.id,
            username: user.username,
            full_name: user.full_name,
            roles: user_roles,
        },
    };

    Ok(Json(login_response))
}