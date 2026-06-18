// src/modules/sdm/absensi_handler.rs
use super::{
    absensi_model::{
        BiometrikStatusDetail, ClockPayload, ClockResponseFlat, LaporanAbsensiResponse,
        LaporanAbsensiRow, LaporanBulananFilter, LaporanBulananResponse, LaporanHarianFilter,
        LogAbsensi, LogDayFilter, RekapAbsensiFilter, RekapAbsensiHarian, RekapManualPayload,
        StatusAbsensi, TipeAbsensi,
    },
    absensi_repo as repo, absensi_wajah_repo, face_compare_client,
    face_compare_client::FaceCompareProvider,
    repo as pegawai_repo,
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

use once_cell::sync::Lazy;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

// ==========================================
// SISTEM ANTREAN (QUEUE) FACE++
// ==========================================

struct FaceQueueMessage {
    ref_bytes: Vec<u8>,
    selfie_bytes: Vec<u8>,
    reply_tx: oneshot::Sender<Result<(bool, f32), AppError>>,
}

static FACE_QUEUE: Lazy<mpsc::Sender<FaceQueueMessage>> = Lazy::new(|| {
    let (tx, mut rx) = mpsc::channel::<FaceQueueMessage>(100);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let res = verify_face_faceplusplus_direct(msg.ref_bytes, msg.selfie_bytes).await;
            let _ = msg.reply_tx.send(res);
            tokio::time::sleep(Duration::from_millis(1100)).await;
        }
    });

    tx
});

// ==========================================
// HELPER LIVENESS & FACE++
// ==========================================

