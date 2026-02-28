// src/modules/sdm/absensi_handler.rs
use super::{
    absensi_model::{
        ClockPayload, LogAbsensi, LogDayFilter, RekapAbsensiFilter, RekapAbsensiHarian,
        RekapManualPayload, TipeAbsensi,
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

// Import untuk sistem Queue
use once_cell::sync::Lazy;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

// ==========================================
// SISTEM ANTREAN (QUEUE) FACE++
// ==========================================

// Struktur pesan yang akan masuk ke dalam antrean
struct FaceQueueMessage {
    ref_bytes: Vec<u8>,
    selfie_bytes: Vec<u8>,
    reply_tx: oneshot::Sender<Result<(bool, f32), AppError>>,
}

// Inisialisasi antrean secara global (hanya dibuat 1 kali saat aplikasi berjalan)
static FACE_QUEUE: Lazy<mpsc::Sender<FaceQueueMessage>> = Lazy::new(|| {
    // Kapasitas antrean maksimal 100 orang di waktu bersamaan
    let (tx, mut rx) = mpsc::channel::<FaceQueueMessage>(100);

    // Background worker (Kasir)
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            // 1. Eksekusi tembakan ke Face++
            let res = verify_face_faceplusplus_direct(msg.ref_bytes, msg.selfie_bytes).await;

            // 2. Kembalikan hasilnya ke user yang sedang menunggu (Pager)
            let _ = msg.reply_tx.send(res);

            // 3. JEDA PAKSA 1.1 Detik agar tidak terkena limit 1 QPS Face++
            tokio::time::sleep(Duration::from_millis(1100)).await;
        }
    });

    tx
});

// ==========================================
// HELPER FACE++ (DIRECT CALL)
// ==========================================

// Ini adalah kode asli tanpa antrean
async fn verify_face_faceplusplus_direct(
    ref_bytes: Vec<u8>,
    selfie_bytes: Vec<u8>,
) -> Result<(bool, f32), AppError> {
    let api_key = env::var("FACEPP_API_KEY").unwrap_or_default();
    let api_secret = env::var("FACEPP_API_SECRET").unwrap_or_default();
    let endpoint = env::var("FACEPP_ENDPOINT").unwrap_or_default();

    if api_key.is_empty() || api_secret.is_empty() {
        return Ok((true, 0.99)); // Bypass saat development
    }

    let client = reqwest::Client::new();

    let part1 = reqwest::multipart::Part::bytes(ref_bytes)
        .file_name("referensi.jpg")
        .mime_str("image/jpeg")
        .map_err(|e| AppError::AnyhowError(anyhow::anyhow!("Gagal membaca foto ref: {}", e)))?;

    let part2 = reqwest::multipart::Part::bytes(selfie_bytes)
        .file_name("selfie.jpg")
        .mime_str("image/jpeg")
        .map_err(|e| AppError::AnyhowError(anyhow::anyhow!("Gagal membaca foto selfie: {}", e)))?;

    let form = reqwest::multipart::Form::new()
        .text("api_key", api_key)
        .text("api_secret", api_secret)
        .part("image_file1", part1)
        .part("image_file2", part2);

    let res = client
        .post(&endpoint)
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            AppError::AnyhowError(anyhow::anyhow!("Gagal menghubungi server Face++: {}", e))
        })?;

    let json: Value = res
        .json()
        .await
        .map_err(|e| AppError::AnyhowError(anyhow::anyhow!("Gagal parse respons Face++: {}", e)))?;

    if let Some(err_msg) = json.get("error_message") {
        return Err(AppError::Forbidden(format!(
            "Error dari Face++: {}",
            err_msg.as_str().unwrap_or("Unknown")
        )));
    }

    let confidence = json
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;
    let is_identical = confidence >= 70.0;

    Ok((is_identical, confidence / 100.0))
}

// ==========================================
// ROUTER LOGIKA (PILIH QUEUE ATAU DIRECT)
// ==========================================

