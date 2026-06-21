use super::{
    report_model::{ReportKelasRow, ReportPembelajaranQuery, ReportPertemuanRow},
    report_repo,
};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;

pub async fn list_report_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(query): Query<ReportPembelajaranQuery>,
) -> Result<Json<Vec<ReportKelasRow>>, AppError> {
    Ok(Json(
        report_repo::list_report(&pool, &claims, query.tahun_akademik_id).await?,
    ))
}

pub async fn detail_report_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(jadwal_id): Path<Uuid>,
) -> Result<Json<Vec<ReportPertemuanRow>>, AppError> {
    Ok(Json(
        report_repo::detail_report(&pool, &claims, jadwal_id).await?,
    ))
}
