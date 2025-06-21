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
pub async fn login_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<TokenResponse>, AppError> {
    // 1. Cari user berdasarkan username
    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = $1 AND is_active = true",
        payload.username
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(anyhow::anyhow!("Username atau password salah"))?; // Error generik

    // 2. Verifikasi password
    let is_password_valid = verify(payload.password, &user.password_hash)?;
    if !is_password_valid {
        return Err(anyhow::anyhow!("Username atau password salah").into());
    }

    // 3. Ambil semua peran (roles) yang dimiliki user ini
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
    .map(|row| row.name) // Ambil hanya nama rolenya
    .collect(); // Kumpulkan menjadi Vec<String>
    // 4. Buat JWT Claims dengan data roles
    let now = chrono::Utc::now().timestamp();
    let claims = TokenClaims {
        sub: user.id,
        roles: user_roles, // <-- Masukkan roles ke dalam claims
        iat: now,
        exp: now + CONFIG.jwt_expires_in,
    };

    // 5. Encode token (tidak ada perubahan di sini)
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
    )?;

    Ok(Json(TokenResponse { token }))
}