async fn check_liveness_api(
    api_key: String,
    api_secret: String,
    selfie_bytes: Vec<u8>,
) -> Result<bool, AppError> {
    let endpoint = env::var("FACEPP_LIVENESS_ENDPOINT").unwrap_or_default();
    if endpoint.is_empty() {
        return Ok(true);
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

    if let Some(err_msg) = json.get("error_message") {
        let err_str = err_msg.as_str().unwrap_or("Unknown");
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

async fn verify_face_faceplusplus_direct(
    ref_bytes: Vec<u8>,
    selfie_bytes: Vec<u8>,
) -> Result<(bool, f32), AppError> {
    let api_key = env::var("FACEPP_API_KEY").unwrap_or_default();
    let api_secret = env::var("FACEPP_API_SECRET").unwrap_or_default();

    if api_key.is_empty() || api_secret.is_empty() {
        return Ok((true, 0.99));
    }

    let use_liveness = env::var("USE_FACE_LIVENESS_BE").unwrap_or_else(|_| "false".to_string());
    if use_liveness == "true" || use_liveness == "1" {
        let is_live =
            check_liveness_api(api_key.clone(), api_secret.clone(), selfie_bytes.clone()).await?;
        if !is_live {
            return Err(AppError::Forbidden("Absensi ditolak: Wajah terdeteksi tidak nyata (terindikasi menggunakan foto atau layar).".to_string()));
        }
        tokio::time::sleep(Duration::from_millis(1100)).await;
    }

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
    Ok((confidence >= 70.0, confidence / 100.0))
}

async fn verify_face_faceplusplus(
    ref_bytes: Vec<u8>,
    selfie_bytes: Vec<u8>,
) -> Result<(bool, f32), AppError> {
    let use_queue = env::var("USE_FACE_QUEUE").unwrap_or_else(|_| "false".to_string());
    if use_queue == "true" || use_queue == "1" {
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
        reply_rx.await.unwrap_or_else(|_| {
            Err(AppError::AnyhowError(anyhow::anyhow!(
                "Gagal menerima respon dari antrean AI"
            )))
        })
    } else {
        verify_face_faceplusplus_direct(ref_bytes, selfie_bytes).await
    }
}

async fn verify_face_for_attendance(
    pool: &DbPool,
    pegawai_id: Uuid,
    reference_bytes: Vec<u8>,
    reference_embedding: Option<Value>,
    selfie_bytes: Vec<u8>,
) -> Result<(bool, f32, Option<Value>), AppError> {
    match face_compare_client::provider_from_env() {
        FaceCompareProvider::OpenCvSFace => {
            let embedding = match reference_embedding {
                Some(embedding) => embedding,
                None => {
                    let embedding =
                        face_compare_client::extract_embedding(reference_bytes.clone()).await?;
                    absensi_wajah_repo::update_reference_embedding_repo(
                        pool,
                        pegawai_id,
                        embedding.clone(),
                    )
                    .await?;
                    embedding
                }
            };

            let result = face_compare_client::verify_embedding(&embedding, selfie_bytes).await?;
            Ok((result.is_match, result.similarity, result.probe_embedding))
        }
        FaceCompareProvider::Disabled => Ok((true, 0.99, None)),
        FaceCompareProvider::FacePlusPlus => {
            let (is_verified, confidence) =
                verify_face_faceplusplus(reference_bytes, selfie_bytes).await?;
            Ok((is_verified, confidence, None))
        }
    }
}

// ==========================================
// HELPER JARINGAN & LOKASI
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

    Ok((
        ClockPayload {
            latitude: lat.unwrap(),
            longitude: lon.unwrap(),
            alamat_absensi: alamat,
            foto_absensi_path: None,
            face_confidence_score: None,
            is_face_verified: None,
            face_absensi_embedding: None,
        },
        foto_bytes.unwrap(),
    ))
}

fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r_earth = 6371000.0;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r_earth * c
}

// ==========================================
// CORE ABSENSI LOGIC
// ==========================================

async fn proses_absensi(
    pool: &DbPool,
    pegawai_id: Uuid,
    mut payload: ClockPayload,
    foto_bytes: Vec<u8>,
    tipe: TipeAbsensi,
) -> Result<ClockResponseFlat, AppError> {
    let today = (time::OffsetDateTime::now_utc() + time::Duration::hours(7)).date();
    let ijin_lokasi = repo::cek_ijin_lokasi_aktif(pool, pegawai_id, today).await?;

    let kampus_lat: f64 = env::var("KAMPUS_LATITUDE")
        .unwrap_or_else(|_| "-7.336465677499996".to_string())
        .parse()
        .unwrap_or(-7.336465677499996);
    let kampus_lon: f64 = env::var("KAMPUS_LONGITUDE")
        .unwrap_or_else(|_| "108.15347757116479".to_string())
        .parse()
        .unwrap_or(108.15347757116479);
    let kampus_radius: f64 = env::var("KAMPUS_RADIUS_METER")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap_or(100.0);

    let lat_user = payload.latitude.to_string().parse::<f64>().unwrap_or(0.0);
    let lon_user = payload.longitude.to_string().parse::<f64>().unwrap_or(0.0);

    let dist = haversine_distance(kampus_lat, kampus_lon, lat_user, lon_user);
    let mut pesan_notifikasi = "Absensi berhasil dicatat.".to_string();

    if dist > kampus_radius {
        if let Some(jenis_ijin) = ijin_lokasi {
            pesan_notifikasi = format!(
                "Absen berhasil. Anda diizinkan absen di luar kampus karena berstatus {}.",
                jenis_ijin
            );
        } else {
            return Err(AppError::Forbidden(format!(
                "Absensi ditolak. Anda berada {:.0} meter dari kampus. Anda harus berada di dalam radius {} meter atau memiliki Ijin Dinas/WFH.",
                dist, kampus_radius
            )));
        }
    }

    let foto_profil = repo::get_foto_profil_pegawai(pool, pegawai_id).await?;
    let foto_ref_path = format!("./{}", foto_profil.path);
    let ref_bytes = fs::read(&foto_ref_path).await?;

    let (is_verified, confidence, absensi_embedding) = verify_face_for_attendance(
        pool,
        pegawai_id,
        ref_bytes,
        foto_profil.reference_embedding,
        foto_bytes.clone(),
    )
    .await?;

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

    payload.foto_absensi_path = Some(format!(
        "uploads/absensi/{}/{}",
        pegawai_id, nama_file_selfie
    ));
    payload.face_confidence_score = Some(confidence);
    payload.is_face_verified = Some(is_verified);
    payload.face_absensi_embedding = absensi_embedding;

    let log = if tipe == TipeAbsensi::ClockIn {
        repo::clock_in_repo(pool, pegawai_id, payload).await?
    } else {
        repo::clock_out_repo(pool, pegawai_id, payload).await?
    };

    Ok(ClockResponseFlat {
        pesan_notifikasi: Some(pesan_notifikasi),
        data: log,
    })
}

// =====================================
// ENDPOINT HANDLERS
// =====================================

pub async fn clock_in_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<ClockResponseFlat>), AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;

    let (payload, foto_bytes) = parse_clock_multipart(multipart).await?;
    let response_data =
        proses_absensi(&pool, pegawai_id, payload, foto_bytes, TipeAbsensi::ClockIn).await?;

    Ok((StatusCode::CREATED, Json(response_data)))
}

