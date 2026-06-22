use super::{access, model::*, nilai_akhir_repo, repo};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use axum::{
    body::Body,
    extract::{Json, Multipart, Path, Query, State},
    http::{header, Response, StatusCode},
    Extension,
};
use std::{ffi::OsStr, path::Path as StdPath};
use uuid::Uuid;

fn validate_payload(payload: &UpsertAsesmenPayload) -> Result<(), AppError> {
    if !["Kuis", "Tugas", "UTS", "UAS", "Praktik"].contains(&payload.jenis.as_str()) {
        return Err(AppError::BadRequest(
            "Jenis asesmen tidak valid.".to_string(),
        ));
    }
    if !["Manual", "Online"].contains(&payload.mode.as_str()) {
        return Err(AppError::BadRequest(
            "Mode asesmen tidak valid.".to_string(),
        ));
    }
    if payload.judul.trim().is_empty()
        || payload.durasi_menit <= 0
        || payload.selesai_terjadwal <= payload.mulai_terjadwal
    {
        return Err(AppError::BadRequest(
            "Judul, durasi, dan rentang waktu asesmen tidak valid.".to_string(),
        ));
    }
    if payload.mode == "Online"
        && !payload
            .online_url
            .as_deref()
            .is_some_and(|url| url.starts_with("https://") || url.starts_with("http://"))
    {
        return Err(AppError::BadRequest(
            "Asesmen online memerlukan URL HTTP/HTTPS yang valid.".to_string(),
        ));
    }
    Ok(())
}

pub async fn schedule_options_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(query): Query<AsesmenQuery>,
) -> Result<Json<Vec<JadwalAsesmenOption>>, AppError> {
    Ok(Json(
        repo::schedule_options(&pool, &claims, query.tahun_akademik_id).await?,
    ))
}

pub async fn list_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(query): Query<AsesmenQuery>,
) -> Result<Json<Vec<AsesmenListRow>>, AppError> {
    Ok(Json(
        repo::list(&pool, &claims, query.tahun_akademik_id).await?,
    ))
}

pub async fn detail_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<AsesmenDetailResponse>, AppError> {
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_view(&permission)?;
    let mut response = repo::detail(&pool, id).await?;
    let production_only = permission.production
        && !permission.assigned
        && !permission.kaprodi
        && !permission.academic;
    if production_only {
        if response.asesmen.mode != "Manual"
            || !matches!(
                response.asesmen.status.as_str(),
                "Disetujui"
                    | "SiapDilaksanakan"
                    | "Berlangsung"
                    | "Selesai"
                    | "Dinilai"
                    | "Dikunci"
            )
        {
            return Err(AppError::Forbidden(
                "Asesmen belum tersedia untuk bagian penggandaan.".to_string(),
            ));
        }
        response.asesmen.online_url = None;
        response
            .dokumen
            .retain(|document| document.jenis != "KunciJawaban");
        response.review.clear();
        response.pelaksanaan = None;
        response.roster.clear();
        response.sesi_presensi = None;
    }
    Ok(Json(response))
}

pub async fn create_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<UpsertAsesmenPayload>,
) -> Result<(StatusCode, Json<AsesmenDetailResponse>), AppError> {
    validate_payload(&payload)?;
    let permission = access::for_jadwal(&pool, &claims, payload.jadwal_kuliah_id).await?;
    access::require_edit(&permission)?;
    let id = repo::create(&pool, claims.sub, payload).await?;
    Ok((StatusCode::CREATED, Json(repo::detail(&pool, id).await?)))
}

pub async fn update_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpsertAsesmenPayload>,
) -> Result<Json<AsesmenDetailResponse>, AppError> {
    validate_payload(&payload)?;
    let current_jadwal = access::jadwal_for_asesmen(&pool, id).await?;
    if current_jadwal != payload.jadwal_kuliah_id {
        return Err(AppError::BadRequest(
            "Kelas asesmen tidak dapat dipindahkan.".to_string(),
        ));
    }
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_edit(&permission)?;
    repo::update(&pool, id, payload).await?;
    Ok(Json(repo::detail(&pool, id).await?))
}

pub async fn submit_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_content(&permission)?;
    repo::submit(&pool, id).await?;
    Ok(Json(MessageResponse {
        message: "Asesmen diajukan untuk review Prodi.".to_string(),
    }))
}

pub async fn review_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReviewPayload>,
) -> Result<Json<MessageResponse>, AppError> {
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_review(&permission)?;
    repo::review(&pool, id, claims.sub, payload).await?;
    Ok(Json(MessageResponse {
        message: "Review asesmen tersimpan.".to_string(),
    }))
}

