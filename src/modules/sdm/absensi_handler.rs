// src/modules/sdm/absensi_handler.rs
use super::{
    absensi_model::{
        ClockPayload, LaporanAbsensiResponse, LaporanAbsensiRow, LaporanBulananFilter,
        LaporanHarianFilter, LogAbsensi, LogDayFilter, RekapAbsensiFilter, RekapAbsensiHarian,
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
            // 1. Eksekusi tembakan ke API Face++ (Liveness + Compare)
            let res = verify_face_faceplusplus_direct(msg.ref_bytes, msg.selfie_bytes).await;

            // 2. Kembalikan hasilnya ke user yang sedang menunggu (Pager)
            let _ = msg.reply_tx.send(res);

            // 3. JEDA PAKSA 1.1 Detik setelah selesai 1 antrean agar tidak terkena limit 1 QPS Face++
            tokio::time::sleep(Duration::from_millis(1100)).await;
        }
    });

    tx
});

// ==========================================
// HELPER LIVENESS FACE++ (ANTI-SPOOFING)
// ==========================================

async fn check_liveness_api(
    api_key: String,
    api_secret: String,
    selfie_bytes: Vec<u8>,
) -> Result<bool, AppError> {
    let endpoint = env::var("FACEPP_LIVENESS_ENDPOINT").unwrap_or_default();
    if endpoint.is_empty() {
        return Ok(true); // Bypass jika endpoint belum diset di .env
    }

    let client = reqwest::Client::new();
    let part = reqwest::multipart::Part::bytes(selfie_bytes)
        .file_name("liveness_selfie.jpg")
        .mime_str("image/jpeg")
        .map_err(|e| {
            AppError::AnyhowError(anyhow::anyhow!(
                "Gagal membaca foto selfie untuk liveness: {}",
                e
            ))
        })?;

    let form = reqwest::multipart::Form::new()
        .text("api_key", api_key)
        .text("api_secret", api_secret)
        .part("image_file", part);

    let res = client
        .post(&endpoint)
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            AppError::AnyhowError(anyhow::anyhow!(
                "Gagal menghubungi server Face++ Liveness: {}",
                e
            ))
        })?;

    let json: Value = res.json().await.map_err(|e| {
        AppError::AnyhowError(anyhow::anyhow!(
            "Gagal parse respons Face++ Liveness: {}",
            e
        ))
    })?;

    // Menangani Error dari Face++
    if let Some(err_msg) = json.get("error_message") {
        let err_str = err_msg.as_str().unwrap_or("Unknown");

        // GRACEFUL FALLBACK: Jika fitur Liveness tidak tersedia di paket Face++ (API_NOT_FOUND)
        // Kita otomatis bypass pengecekan ini agar karyawan tetap bisa absen.
        if err_str == "API_NOT_FOUND" {
            println!(
                "[WARNING] Face++ Liveness Endpoint API_NOT_FOUND. Mem-bypass pengecekan Liveness..."
            );
            return Ok(true);
        }

        return Err(AppError::Forbidden(format!(
            "Error dari Face++ Liveness: {}",
            err_str
        )));
    }

    // CATATAN PARSING:
    // Sesuaikan kunci (key) JSON di bawah ini dengan dokumentasi API Anti-Spoofing yang Anda gunakan.
    let is_fake = if let Some(liveness) = json.get("liveness") {
        let fake_score = liveness.get("fake").and_then(|v| v.as_f64()).unwrap_or(0.0);
        fake_score > 50.0
    } else if let Some(confidence) = json.get("confidence") {
        let liveness_confidence = confidence.as_f64().unwrap_or(0.0);
        liveness_confidence < 60.0
    } else if let Some(result_str) = json.get("result").and_then(|v| v.as_str()) {
        result_str.to_lowercase().contains("fake")
    } else {
        false
    };

    Ok(!is_fake)
}

// ==========================================
// HELPER FACE++ (DIRECT CALL UNTUK LIVENESS + COMPARE)
// ==========================================