pub async fn clock_out_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<ClockResponseFlat>), AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;

    let (payload, foto_bytes) = parse_clock_multipart(multipart).await?;
    let response_data = proses_absensi(
        &pool,
        pegawai_id,
        payload,
        foto_bytes,
        TipeAbsensi::ClockOut,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(response_data)))
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

    let bulan = filter.bulan;
    let tahun = filter.tahun;
    let mut list = repo::get_my_rekap_absensi_repo(&pool, pegawai_id, filter).await?;

    // --- INJECT DYNAMIC IJIN & CUTI ---
    let bulan_u8: u8 = bulan.try_into().unwrap_or(1);
    let bulan_enum = time::Month::try_from(bulan_u8).unwrap_or(time::Month::January);
    let start_date = time::Date::from_calendar_date(tahun, bulan_enum, 1)
        .map_err(|_| AppError::BadRequest("Tanggal tidak valid".to_string()))?;
    let end_date = (start_date + time::Duration::days(32))
        .replace_day(1)
        .unwrap()
        - time::Duration::days(1);

    // 1. Fetch Ijin (Sakit, Urusan Keluarga, WFH, dll)
    let ijins = sqlx::query!(
        r#"
        SELECT kategori::TEXT as "kategori!", tanggal_mulai, tanggal_selesai, alasan 
        FROM pengajuan_ijin 
        WHERE pegawai_id = $1 AND status = 'Disetujui' 
          AND tanggal_mulai <= $3 AND tanggal_selesai >= $2
        "#,
        pegawai_id,
        start_date,
        end_date
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    for ijin in ijins {
        let mut curr = ijin.tanggal_mulai;
        while curr <= ijin.tanggal_selesai {
            if curr >= start_date && curr <= end_date {
                // Pastikan belum ada rekap di tanggal ini agar tidak double
                if !list.iter().any(|r| r.tanggal == curr) {
                    let status_absensi = match ijin.kategori.as_str() {
                        "Sakit" => StatusAbsensi::Sakit,
                        "WFH" | "Dinas Luar" => StatusAbsensi::Hadir,
                        _ => StatusAbsensi::Ijin,
                    };
                    list.push(RekapAbsensiHarian {
                        id: Uuid::new_v4(), // ID virtual untuk response
                        pegawai_id,
                        tanggal: curr,
                        status: status_absensi,
                        keterangan: Some(format!("{} - {}", ijin.kategori, ijin.alasan)),
                    });
                }
            }
            curr = curr + time::Duration::days(1);
        }
    }

    // 2. Fetch Cuti
    let cutis = sqlx::query!(
        r#"
        SELECT kategori::TEXT as "kategori!", tanggal_mulai, tanggal_selesai, alasan 
        FROM pengajuan_cuti 
        WHERE pegawai_id = $1 AND status = 'Disetujui' 
          AND tanggal_mulai <= $3 AND tanggal_selesai >= $2
        "#,
        pegawai_id,
        start_date,
        end_date
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    for cuti in cutis {
        let mut curr = cuti.tanggal_mulai;
        while curr <= cuti.tanggal_selesai {
            if curr >= start_date && curr <= end_date {
                if !list.iter().any(|r| r.tanggal == curr) {
                    list.push(RekapAbsensiHarian {
                        id: Uuid::new_v4(),
                        pegawai_id,
                        tanggal: curr,
                        status: StatusAbsensi::Cuti,
                        keterangan: Some(format!("{} - {}", cuti.kategori, cuti.alasan)),
                    });
                }
            }
            curr = curr + time::Duration::days(1);
        }
    }

    // Urutkan kembali berdasarkan tanggal karena data baru saja di-inject
    list.sort_by(|a, b| a.tanggal.cmp(&b.tanggal));

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
    let bulan = filter.bulan;
    let tahun = filter.tahun;
    let pegawai_id_filter = filter.pegawai_id;

    let mut list = repo::get_all_rekap_absensi_repo(&pool, filter).await?;

    // --- INJECT DYNAMIC IJIN & CUTI UNTUK SEMUA PEGAWAI ---
    let bulan_u8: u8 = bulan.try_into().unwrap_or(1);
    let bulan_enum = time::Month::try_from(bulan_u8).unwrap_or(time::Month::January);
    let start_date = time::Date::from_calendar_date(tahun, bulan_enum, 1)
        .map_err(|_| AppError::BadRequest("Tanggal tidak valid".to_string()))?;
    let end_date = (start_date + time::Duration::days(32))
        .replace_day(1)
        .unwrap()
        - time::Duration::days(1);

    #[derive(Debug, sqlx::FromRow)]
    struct PengajuanRow {
        pegawai_id: Uuid,
        kategori: String,
        tanggal_mulai: time::Date,
        tanggal_selesai: time::Date,
        alasan: String,
    }

    // 1. Query Ijin
    let mut ijin_query = sqlx::QueryBuilder::new(
        "SELECT pegawai_id, kategori::TEXT as kategori, tanggal_mulai, tanggal_selesai, alasan FROM pengajuan_ijin WHERE status = 'Disetujui' AND tanggal_mulai <= ",
    );
    ijin_query.push_bind(end_date);
    ijin_query.push(" AND tanggal_selesai >= ");
    ijin_query.push_bind(start_date);

    if let Some(pid) = pegawai_id_filter {
        ijin_query.push(" AND pegawai_id = ");
        ijin_query.push_bind(pid);
    }

    let ijins: Vec<PengajuanRow> = ijin_query
        .build_query_as()
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    for ijin in ijins {
        let mut curr = ijin.tanggal_mulai;
        while curr <= ijin.tanggal_selesai {
            if curr >= start_date && curr <= end_date {
                if !list
                    .iter()
                    .any(|r| r.tanggal == curr && r.pegawai_id == ijin.pegawai_id)
                {
                    let status_absensi = match ijin.kategori.as_str() {
                        "Sakit" => StatusAbsensi::Sakit,
                        "WFH" | "Dinas Luar" => StatusAbsensi::Hadir,
                        _ => StatusAbsensi::Ijin,
                    };
                    list.push(RekapAbsensiHarian {
                        id: Uuid::new_v4(), // ID virtual
                        pegawai_id: ijin.pegawai_id,
                        tanggal: curr,
                        status: status_absensi,
                        keterangan: Some(format!("{} - {}", ijin.kategori, ijin.alasan)),
                    });
                }
            }
            curr = curr + time::Duration::days(1);
        }
    }

    // 2. Query Cuti
    let mut cuti_query = sqlx::QueryBuilder::new(
        "SELECT pegawai_id, kategori::TEXT as kategori, tanggal_mulai, tanggal_selesai, alasan FROM pengajuan_cuti WHERE status = 'Disetujui' AND tanggal_mulai <= ",
    );
    cuti_query.push_bind(end_date);
    cuti_query.push(" AND tanggal_selesai >= ");
    cuti_query.push_bind(start_date);

    if let Some(pid) = pegawai_id_filter {
        cuti_query.push(" AND pegawai_id = ");
        cuti_query.push_bind(pid);
    }

    let cutis: Vec<PengajuanRow> = cuti_query
        .build_query_as()
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    for cuti in cutis {
        let mut curr = cuti.tanggal_mulai;
        while curr <= cuti.tanggal_selesai {
            if curr >= start_date && curr <= end_date {
                if !list
                    .iter()
                    .any(|r| r.tanggal == curr && r.pegawai_id == cuti.pegawai_id)
                {
                    list.push(RekapAbsensiHarian {
                        id: Uuid::new_v4(),
                        pegawai_id: cuti.pegawai_id,
                        tanggal: curr,
                        status: StatusAbsensi::Cuti,
                        keterangan: Some(format!("{} - {}", cuti.kategori, cuti.alasan)),
                    });
                }
            }
            curr = curr + time::Duration::days(1);
        }
    }

    list.sort_by(|a, b| a.tanggal.cmp(&b.tanggal));

    Ok(Json(list))
}

/// HELPER FUNGSI: Mengonversi DB Row menjadi Response API + Menghitung Keterangan, Menit, & Lokasi
fn kalkulasi_keterangan(row: &LaporanAbsensiRow) -> LaporanAbsensiResponse {
    let jam_masuk_str = env::var("JAM_MASUK_KERJA")
        .unwrap_or_else(|_| "07:30".to_string())
        .replace(".", ":");
    let jam_pulang_str = env::var("JAM_PULANG_KERJA")
        .unwrap_or_else(|_| "16:30".to_string())
        .replace(".", ":");
    let toleransi_str =
        env::var("TOLERANSI_KETERLAMBATAN_PERHARI").unwrap_or_else(|_| "30".to_string());

    let kampus_lat: f64 = env::var("KAMPUS_LATITUDE")
        .unwrap_or_else(|_| "-7.336465677499996".to_string())
        .parse()
        .unwrap_or(-7.336465677499996);
    let kampus_lon: f64 = env::var("KAMPUS_LONGITUDE")
        .unwrap_or_else(|_| "108.15347757116479".to_string())
        .parse()
        .unwrap_or(108.15347757116479);
    let kampus_radius: f64 = env::var("KAMPUS_RADIUS_METER")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap_or(100.0);

    let format = time::format_description::parse("[hour]:[minute]").unwrap();
    let jam_masuk = time::Time::parse(&jam_masuk_str, &format)
        .unwrap_or(time::Time::from_hms(7, 30, 0).unwrap());
    let jam_pulang = time::Time::parse(&jam_pulang_str, &format)
        .unwrap_or(time::Time::from_hms(16, 30, 0).unwrap());
    let toleransi_mnt: i32 = toleransi_str.parse().unwrap_or(30);

    let target_masuk_mnt = jam_masuk.hour() as i32 * 60 + jam_masuk.minute() as i32;
    let target_pulang_mnt = jam_pulang.hour() as i32 * 60 + jam_pulang.minute() as i32;

    let offset_wib = time::UtcOffset::from_hms(7, 0, 0).unwrap();

    let mut actual_in = row.clock_in;
    let mut actual_out = row.clock_out;

    if actual_in.is_some() && actual_out.is_none() {
        let waktu_wib = actual_in.unwrap().to_offset(offset_wib).time();
        let mnt = waktu_wib.hour() as i32 * 60 + waktu_wib.minute() as i32;
        if mnt > target_pulang_mnt {
            actual_in = None;
        }
    }
    if actual_out.is_some() && actual_in.is_none() {
        let waktu_wib = actual_out.unwrap().to_offset(offset_wib).time();
        let mnt = waktu_wib.hour() as i32 * 60 + waktu_wib.minute() as i32;
        if mnt < target_masuk_mnt {
            actual_out = None;
        }
    }

    let mut ket = Vec::new();
    let mut telat_aktual = 0;
    let mut telat_toleransi = 0;
    let mut lembur_aktual = 0;

    // --- PERBAIKAN LOGIKA REKAP MANUAL & IJIN OTOMATIS ---
    let manual_status = row.status_harian.as_deref();
    let ijin_kategori = row.ijin_kategori.as_deref();
    let is_remote_allowed = matches!(ijin_kategori, Some("WFH") | Some("Dinas Luar"));

    if actual_in.is_none() && actual_out.is_none() {
        // Karyawan tidak absen sama sekali hari ini
        if let Some(manual) = manual_status {
            if manual == "Hadir" {
                ket.push("Lupa Absen (Direkap Manual: Hadir)".to_string());
            } else {
                ket.push(format!("Keterangan: {}", manual)); // Misal: Keterangan: Cuti
            }
        } else if let Some(ijin) = ijin_kategori {
            if is_remote_allowed {
                ket.push(format!("Lupa Absen ({})", ijin)); // WFH tapi ga absen foto
            } else {
                ket.push(format!("Keterangan: {}", ijin)); // Otomatis terbaca: Keterangan: Sakit / Urusan Keluarga
            }
        } else {
            ket.push("Tidak Absen (Alpa)".to_string());
        }
    } else {
        // Karyawan melakukan Absen Mesin (Ada data jam masuk/pulang)

        // Beri Catatan jika mereka absen mesin tapi sebenarnya sedang Cuti/Sakit
        if let Some(manual) = manual_status {
            if manual != "Hadir" {
                ket.push(format!("(Status Rekap: {})", manual));
            }
        } else if let Some(ijin) = ijin_kategori {
            if !is_remote_allowed {
                ket.push(format!("(Status Ijin: {})", ijin));
            }
        }

        // --- Cek Clock In ---
        if let Some(in_dt) = actual_in {
            let waktu_wib = in_dt.to_offset(offset_wib).time();
            let real_masuk_mnt = waktu_wib.hour() as i32 * 60 + waktu_wib.minute() as i32;

            if real_masuk_mnt > target_masuk_mnt {
                telat_aktual = real_masuk_mnt - target_masuk_mnt;
                telat_toleransi = if telat_aktual > toleransi_mnt {
                    telat_aktual - toleransi_mnt
                } else {
                    0
                };
                ket.push(format!(
                    "Terlambat {} jam {} menit (Dihitung: {} jam {} menit)",
                    telat_aktual / 60,
                    telat_aktual % 60,
                    telat_toleransi / 60,
                    telat_toleransi % 60
                ));
            }

            if let (Some(lat_dec), Some(lon_dec)) = (row.latitude_in, row.longitude_in) {
                if is_remote_allowed {
                    ket.push(format!("Clock In ({})", ijin_kategori.unwrap_or("Remote")));
                } else {
                    let lat_f = lat_dec.to_string().parse::<f64>().unwrap_or(0.0);
                    let lon_f = lon_dec.to_string().parse::<f64>().unwrap_or(0.0);
                    if haversine_distance(kampus_lat, kampus_lon, lat_f, lon_f) > kampus_radius {
                        ket.push("Clock In diluar kampus".to_string());
                    }
                }
            }
        } else {
            ket.push("Tidak Absen Masuk".to_string());
        }

        // --- Cek Clock Out ---
        if let Some(out_dt) = actual_out {
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
                    lembur_aktual = over;
                    ket.push(format!("Lembur {} jam {} menit", over / 60, over % 60));
                }
            }

            if let (Some(lat_dec), Some(lon_dec)) = (row.latitude_out, row.longitude_out) {
                if is_remote_allowed {
                    ket.push(format!("Clock Out ({})", ijin_kategori.unwrap_or("Remote")));
                } else {
                    let lat_f = lat_dec.to_string().parse::<f64>().unwrap_or(0.0);
                    let lon_f = lon_dec.to_string().parse::<f64>().unwrap_or(0.0);
                    if haversine_distance(kampus_lat, kampus_lon, lat_f, lon_f) > kampus_radius {
                        ket.push("Clock Out diluar kampus".to_string());
                    }
                }
            }
        } else {
            ket.push("Tidak Absen Pulang".to_string());
        }
    }

    let keterangan_final = if ket.is_empty() {
        if is_remote_allowed {
            format!("Hadir Tepat Waktu ({})", ijin_kategori.unwrap_or("Remote"))
        } else {
            "Hadir Tepat Waktu".to_string()
        }
    } else {
        ket.join(", ")
    };

    LaporanAbsensiResponse {
        pegawai_id: row.pegawai_id,
        nama_pegawai: row.nama_pegawai.clone(),
        tanggal: row.tanggal,
        clock_in: actual_in,
        clock_out: actual_out,
        keterangan: keterangan_final,
        terlambat_menit: telat_aktual,
        terlambat_toleransi_menit: telat_toleransi,
        lembur_menit: lembur_aktual,
        foto_absensi_path_in: row.foto_absensi_path_in.clone(),
        foto_absensi_path_out: row.foto_absensi_path_out.clone(),
        latitude_in: row.latitude_in,
        longitude_in: row.longitude_in,
        latitude_out: row.latitude_out,
        longitude_out: row.longitude_out,
    }
}