pub async fn upload_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path((id, jenis)): Path<(Uuid, String)>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<DokumenAsesmen>), AppError> {
    if !["Soal", "Lampiran", "KunciJawaban"].contains(&jenis.as_str()) {
        return Err(AppError::BadRequest(
            "Jenis dokumen tidak valid.".to_string(),
        ));
    }
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_content(&permission)?;
    let asesmen = repo::get_record(&pool, id).await?;
    if !matches!(asesmen.status.as_str(), "Draft" | "PerluRevisi") {
        return Err(AppError::BadRequest(
            "Dokumen hanya dapat diunggah saat Draft atau Perlu Revisi.".to_string(),
        ));
    }
    let field = multipart
        .next_field()
        .await?
        .ok_or_else(|| AppError::BadRequest("File wajib diunggah.".to_string()))?;
    let original_name = field.file_name().unwrap_or("dokumen.pdf").to_string();
    let data = field.bytes().await?.to_vec();
    if data.len() > 10 * 1024 * 1024 {
        return Err(AppError::BadRequest(
            "Ukuran dokumen maksimal 10 MB.".to_string(),
        ));
    }
    let ext = StdPath::new(&original_name)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    if !["pdf", "doc", "docx", "zip"].contains(&ext.as_str()) {
        return Err(AppError::BadRequest(
            "Dokumen harus PDF, DOC, DOCX, atau ZIP.".to_string(),
        ));
    }
    let mime = infer::get(&data)
        .map(|kind| kind.mime_type().to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let folder = format!("uploads/akademik/asesmen/{id}");
    tokio::fs::create_dir_all(&folder).await?;
    let path = format!("{folder}/{}.{}", Uuid::new_v4(), ext);
    tokio::fs::write(&path, &data).await?;
    let document_id = match repo::add_document(
        &pool,
        id,
        jenis,
        original_name,
        path.clone(),
        mime,
        data.len() as i64,
        claims.sub,
    )
    .await
    {
        Ok(document_id) => document_id,
        Err(error) => {
            let _ = tokio::fs::remove_file(path).await;
            return Err(error);
        }
    };
    let document = repo::detail(&pool, id)
        .await?
        .dokumen
        .into_iter()
        .find(|item| item.id == document_id)
        .ok_or(sqlx::Error::RowNotFound)?;
    Ok((StatusCode::CREATED, Json(document)))
}

pub async fn download_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path((id, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Response<Body>, AppError> {
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_view(&permission)?;
    let asesmen = repo::get_record(&pool, id).await?;
    let (path, name, jenis) = repo::document_path(&pool, id, document_id).await?;
    if permission.production
        && !permission.assigned
        && !permission.kaprodi
        && !permission.academic
        && (jenis == "KunciJawaban"
            || asesmen.mode != "Manual"
            || !matches!(
                asesmen.status.as_str(),
                "Disetujui"
                    | "SiapDilaksanakan"
                    | "Berlangsung"
                    | "Selesai"
                    | "Dinilai"
                    | "Dikunci"
            ))
    {
        return Err(AppError::Forbidden(
            "Dokumen belum dapat diakses bagian penggandaan.".to_string(),
        ));
    }
    let root = tokio::fs::canonicalize("./uploads").await?;
    let file = tokio::fs::canonicalize(path).await?;
    if !file.starts_with(root) || !file.is_file() {
        return Err(AppError::Forbidden("Path dokumen tidak valid.".to_string()));
    }
    let content = tokio::fs::read(&file).await?;
    repo::audit_document_download(&pool, document_id, claims.sub).await?;
    let safe_name = name.replace(['"', '\r', '\n'], "_");
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            mime_guess::from_path(&file)
                .first_or_octet_stream()
                .to_string(),
        )
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{safe_name}\""),
        )
        .body(Body::from(content))
        .map_err(|error| AppError::InternalServerError(error.to_string()))?)
}

pub async fn production_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<PenggandaanPayload>,
) -> Result<Json<MessageResponse>, AppError> {
    if !["Diajukan", "Diproses", "Selesai", "Diserahkan"].contains(&payload.status.as_str()) {
        return Err(AppError::BadRequest(
            "Status penggandaan tidak valid.".to_string(),
        ));
    }
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_production(&permission)?;
    repo::production(&pool, id, claims.sub, payload).await?;
    Ok(Json(MessageResponse {
        message: "Status penggandaan diperbarui.".to_string(),
    }))
}

pub async fn start_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<SesiAsesmenResponse>, AppError> {
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_execute(&permission)?;
    let code = Uuid::new_v4().simple().to_string()[..8].to_uppercase();
    Ok(Json(repo::start(&pool, id, claims.sub, code).await?))
}

pub async fn finish_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<FinishAsesmenPayload>,
) -> Result<Json<MessageResponse>, AppError> {
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_execute(&permission)?;
    repo::finish(&pool, id, payload).await?;
    Ok(Json(MessageResponse {
        message: "Pelaksanaan asesmen ditutup.".to_string(),
    }))
}

