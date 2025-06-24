// src/handlers/auth_handler.rs
use crate::auth::TokenClaims;
use crate::{
    config::CONFIG,
    db::DbPool,
    errors::AppError,
    models::auth_model::{LoginPayload, RegisterPayload, TokenResponse},
};
use axum::{Json, extract::State, http::StatusCode};
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{EncodingKey, Header, encode};
use anyhow::anyhow;

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
) -> Result<Json<TokenResponse>, AppError> {
    // 1. Cari user berdasarkan username (tidak ada perubahan)
    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = $1 AND is_active = true",
        payload.username
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(anyhow!("Username atau password salah"))?;

    // 2. Verifikasi password (tidak ada perubahan)
    let is_password_valid = verify(payload.password, &user.password_hash)?;
    if !is_password_valid {
        return Err(anyhow!("Username atau password salah").into());
    }

    // 3. Ambil semua peran (roles) yang dimiliki user ini (tidak ada perubahan)
    let user_roles = sqlx::query!(
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

    // --- PERUBAHAN UTAMA DI SINI ---
    // 4. Buat JWT Claims dengan pustaka `time`
    let now = time::OffsetDateTime::now_utc();
    let claims = TokenClaims {
        sub: user.id,
        roles: user_roles,
        iat: now.unix_timestamp(), // Gunakan .unix_timestamp()
        exp: (now + time::Duration::seconds(CONFIG.jwt_expires_in)).unix_timestamp(), // Kalkulasi waktu kedaluwarsa
    };
    // --- AKHIR PERUBAHAN ---

    // 5. Encode token (tidak ada perubahan)
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
    )?;

    Ok(Json(TokenResponse { token }))
}