pub async fn laporan_absensi_harian_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<LaporanHarianFilter>,
) -> Result<Json<Vec<LaporanAbsensiResponse>>, AppError> {
    let rows = repo::get_laporan_harian_repo(&pool, filter.tanggal).await?;

    let responses: Vec<LaporanAbsensiResponse> = rows.iter().map(kalkulasi_keterangan).collect();
    Ok(Json(responses))
}

pub async fn laporan_absensi_bulanan_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<LaporanBulananFilter>,
) -> Result<Json<LaporanBulananResponse>, AppError> {
    let rows = repo::get_laporan_bulanan_repo(&pool, filter.pegawai_id, filter.bulan, filter.tahun)
        .await?;

    let mut total_terlambat = 0;
    let mut total_terlambat_toleransi = 0;
    let mut total_lembur = 0;
    let mut nama_pegawai = "Unknown".to_string();

    let rekap_harian: Vec<LaporanAbsensiResponse> = rows
        .iter()
        .map(|row| {
            if nama_pegawai == "Unknown" {
                nama_pegawai = row.nama_pegawai.clone();
            }

            let response = kalkulasi_keterangan(row);

            total_terlambat += response.terlambat_menit;
            total_terlambat_toleransi += response.terlambat_toleransi_menit;
            total_lembur += response.lembur_menit;

            response
        })
        .collect();

    let result = LaporanBulananResponse {
        pegawai_id: filter.pegawai_id,
        nama_pegawai,
        bulan: filter.bulan,
        tahun: filter.tahun,
        total_terlambat_menit: total_terlambat,
        total_terlambat_toleransi_menit: total_terlambat_toleransi,
        total_lembur_menit: total_lembur,
        rekap_harian,
    };

    Ok(Json(result))
}

pub async fn get_all_biometrik_status_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<BiometrikStatusDetail>>, AppError> {
    let list = repo::get_all_biometrik_status_repo(&pool).await?;
    Ok(Json(list))
}
