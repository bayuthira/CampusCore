// src/handlers/lookup_handler.rs
use crate::{db::DbPool, errors::AppError};
use axum::{extract::State, Json};
use serde::Deserialize;
use sqlx::FromRow;

// Struct sementara untuk menampung hasil query dari pg_enum
#[derive(Debug, FromRow, Deserialize)]
struct EnumLabel {
    enumlabel: String,
}

/// Handler untuk mengambil semua nilai dari ENUM 'EnrollmentStatus'
pub async fn get_enrollment_statuses_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<String>>, AppError> {
    // Jalankan query ke katalog sistem postgres
    let enum_values = sqlx::query_as::<_, EnumLabel>(
        r#"
        SELECT enumlabel
        FROM pg_enum
        JOIN pg_type ON pg_enum.enumtypid = pg_type.oid
        WHERE pg_type.typname = 'EnrollmentStatus'
        ORDER BY enumsortorder
        "#,
    )
    .fetch_all(&pool)
    .await?;

    // Ubah hasil dari Vec<EnumLabel> menjadi Vec<String> agar format JSON-nya bersih
    let status_list: Vec<String> = enum_values.into_iter().map(|item| item.enumlabel).collect();

    Ok(Json(status_list))
}


/// Handler untuk mengambil semua nilai dari ENUM 'KondisiAset'
pub async fn get_kondisi_aset_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<String>>, AppError> {
    // Struct sementara, sama seperti sebelumnya
    #[derive(sqlx::FromRow)]
    struct EnumLabel {
        enumlabel: String,
    }

    // Query ke katalog sistem, ganti typname menjadi 'KondisiAset'
    let enum_values = sqlx::query_as::<_, EnumLabel>(
        r#"
        SELECT enumlabel
        FROM pg_enum
        JOIN pg_type ON pg_enum.enumtypid = pg_type.oid
        WHERE pg_type.typname = 'KondisiAset'
        ORDER BY enumsortorder
        "#,
    )
    .fetch_all(&pool)
    .await?;

    // Ubah hasilnya menjadi array string
    let list: Vec<String> = enum_values.into_iter().map(|item| item.enumlabel).collect();

    Ok(Json(list))
}