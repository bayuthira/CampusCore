// src/modules/sdm/absensi_handler.rs
use super::{
    absensi_model::{
        ClockPayload, LogAbsensi, LogDayFilter, RekapAbsensiFilter, RekapAbsensiHarian,
        RekapManualPayload,
    },
    absensi_repo as repo, repo as pegawai_repo,
};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use axum::{
    Extension,
    extract::{Json, Multipart, Query, State},
    http::StatusCode,
};
use rust_decimal::Decimal;
use serde_json::Value;
use std::env;
use std::str::FromStr;
use tokio::fs;
use uuid::Uuid;

// ==========================================
// HELPER AZURE FACE API (LIVENESS & VERIFY)
// ==========================================
async fn detect_face_azure(image_bytes: Vec<u8>) -> Result<String, AppError> {
    let endpoint = env::var("AZURE_FACE_ENDPOINT").unwrap_or_default();
    let api_key = env::var("AZURE_FACE_API_KEY").unwrap_or_default();

    if endpoint.is_empty() || api_key.is_empty() {
        // Jika belum disetting, kita loloskan otomatis untuk development
        return Ok("dummy_face_id".to_string());
    }

    let url = format!("{}/face/v1.0/detect?returnFaceId=true", endpoint);
    let client = reqwest::Client::new();

    let res = client
        .post(&url)
        .header("Ocp-Apim-Subscription-Key", api_key)
        .header("Content-Type", "application/octet-stream")
        .body(image_bytes)
        .send()
        .await
        .map_err(|e| {
            AppError::AnyhowError(anyhow::anyhow!("Gagal menghubungi server Azure: {}", e))
        })?;

    let json: Value = res
        .json()
        .await
        .map_err(|e| AppError::AnyhowError(anyhow::anyhow!("Gagal parse respons Azure: {}", e)))?;

    if let Some(arr) = json.as_array() {
        if let Some(first_face) = arr.first() {
            if let Some(face_id) = first_face.get("faceId").and_then(|v| v.as_str()) {
                return Ok(face_id.to_string());
            }
        }
    }

    Err(AppError::Forbidden(
        "Wajah tidak terdeteksi dalam foto selfie Anda!".to_string(),
    ))
}

async fn verify_face_azure(face_id1: &str, face_id2: &str) -> Result<(bool, f64), AppError> {
    if face_id1 == "dummy_face_id" {
        return Ok((true, 0.99)); // Bypass saat development
    }

    let endpoint = env::var("AZURE_FACE_ENDPOINT").unwrap_or_default();
    let api_key = env::var("AZURE_FACE_API_KEY").unwrap_or_default();
    let url = format!("{}/face/v1.0/verify", endpoint);

    let body = serde_json::json!({
        "faceId1": face_id1,
        "faceId2": face_id2
    });

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .header("Ocp-Apim-Subscription-Key", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            AppError::AnyhowError(anyhow::anyhow!("Gagal menghubungi server Azure: {}", e))
        })?;

    let json: Value = res
        .json()
        .await
        .map_err(|e| AppError::AnyhowError(anyhow::anyhow!("Gagal parse respons Azure: {}", e)))?;

    let is_identical = json
        .get("isIdentical")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let confidence = json
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    Ok((is_identical, confidence))
}

// ==========================================
// HELPER EXTRACT MULTIPART (FORM-DATA)
// ==========================================
async fn parse_clock_multipart(
    mut multipart: Multipart,
) -> Result<(ClockPayload, Vec<u8>), AppError> {
    let mut lat: Option<Decimal> = None;
    let mut lon: Option<Decimal> = None;
    let mut alamat: Option<String> = None;
    let mut foto_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name == "latitude" {
            let text = field.text().await.unwrap_or_default();
            lat = Decimal::from_str(&text).ok();
        } else if name == "longitude" {
            let text = field.text().await.unwrap_or_default();
            lon = Decimal::from_str(&text).ok();
        } else if name == "alamat_absensi" {
            alamat = Some(field.text().await.unwrap_or_default());
        } else if name == "foto_selfie" {
            foto_bytes = Some(field.bytes().await.unwrap_or_default().to_vec());
        }
    }

    if lat.is_none() || lon.is_none() || foto_bytes.is_none() {
        return Err(AppError::Forbidden(
            "Data latitude, longitude, dan foto_selfie wajib diisi".to_string(),
        ));
    }

    let payload = ClockPayload {
        latitude: lat.unwrap(),
        longitude: lon.unwrap(),
        alamat_absensi: alamat,
        foto_absensi_path: None,     // Diisi nanti
        face_confidence_score: None, // Diisi nanti
        is_face_verified: None,      // Diisi nanti
    };

    Ok((payload, foto_bytes.unwrap()))
}