pub async fn attendance_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path((id, enrollment_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<PresensiAsesmenPayload>,
) -> Result<Json<MessageResponse>, AppError> {
    if !["Hadir", "Terlambat", "Izin", "Sakit", "Alpa"].contains(&payload.status.as_str()) {
        return Err(AppError::BadRequest(
            "Status presensi tidak valid.".to_string(),
        ));
    }
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_execute(&permission)?;
    repo::manual_attendance(&pool, id, enrollment_id, claims.sub, payload).await?;
    Ok(Json(MessageResponse {
        message: "Presensi ujian diperbarui.".to_string(),
    }))
}

pub async fn grade_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path((id, enrollment_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<NilaiAsesmenPayload>,
) -> Result<Json<MessageResponse>, AppError> {
    if payload.nilai < rust_decimal::Decimal::ZERO
        || payload.nilai > rust_decimal::Decimal::from(100)
    {
        return Err(AppError::BadRequest(
            "Nilai harus berada pada rentang 0–100.".to_string(),
        ));
    }
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_grade(&permission)?;
    repo::grade(&pool, id, enrollment_id, claims.sub, payload).await?;
    Ok(Json(MessageResponse {
        message: "Nilai asesmen tersimpan.".to_string(),
    }))
}

pub async fn lock_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_grade(&permission)?;
    repo::lock(&pool, id).await?;
    Ok(Json(MessageResponse {
        message: "Nilai asesmen dikunci.".to_string(),
    }))
}

pub async fn reopen_grade_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    let permission = access::for_asesmen(&pool, &claims, id).await?;
    access::require_grade(&permission)?;
    repo::reopen_grade(&pool, id).await?;
    Ok(Json(MessageResponse {
        message: "Nilai asesmen dibuka kembali untuk revisi.".to_string(),
    }))
}

pub async fn student_list_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(query): Query<AsesmenQuery>,
) -> Result<Json<Vec<AsesmenMahasiswaRow>>, AppError> {
    Ok(Json(
        repo::student_list(&pool, claims.sub, query.tahun_akademik_id).await?,
    ))
}

pub async fn student_check_in_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<CheckInAsesmenPayload>,
) -> Result<Json<MessageResponse>, AppError> {
    repo::student_check_in(&pool, claims.sub, payload.kode).await?;
    Ok(Json(MessageResponse {
        message: "Presensi ujian berhasil direkam.".to_string(),
    }))
}

pub async fn final_grade_classes_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(query): Query<AsesmenQuery>,
) -> Result<Json<Vec<KelasNilaiAkhir>>, AppError> {
    Ok(Json(
        nilai_akhir_repo::list_classes(&pool, &claims, query).await?,
    ))
}

pub async fn final_grade_detail_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(jadwal_id): Path<Uuid>,
) -> Result<Json<NilaiAkhirDetail>, AppError> {
    Ok(Json(
        nilai_akhir_repo::detail(&pool, &claims, jadwal_id).await?,
    ))
}

pub async fn final_grade_submit_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(jadwal_id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    nilai_akhir_repo::submit(&pool, &claims, jadwal_id).await?;
    Ok(Json(MessageResponse {
        message: "Nilai akhir diajukan kepada Kaprodi.".to_string(),
    }))
}

pub async fn final_grade_review_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(jadwal_id): Path<Uuid>,
    Json(payload): Json<ReviewNilaiAkhirPayload>,
) -> Result<Json<MessageResponse>, AppError> {
    nilai_akhir_repo::review(
        &pool,
        &claims,
        jadwal_id,
        &payload.aksi,
        payload.catatan.as_deref(),
    )
    .await?;
    Ok(Json(MessageResponse {
        message: "Review nilai akhir tersimpan.".to_string(),
    }))
}

pub async fn final_grade_publish_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(jadwal_id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    nilai_akhir_repo::publish(&pool, &claims, jadwal_id).await?;
    Ok(Json(MessageResponse {
        message: "Nilai akhir dipublikasikan ke KHS mahasiswa.".to_string(),
    }))
}

pub async fn scale_list_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(prodi_id): Path<Uuid>,
) -> Result<Json<Vec<SkalaNilaiRow>>, AppError> {
    Ok(Json(
        nilai_akhir_repo::list_scales(&pool, &claims, prodi_id).await?,
    ))
}

pub async fn scale_save_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(prodi_id): Path<Uuid>,
    Json(payload): Json<UpsertSkalaNilaiPayload>,
) -> Result<Json<Vec<SkalaNilaiRow>>, AppError> {
    Ok(Json(
        nilai_akhir_repo::save_scales(&pool, &claims, prodi_id, payload).await?,
    ))
}

pub async fn student_grades_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(query): Query<AsesmenQuery>,
) -> Result<Json<Vec<NilaiMataKuliahMahasiswa>>, AppError> {
    Ok(Json(
        nilai_akhir_repo::student_grades(&pool, claims.sub, query.tahun_akademik_id).await?,
    ))
}
