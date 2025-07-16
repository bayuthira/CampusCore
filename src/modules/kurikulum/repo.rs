use crate::{db::DbPool, errors::AppError, modules::matakuliah::model::MataKuliahDetail};
use uuid::Uuid;

use super::model::{
    AddMataKuliahToKurikulumPayload, CreateKurikulumPayload, KurikulumDetail,
    UpdateKurikulumPayload,
};

pub async fn get_kurikulum_by_id_repo_inner(
    pool: &DbPool,
    id: Uuid,
) -> Result<KurikulumDetail, AppError> {
    let kurikulum = sqlx::query_as!(
        KurikulumDetail,
        r#"
        SELECT k.*, p.nama_prodi FROM kurikulum k
        LEFT JOIN prodi p ON k.prodi_id = p.id
        WHERE k.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(kurikulum)
}

pub async fn create_kurikulum_repo(
    pool: &DbPool,
    payload: CreateKurikulumPayload,
) -> Result<KurikulumDetail, AppError> {
    let id = sqlx::query_scalar!(
        "INSERT INTO kurikulum (nama, tahun_mulai, is_active, prodi_id) VALUES ($1, $2, $3, $4) RETURNING id",
        payload.nama, payload.tahun_mulai, payload.is_active, payload.prodi_id
    ).fetch_one(pool).await?;
    let new_kurikulum = get_kurikulum_by_id_repo_inner(pool, id).await?;
    Ok(new_kurikulum)
}

pub async fn get_all_kurikulum_repo(pool: &DbPool) -> Result<Vec<KurikulumDetail>, AppError> {
    let kurikulum_list = sqlx::query_as!(
        KurikulumDetail,
        r#"
        SELECT k.*, COALESCE(p.nama_prodi, 'Prodi Tidak Ditemukan') as "nama_prodi!"
        FROM kurikulum k
        LEFT JOIN prodi p ON k.prodi_id = p.id
        ORDER BY k.tahun_mulai DESC, k.nama ASC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(kurikulum_list)
}

pub async fn update_kurikulum_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateKurikulumPayload,
) -> Result<KurikulumDetail, AppError> {
    sqlx::query!(
        "UPDATE kurikulum SET nama = $1, tahun_mulai = $2, is_active = $3, prodi_id = $4, updated_at = now() WHERE id = $5",
        payload.nama, payload.tahun_mulai, payload.is_active, payload.prodi_id, id
    ).execute(pool).await?;
    let updated_kurikulum = get_kurikulum_by_id_repo_inner(pool, id).await?;
    Ok(updated_kurikulum)
}

pub async fn delete_kurikulum_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM kurikulum WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}

pub async fn add_matakuliah_to_kurikulum_repo(
    pool: &DbPool,
    kurikulum_id: Uuid,
    payload: AddMataKuliahToKurikulumPayload,
) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO kurikulum_matakuliah (kurikulum_id, matakuliah_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        kurikulum_id, payload.matakuliah_id
    ).execute(pool).await?;
    Ok(())
}

pub async fn get_matakuliah_in_kurikulum_repo(
    pool: &DbPool,
    kurikulum_id: Uuid,
) -> Result<Vec<MataKuliahDetail>, AppError> {
    let mk_list = sqlx::query_as!(
        MataKuliahDetail,
        r#"
        SELECT 
            mk.id,
            mk.kode_mk,
            mk.nama_mk,
            mk.sks,
            mk.semester_target,
            mk.prodi_id,
            COALESCE(p.nama_prodi, 'Prodi Tidak Ditemukan') as "nama_prodi!"
        FROM mata_kuliah mk
        INNER JOIN kurikulum_matakuliah km ON mk.id = km.matakuliah_id
        LEFT JOIN prodi p ON mk.prodi_id = p.id
        WHERE km.kurikulum_id = $1
        ORDER BY mk.kode_mk
        "#,
        kurikulum_id
    )
    .fetch_all(pool)
    .await?;
    Ok(mk_list)
}

pub async fn remove_matakuliah_from_kurikulum_repo(
    pool: &DbPool,
    kurikulum_id: Uuid,
    matakuliah_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM kurikulum_matakuliah WHERE kurikulum_id = $1 AND matakuliah_id = $2",
        kurikulum_id,
        matakuliah_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
