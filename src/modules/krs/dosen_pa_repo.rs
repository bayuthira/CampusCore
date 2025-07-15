// src/repositories/dosen_pa_repo.rs
use crate::{
    db::DbPool,
    errors::AppError,
    models::mahasiswa_model::MahasiswaBimbingan,
};
use uuid::Uuid;

pub async fn get_my_advisees_repo(
    pool: &DbPool,
    dosen_pa_id: Uuid,
) -> Result<Vec<MahasiswaBimbingan>, AppError> {
    let advisees = sqlx::query_as!(
        MahasiswaBimbingan,
        r#"
        SELECT
            m.id,
            m.nim,
            m.nama_mahasiswa,
            m.angkatan,
            m.email,
            p.nama_prodi
        FROM mahasiswa m
        JOIN prodi p ON m.prodi_id = p.id
        WHERE m.dosen_pa_id = $1
        ORDER BY m.nim ASC
        "#,
        dosen_pa_id
    )
    .fetch_all(pool)
    .await?;
    Ok(advisees)
}