async fn proses_absensi(
    pool: &DbPool,
    pegawai_id: Uuid,
    mut payload: ClockPayload,
    foto_bytes: Vec<u8>,
    tipe: super::absensi_model::TipeAbsensi,
) -> Result<LogAbsensi, AppError> {
    // 1. Ambil path foto referensi dari DB (Contoh isi: "uploads/sdm/...")
    let path_file_db = repo::get_foto_profil_pegawai(pool, pegawai_id).await?;

    // Tambahkan "./" di depannya agar merujuk dengan tepat ke file lokal server
    let foto_ref_path = format!("./{}", path_file_db);

    // Baca file gambar referensi. Jika tidak ada file, ini akan trigger AppError::IoError (500)
    let ref_bytes = fs::read(&foto_ref_path).await?;

    // 2. Azure Face API Process
    let face_id_ref = detect_face_azure(ref_bytes).await?;
    let face_id_selfie = detect_face_azure(foto_bytes.clone()).await?;
    let (is_verified, confidence) = verify_face_azure(&face_id_ref, &face_id_selfie).await?;

    if !is_verified || confidence < 0.70 {
        // Threshold kemiripan 70%
        return Err(AppError::Forbidden(format!(
            "Absensi ditolak. Wajah tidak cocok (Kemiripan: {:.0}%)",
            confidence * 100.0
        )));
    }

    // 3. Simpan foto selfie ke folder server
    let ext = "jpg";
    let nama_file_selfie = format!("{}.{}", Uuid::new_v4(), ext);

    // Gunakan struktur yang sama dengan modul SDM: uploads/absensi/{pegawai_id}/
    let folder_simpan = format!("./uploads/absensi/{}", pegawai_id);
    let path_simpan = format!("{}/{}", folder_simpan, nama_file_selfie);

    // Pastikan folder untuk pegawai ini ada
    fs::create_dir_all(&folder_simpan).await?;
    fs::write(&path_simpan, foto_bytes).await?;

    // 4. Update Payload dengan path yang bisa disimpan ke DB
    // Format sama seperti `path_file` di tabel `dokumen_sdm`
    let path_db_selfie = format!("uploads/absensi/{}/{}", pegawai_id, nama_file_selfie);

    payload.foto_absensi_path = Some(path_db_selfie);
    payload.face_confidence_score = Some(confidence);
    payload.is_face_verified = Some(is_verified);

    // 5. Simpan ke Database
    if tipe == super::absensi_model::TipeAbsensi::ClockIn {
        repo::clock_in_repo(pool, pegawai_id, payload).await
    } else {
        repo::clock_out_repo(pool, pegawai_id, payload).await
    }
}

// ==========================================
// HANDLERS
// ==========================================

pub async fn clock_in_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    multipart: Multipart, // <--- Berubah dari Json
) -> Result<(StatusCode, Json<LogAbsensi>), AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;

    let (payload, foto_bytes) = parse_clock_multipart(multipart).await?;
    let log = proses_absensi(
        &pool,
        pegawai_id,
        payload,
        foto_bytes,
        super::absensi_model::TipeAbsensi::ClockIn,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(log)))
}

pub async fn clock_out_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    multipart: Multipart, // <--- Berubah dari Json
) -> Result<(StatusCode, Json<LogAbsensi>), AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;

    let (payload, foto_bytes) = parse_clock_multipart(multipart).await?;
    let log = proses_absensi(
        &pool,
        pegawai_id,
        payload,
        foto_bytes,
        super::absensi_model::TipeAbsensi::ClockOut,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(log)))
}

/// Handler Admin: Membuat atau mengoreksi rekap absensi harian
pub async fn create_rekap_manual_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<RekapManualPayload>,
) -> Result<(StatusCode, Json<RekapAbsensiHarian>), AppError> {
    let rekap = repo::create_rekap_manual_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(rekap)))
}

/// Handler Pegawai: Melihat rekap absensi bulanan milik sendiri
pub async fn get_my_rekap_absensi_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(filter): Query<RekapAbsensiFilter>,
) -> Result<Json<Vec<RekapAbsensiHarian>>, AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;
    let list = repo::get_my_rekap_absensi_repo(&pool, pegawai_id, filter).await?;
    Ok(Json(list))
}

/// Handler Pegawai: Melihat log clock-in/out untuk satu hari
pub async fn get_my_logs_for_day_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(filter): Query<LogDayFilter>, // <-- 2. UBAH TIPE DI SINI
) -> Result<Json<Vec<LogAbsensi>>, AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;
    let list = repo::get_my_logs_for_day_repo(&pool, pegawai_id, filter.tanggal).await?; // <-- 3. Gunakan filter.tanggal
    Ok(Json(list))
}

/// Handler Admin: Melihat rekap absensi bulanan semua pegawai
pub async fn get_all_rekap_absensi_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<RekapAbsensiFilter>,
) -> Result<Json<Vec<RekapAbsensiHarian>>, AppError> {
    let list = repo::get_all_rekap_absensi_repo(&pool, filter).await?;
    Ok(Json(list))
}