async fn verify_face_faceplusplus(
    ref_bytes: Vec<u8>,
    selfie_bytes: Vec<u8>,
) -> Result<(bool, f32), AppError> {
    // Cek konfigurasi dari .env
    let use_queue = env::var("USE_FACE_QUEUE").unwrap_or_else(|_| "false".to_string());

    if use_queue == "true" || use_queue == "1" {
        // --- JALUR ANTREAN ---
        // Buat Pager (oneshot channel) untuk menunggu jawaban
        let (reply_tx, reply_rx) = oneshot::channel();

        // Buat paket pesan
        let msg = FaceQueueMessage {
            ref_bytes,
            selfie_bytes,
            reply_tx,
        };

        // Masukkan ke keranjang antrean
        if FACE_QUEUE.send(msg).await.is_err() {
            return Err(AppError::AnyhowError(anyhow::anyhow!(
                "Sistem antrean AI sedang down"
            )));
        }

        // Tunggu Pager berbunyi (await)
        match reply_rx.await {
            Ok(res) => res, // Mengembalikan hasil (Ok atau Err dari Face++)
            Err(_) => Err(AppError::AnyhowError(anyhow::anyhow!(
                "Gagal menerima respon dari antrean AI"
            ))),
        }
    } else {
        // --- JALUR LANGSUNG (TANPA ANTREAN) ---
        verify_face_faceplusplus_direct(ref_bytes, selfie_bytes).await
    }
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
        foto_absensi_path: None,
        face_confidence_score: None,
        is_face_verified: None,
    };

    Ok((payload, foto_bytes.unwrap()))
}

async fn proses_absensi(
    pool: &DbPool,
    pegawai_id: Uuid,
    mut payload: ClockPayload,
    foto_bytes: Vec<u8>,
    tipe: TipeAbsensi,
) -> Result<LogAbsensi, AppError> {
    // 1. Ambil path foto referensi dari DB
    let path_file_db = repo::get_foto_profil_pegawai(pool, pegawai_id).await?;
    let foto_ref_path = format!("./{}", path_file_db);
    let ref_bytes = fs::read(&foto_ref_path).await?;

    // 2. FACE++ Process (Akan memilih jalur Queue atau Direct otomatis)
    let (is_verified, confidence) = verify_face_faceplusplus(ref_bytes, foto_bytes.clone()).await?;

    if !is_verified || confidence < 0.70 {
        return Err(AppError::Forbidden(format!(
            "Absensi ditolak. Wajah tidak cocok (Kemiripan: {:.0}%)",
            confidence * 100.0
        )));
    }

    // 3. Simpan foto selfie ke folder server
    let ext = "jpg";
    let nama_file_selfie = format!("{}.{}", Uuid::new_v4(), ext);
    let folder_simpan = format!("./uploads/absensi/{}", pegawai_id);
    let path_simpan = format!("{}/{}", folder_simpan, nama_file_selfie);

    fs::create_dir_all(&folder_simpan).await?;
    fs::write(&path_simpan, foto_bytes).await?;

    // 4. Update Payload
    let path_db_selfie = format!("uploads/absensi/{}/{}", pegawai_id, nama_file_selfie);
    payload.foto_absensi_path = Some(path_db_selfie);
    payload.face_confidence_score = Some(confidence);
    payload.is_face_verified = Some(is_verified);

    // 5. Simpan ke Database
    if tipe == TipeAbsensi::ClockIn {
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
    multipart: Multipart,
) -> Result<(StatusCode, Json<LogAbsensi>), AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;

    let (payload, foto_bytes) = parse_clock_multipart(multipart).await?;
    let log = proses_absensi(&pool, pegawai_id, payload, foto_bytes, TipeAbsensi::ClockIn).await?;

    Ok((StatusCode::CREATED, Json(log)))
}

pub async fn clock_out_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<LogAbsensi>), AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;

    let (payload, foto_bytes) = parse_clock_multipart(multipart).await?;
    let log = proses_absensi(
        &pool,
        pegawai_id,
        payload,
        foto_bytes,
        TipeAbsensi::ClockOut,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(log)))
}

pub async fn create_rekap_manual_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<RekapManualPayload>,
) -> Result<(StatusCode, Json<RekapAbsensiHarian>), AppError> {
    let rekap = repo::create_rekap_manual_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(rekap)))
}

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

pub async fn get_my_logs_for_day_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(filter): Query<LogDayFilter>,
) -> Result<Json<Vec<LogAbsensi>>, AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;
    let list = repo::get_my_logs_for_day_repo(&pool, pegawai_id, filter.tanggal).await?;
    Ok(Json(list))
}

pub async fn get_all_rekap_absensi_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<RekapAbsensiFilter>,
) -> Result<Json<Vec<RekapAbsensiHarian>>, AppError> {
    let list = repo::get_all_rekap_absensi_repo(&pool, filter).await?;
    Ok(Json(list))
}
