// src/repositories/histori_aset_repo.rs
use crate::{db::DbPool, errors::AppError, models::aset_model::HistoriAsetDetail};
use uuid::Uuid;

pub async fn get_histori_by_aset_id_repo(pool: &DbPool, aset_id: Uuid) -> Result<Vec<HistoriAsetDetail>, AppError> {
    let histori_list = sqlx::query_as!(
        HistoriAsetDetail,
        r#"
        SELECT 
            h.id,
            h.status as "status: _",
            h.catatan,
            h.tanggal_kejadian,
            h.user_aksi_id,
            u.full_name as "nama_user_aksi!",
            dari.nama_ruangan as dari_ruangan,
            ke.nama_ruangan as ke_ruangan
        FROM histori_aset h
        JOIN users u ON h.user_aksi_id = u.id
        LEFT JOIN ruangan dari ON h.dari_ruangan_id = dari.id
        LEFT JOIN ruangan ke ON h.ke_ruangan_id = ke.id
        WHERE h.aset_id = $1
        ORDER BY h.tanggal_kejadian DESC
        "#,
        aset_id
    )
    .fetch_all(pool)
    .await?;
    Ok(histori_list)
}