async fn verify_face_faceplusplus_direct(
    ref_bytes: Vec<u8>,
    selfie_bytes: Vec<u8>,
) -> Result<(bool, f32), AppError> {
    let api_key = env::var("FACEPP_API_KEY").unwrap_or_default();
    let api_secret = env::var("FACEPP_API_SECRET").unwrap_or_default();

    if api_key.is_empty() || api_secret.is_empty() {
        return Ok((true, 0.99)); // Bypass saat development
    }

    // -----------------------------------------------------
    // 1. CEK LIVENESS (OPSIONAL BERDASARKAN .ENV)
    // -----------------------------------------------------
    let use_liveness = env::var("USE_FACE_LIVENESS_BE").unwrap_or_else(|_| "false".to_string());

    if use_liveness == "true" || use_liveness == "1" {
        let is_live =
            check_liveness_api(api_key.clone(), api_secret.clone(), selfie_bytes.clone()).await?;

        if !is_live {
            // Ditolak! Karyawan ketahuan pakai foto / layar HP / topeng
            return Err(AppError::Forbidden(
                "Absensi ditolak: Wajah terdeteksi tidak nyata (terindikasi menggunakan foto atau layar).".to_string(),
            ));
        }

        // SANGAT PENTING: Jeda 1.1 Detik agar tidak terkena limit 1 QPS saat lanjut ke endpoint Compare!
        tokio::time::sleep(Duration::from_millis(1100)).await;
    }

    // -----------------------------------------------------
    // 2. CEK KEMIRIPAN (COMPARE)
    // -----------------------------------------------------
    let endpoint_compare = env::var("FACEPP_ENDPOINT").unwrap_or_default();
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
        .post(&endpoint_compare)
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            AppError::AnyhowError(anyhow::anyhow!(
                "Gagal menghubungi server Face++ Compare: {}",
                e
            ))
        })?;

    let json: Value = res.json().await.map_err(|e| {
        AppError::AnyhowError(anyhow::anyhow!("Gagal parse respons Face++ Compare: {}", e))
    })?;

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
        let (reply_tx, reply_rx) = oneshot::channel();
        let msg = FaceQueueMessage {
            ref_bytes,
            selfie_bytes,
            reply_tx,
        };

        if FACE_QUEUE.send(msg).await.is_err() {
            return Err(AppError::AnyhowError(anyhow::anyhow!(
                "Sistem antrean AI sedang down"
            )));
        }

        match reply_rx.await {
            Ok(res) => res,
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
    let path_file_db = repo::get_foto_profil_pegawai(pool, pegawai_id).await?;
    let foto_ref_path = format!("./{}", path_file_db);
    let ref_bytes = fs::read(&foto_ref_path).await?;

    let (is_verified, confidence) = verify_face_faceplusplus(ref_bytes, foto_bytes.clone()).await?;

    if !is_verified || confidence < 0.70 {
        return Err(AppError::Forbidden(format!(
            "Absensi ditolak. Wajah tidak cocok (Kemiripan: {:.0}%)",
            confidence * 100.0
        )));
    }

    let ext = "jpg";
    let nama_file_selfie = format!("{}.{}", Uuid::new_v4(), ext);
    let folder_simpan = format!("./uploads/absensi/{}", pegawai_id);
    let path_simpan = format!("{}/{}", folder_simpan, nama_file_selfie);

    fs::create_dir_all(&folder_simpan).await?;
    fs::write(&path_simpan, foto_bytes).await?;

    let path_db_selfie = format!("uploads/absensi/{}/{}", pegawai_id, nama_file_selfie);
    payload.foto_absensi_path = Some(path_db_selfie);
    payload.face_confidence_score = Some(confidence);
    payload.is_face_verified = Some(is_verified);

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

/// HELPER FUNGSI: Mengonversi DB Row menjadi Response API + Menghitung Keterangan
fn kalkulasi_keterangan(row: &LaporanAbsensiRow) -> LaporanAbsensiResponse {
    // 1. Ambil env (bisa support format 07.30 maupun 07:30)
    let jam_masuk_str = env::var("JAM_MASUK_KERJA")
        .unwrap_or_else(|_| "07:30".to_string())
        .replace(".", ":");
    let jam_pulang_str = env::var("JAM_PULANG_KERJA")
        .unwrap_or_else(|_| "16:30".to_string())
        .replace(".", ":");

    // 2. Parsing text jadi struct Waktu
    let format = time::format_description::parse("[hour]:[minute]").unwrap();
    let jam_masuk = time::Time::parse(&jam_masuk_str, &format)
        .unwrap_or(time::Time::from_hms(7, 30, 0).unwrap());
    let jam_pulang = time::Time::parse(&jam_pulang_str, &format)
        .unwrap_or(time::Time::from_hms(16, 30, 0).unwrap());

    // Konversi jam ke satuan Menit agar mudah dihitung matematiknya
    let target_masuk_mnt = jam_masuk.hour() as i32 * 60 + jam_masuk.minute() as i32;
    let target_pulang_mnt = jam_pulang.hour() as i32 * 60 + jam_pulang.minute() as i32;

    let mut ket = Vec::new();
    let offset_wib = time::UtcOffset::from_hms(7, 0, 0).unwrap(); // Zona Waktu WIB

    // LOGIKA PENENTUAN KETERANGAN
    if row.clock_in.is_none() && row.clock_out.is_none() {
        let status_text = match row.status_harian.as_deref() {
            Some("Hadir") => "Lupa Absen Mesin (Direkap Manual: Hadir)".to_string(),
            Some(s) => format!("Keterangan: {}", s), // Misal: Sakit, Cuti, Ijin
            None => "Tidak Absen (Alpa)".to_string(),
        };
        ket.push(status_text);
    } else {
        // Cek Clock In (Keterlambatan)
        if let Some(in_dt) = row.clock_in {
            let waktu_wib = in_dt.to_offset(offset_wib).time();
            let real_masuk_mnt = waktu_wib.hour() as i32 * 60 + waktu_wib.minute() as i32;

            if real_masuk_mnt > target_masuk_mnt {
                let telat = real_masuk_mnt - target_masuk_mnt;
                ket.push(format!("Terlambat {} jam {} menit", telat / 60, telat % 60));
            }
        } else {
            ket.push("Tidak ada Clock In".to_string());
        }

        // Cek Clock Out (Pulang Cepat & Lembur)
        if let Some(out_dt) = row.clock_out {
            let waktu_wib = out_dt.to_offset(offset_wib).time();
            let real_pulang_mnt = waktu_wib.hour() as i32 * 60 + waktu_wib.minute() as i32;

            if real_pulang_mnt < target_pulang_mnt {
                let cepat = target_pulang_mnt - real_pulang_mnt;
                ket.push(format!(
                    "Pulang Cepat {} jam {} menit",
                    cepat / 60,
                    cepat % 60
                ));
            } else {
                let over = real_pulang_mnt - target_pulang_mnt;
                if over >= 60 {
                    // Lembur dihitung jika minimal lewat 1 Jam (60 menit)
                    ket.push(format!("Lembur {} jam {} menit", over / 60, over % 60));
                }
            }
        } else {
            ket.push("Tidak ada Clock Out".to_string());
        }
    }

    let keterangan_final = if ket.is_empty() {
        "Hadir Tepat Waktu".to_string()
    } else {
        ket.join(", ")
    };

    LaporanAbsensiResponse {
        pegawai_id: row.pegawai_id,
        nama_pegawai: row.nama_pegawai.clone(),
        tanggal: row.tanggal,
        clock_in: row.clock_in,
        clock_out: row.clock_out,
        keterangan: keterangan_final,
        foto_absensi_path_in: row.foto_absensi_path_in.clone(),
        foto_absensi_path_out: row.foto_absensi_path_out.clone(),
        latitude_in: row.latitude_in,
        longitude_in: row.longitude_in,
        latitude_out: row.latitude_out,
        longitude_out: row.longitude_out,
    }
}

// =====================================
// ENDPOINT HANDLERS
// =====================================

pub async fn laporan_absensi_harian_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<LaporanHarianFilter>,
) -> Result<Json<Vec<LaporanAbsensiResponse>>, AppError> {
    let rows = repo::get_laporan_harian_repo(&pool, filter.tanggal).await?;

    // Petakan raw data menjadi respons JSON yang sudah melewati logika '.env'
    let responses: Vec<LaporanAbsensiResponse> = rows.iter().map(kalkulasi_keterangan).collect();
    Ok(Json(responses))
}

pub async fn laporan_absensi_bulanan_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<LaporanBulananFilter>,
) -> Result<Json<Vec<LaporanAbsensiResponse>>, AppError> {
    let rows = repo::get_laporan_bulanan_repo(&pool, filter.pegawai_id, filter.bulan, filter.tahun)
        .await?;

    let responses: Vec<LaporanAbsensiResponse> = rows.iter().map(kalkulasi_keterangan).collect();
    Ok(Json(responses))
}
