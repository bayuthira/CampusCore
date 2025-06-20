// src/errors.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    SqlxError(sqlx::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::SqlxError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::SqlxError(sqlx::Error::Database(db_err)) => {
                // Cek apakah ini PostgreSQL error
                if let Some(pg_err) = db_err.as_ref().try_downcast_ref::<sqlx::postgres::PgDatabaseError>() {
                    match pg_err.code() {
                        "23505" => (
                            StatusCode::CONFLICT,
                            "Data dengan kunci tersebut sudah ada.".to_string(),
                        ),
                        _ => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Terjadi masalah pada database: {}", pg_err),
                        ),
                    }
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Terjadi masalah tak terduga pada database.".to_string(),
                    )
                }
            }
            AppError::SqlxError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Terjadi masalah internal sistem.".to_string(),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}