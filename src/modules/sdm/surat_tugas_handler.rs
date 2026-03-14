// src/modules/sdm/surat_tugas_handler.rs
use super::{
    surat_tugas_model::{
        CreateSuratTugasPayload, SuratTugas, SuratTugasDetail, UpdateSuratTugasPayload,
    },
    surat_tugas_repo as repo,
};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use axum::{
    Extension,
    extract::{Json, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use lazy_static::lazy_static;
use tera::{Context, Tera};
use time::{Month, Weekday};
use uuid::Uuid;

// Inisialisasi Tera secara global (Hanya dicompile sekali saat server menyala)
lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html"]);
        tera
    };
}

// Helper untuk format tanggal ke dalam Bahasa Indonesia (Hari, DD Bulan YYYY)
fn format_tanggal_indo(tanggal: time::Date) -> String {
    let hari = match tanggal.weekday() {
        Weekday::Monday => "Senin",
        Weekday::Tuesday => "Selasa",
        Weekday::Wednesday => "Rabu",
        Weekday::Thursday => "Kamis",
        Weekday::Friday => "Jumat",
        Weekday::Saturday => "Sabtu",
        Weekday::Sunday => "Minggu",
    };
    let bulan = match tanggal.month() {
        Month::January => "Januari",
        Month::February => "Februari",
        Month::March => "Maret",
        Month::April => "April",
        Month::May => "Mei",
        Month::June => "Juni",
        Month::July => "Juli",
        Month::August => "Agustus",
        Month::September => "September",
        Month::October => "Oktober",
        Month::November => "November",
        Month::December => "Desember",
    };
    format!(
        "{}, {:02} {} {}",
        hari,
        tanggal.day(),
        bulan,
        tanggal.year()
    )
}

/// Handler untuk Preview SPPD dalam format HTML
pub async fn preview_sppd_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Ambil data
    let detail = repo::get_surat_tugas_detail_repo(&pool, id).await?;

    // 2. Format Data Personel (Jauh lebih bersih tanpa query di dalam loop)
    let mut list_personel = Vec::new();
    for penerima in &detail.daftar_penerima {
        list_personel.push(super::print_model::PersonelPrint {
            nama: penerima.nama_lengkap.clone(),
            jabatan: penerima.jabatan.clone().unwrap_or_else(|| "-".to_string()),
            unit: penerima
                .unit_kerja
                .clone()
                .unwrap_or_else(|| "-".to_string()), // Ambil langsung dari data repo
        });
    }

    let mut empty_rows = 6;
    if list_personel.len() > 2 {
        empty_rows = 4;
    }
    if list_personel.len() > 4 {
        empty_rows = 3;
    }

    // 3. Masukkan ke Struct Serde
    let template_data = super::print_model::SppdTemplateContext {
        nomor_sppd: detail.nomor_sppd.unwrap_or_else(|| "-".to_string()),
        personel: list_personel,
        alasan: "workshop".to_string(), // TBD logika dari DB
        tujuan_kota: detail.tempat_tugas.clone(),
        alamat: detail.tempat_tugas,
        nama_kegiatan: detail.tugas,

        // Format Tanggal Indo digunakan di sini
        tgl_berangkat: format_tanggal_indo(detail.tanggal_mulai),
        tgl_kembali: format_tanggal_indo(detail.tanggal_selesai),

        penandatangan_tempat: "Tasikmalaya".to_string(),
        penandatangan_tanggal: format_tanggal_indo(detail.created_at.date()),

        // Data Penandatanganan
        penandatangan_jabatan: detail
            .jabatan_penandatangan
            .unwrap_or_else(|| "Ketua".to_string()),
        penandatangan_nama: detail.nama_penandatangan,
        penandatangan_nik: detail.nip_penandatangan,

        empty_row_count: empty_rows,
        tembusan: detail.tembusan,
    };

    // 4. Ubah Struct menjadi Tera Context
    let context = Context::from_serialize(&template_data)
        .map_err(|_| AppError::InternalServerError("Gagal membuat konteks template".to_string()))?;

    // 5. Render HTML dengan Engine Tera
    let rendered_html = TERA
        .render("sppd.html", &context)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(rendered_html))
}

pub async fn create_surat_tugas_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<CreateSuratTugasPayload>,
) -> Result<(StatusCode, Json<SuratTugasDetail>), AppError> {
    let user_pembuat_id = claims.sub;
    let surat_tugas = repo::create_surat_tugas_repo(&pool, user_pembuat_id, payload).await?;
    Ok((StatusCode::CREATED, Json(surat_tugas)))
}

pub async fn get_all_surat_tugas_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<SuratTugas>>, AppError> {
    let list = repo::get_all_surat_tugas_repo(&pool).await?;
    Ok(Json(list))
}

pub async fn get_surat_tugas_detail_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<SuratTugasDetail>, AppError> {
    let detail = repo::get_surat_tugas_detail_repo(&pool, id).await?;
    Ok(Json(detail))
}

pub async fn update_surat_tugas_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSuratTugasPayload>,
) -> Result<Json<SuratTugasDetail>, AppError> {
    let updated_surat_tugas = repo::update_surat_tugas_repo(&pool, id, payload).await?;
    Ok(Json(updated_surat_tugas))
}

pub async fn delete_surat_tugas_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    repo::delete_surat_tugas_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
