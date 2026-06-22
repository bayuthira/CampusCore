use super::{model::*, repo};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use axum::{
    extract::{Json, Path, Query, State},
    Extension,
};
use uuid::Uuid;

pub async fn status_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<StatusAkhirSemester>, AppError> {
    Ok(Json(repo::status(&pool, id).await?))
}
pub async fn close_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    repo::close(&pool, id, claims.sub).await?;
    Ok(Json(MessageResponse {
        message: "Semester ditutup; AKM, IPS/IPK, dan antrean Feeder telah dibuat.".to_string(),
    }))
}
pub async fn khs_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(q): Query<TahunAkademikQuery>,
) -> Result<Json<KhsResponse>, AppError> {
    Ok(Json(
        repo::khs(&pool, claims.sub, q.tahun_akademik_id).await?,
    ))
}
pub async fn transcript_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
) -> Result<Json<TranskripResponse>, AppError> {
    Ok(Json(repo::transcript(&pool, claims.sub).await?))
}
pub async fn outbox_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<FeederOutboxRow>>, AppError> {
    Ok(Json(repo::outbox(&pool).await?))
}
pub async fn feeder_result_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<FeederResultPayload>,
) -> Result<Json<MessageResponse>, AppError> {
    repo::feeder_result(&pool, id, payload).await?;
    Ok(Json(MessageResponse {
        message: "Status sinkronisasi Feeder diperbarui.".to_string(),
    }))
}

pub async fn corrections_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
) -> Result<Json<Vec<KoreksiNilaiRow>>, AppError> {
    Ok(Json(repo::corrections(&pool, &claims).await?))
}
pub async fn submit_correction_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<AjukanKoreksiNilaiPayload>,
) -> Result<Json<MessageResponse>, AppError> {
    repo::submit_correction(&pool, &claims, payload).await?;
    Ok(Json(MessageResponse {
        message: "Koreksi nilai diajukan kepada Kaprodi.".to_string(),
    }))
}
pub async fn review_correction_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReviewKoreksiPayload>,
) -> Result<Json<MessageResponse>, AppError> {
    repo::review_correction(&pool, &claims, id, payload).await?;
    Ok(Json(MessageResponse {
        message: "Review koreksi nilai tersimpan.".to_string(),
    }))
}
pub async fn apply_correction_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    repo::apply_correction(&pool, &claims, id).await?;
    Ok(Json(MessageResponse {
        message: "Koreksi diterapkan dan antrean Feeder diperbarui.".to_string(),
    }))
}
