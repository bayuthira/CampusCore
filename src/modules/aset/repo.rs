use crate::{
    db::DbPool,
    errors::AppError,
    models::aset_model::{AsetDetail, AsetPayload, KondisiAset}, // Impor KondisiAset
};
use uuid::Uuid;

// Helper untuk memetakan dari hasil query DB ke struct AsetDetail
fn map_rec_to_aset_detail(rec: sqlx::postgres::PgRow) -> AsetDetail {
    use sqlx::Row;
    let kondisi_str: Option<String> = rec.try_get("kondisi").unwrap_or_default();
    let kondisi = match kondisi_str.as_deref() {
        Some("Rusak Ringan") => KondisiAset::RusakRingan,
        Some("Rusak Berat") => KondisiAset::RusakBerat,
        Some("Dalam Perbaikan") => KondisiAset::DalamPerbaikan,
        Some("Dihapuskan") => KondisiAset::Dihapuskan,
        _ => KondisiAset::Baik,
    };

    AsetDetail {
        id: rec.get("id"),
        nama_aset: rec.get("nama_aset"),
        kode_aset: rec.get("kode_aset"),
        deskripsi: rec.get("deskripsi"),
        tanggal_pembelian: rec.get("tanggal_pembelian"),
        kondisi,
        jenis_aset_id: rec.get("jenis_aset_id"),
        nama_jenis: rec.get("nama_jenis"),
        ruangan_id: rec.get("ruangan_id"),
        nama_ruangan: rec.get("nama_ruangan"),
        kode_ruangan: rec.get("kode_ruangan"),
        created_at: rec.get("created_at"),
        updated_at: rec.get("updated_at"),
    }
}

pub async fn create_aset_repo(pool: &DbPool, payload: AsetPayload) -> Result<AsetDetail, AppError> {
    let kondisi_str = payload.kondisi.as_str();
    
    // Tambahkan ::"KondisiAset" pada placeholder $5
    let id = sqlx::query_scalar(
        "INSERT INTO aset (nama_aset, kode_aset, deskripsi, tanggal_pembelian, kondisi, jenis_aset_id, ruangan_id) VALUES ($1, $2, $3, $4, $5::\"KondisiAset\", $6, $7) RETURNING id"
    )
    .bind(payload.nama_aset)
    .bind(payload.kode_aset)
    .bind(payload.deskripsi)
    .bind(payload.tanggal_pembelian)
    .bind(kondisi_str) // $5
    .bind(payload.jenis_aset_id)
    .bind(payload.ruangan_id)
    .fetch_one(pool).await?;

    let new_aset = get_aset_by_id_repo(pool, id).await?;
    Ok(new_aset)
}

pub async fn get_all_aset_repo(pool: &DbPool) -> Result<Vec<AsetDetail>, AppError> {
    let list = sqlx::query(
        r#"
        SELECT
            a.id, a.nama_aset, a.kode_aset, a.deskripsi, a.tanggal_pembelian,
            a.kondisi::TEXT, a.jenis_aset_id, ja.nama_jenis,
            a.ruangan_id, r.nama_ruangan, r.kode_ruangan,
            a.created_at, a.updated_at
        FROM aset a
        JOIN jenis_aset ja ON a.jenis_aset_id = ja.id
        LEFT JOIN ruangan r ON a.ruangan_id = r.id
        ORDER BY a.nama_aset ASC
        "#
    )
    .map(map_rec_to_aset_detail)
    .fetch_all(pool).await?;
    Ok(list)
}

pub async fn get_aset_by_id_repo(pool: &DbPool, id: Uuid) -> Result<AsetDetail, AppError> {
    let aset = sqlx::query(
         r#"
        SELECT
            a.id, a.nama_aset, a.kode_aset, a.deskripsi, a.tanggal_pembelian,
            a.kondisi::TEXT, a.jenis_aset_id, ja.nama_jenis,
            a.ruangan_id, r.nama_ruangan, r.kode_ruangan,
            a.created_at, a.updated_at
        FROM aset a
        JOIN jenis_aset ja ON a.jenis_aset_id = ja.id
        LEFT JOIN ruangan r ON a.ruangan_id = r.id
        WHERE a.id = $1
        "#
    )
    .bind(id)
    .map(map_rec_to_aset_detail)
    .fetch_one(pool).await?;
    Ok(aset)
}

pub async fn update_aset_repo(pool: &DbPool, id: Uuid, payload: AsetPayload) -> Result<AsetDetail, AppError> {
    let kondisi_str = payload.kondisi.as_str();

    // Tambahkan ::"KondisiAset" pada placeholder $5
    let rows_affected = sqlx::query(
        "UPDATE aset SET nama_aset = $1, kode_aset = $2, deskripsi = $3, tanggal_pembelian = $4, kondisi = $5::\"KondisiAset\", jenis_aset_id = $6, ruangan_id = $7, updated_at = now() WHERE id = $8"
    )
    .bind(payload.nama_aset).bind(payload.kode_aset).bind(payload.deskripsi)
    .bind(payload.tanggal_pembelian).bind(kondisi_str).bind(payload.jenis_aset_id) // $5
    .bind(payload.ruangan_id).bind(id)
    .execute(pool).await?.rows_affected();
    
    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }

    let updated_aset = get_aset_by_id_repo(pool, id).await?;
    Ok(updated_aset)
}

pub async fn delete_aset_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM aset WHERE id = $1", id).execute(pool).await?.rows_affected();
    if rows_affected == 0 { return Err(sqlx::Error::RowNotFound.into()); }
    Ok(())
}

