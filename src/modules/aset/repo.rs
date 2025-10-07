use super::model::{AsetDetail, AsetPayload, KondisiAset,AsetFilter,KondisiAsetSummary};

use crate::{db::DbPool, errors::AppError};
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
        peminjaman_id: rec.get("peminjaman_id"),
        nama_peminjam: rec.get("nama_peminjam"),
        estimasi_tanggal_kembali: rec.get("estimasi_tanggal_kembali"),
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


pub async fn get_all_aset_repo(
    pool: &DbPool,
    filter: AsetFilter, // <-- Terima struct filter
) -> Result<Vec<AsetDetail>, AppError> {
    
    // Gunakan if/else untuk memilih query yang tepat
    let list = if let Some(ruangan_id) = filter.ruangan_id {
        // --- Query JIKA ADA filter ruangan_id ---
        sqlx::query(
            r#"
            SELECT
                a.id, a.nama_aset, a.kode_aset, a.deskripsi, a.tanggal_pembelian,
                a.kondisi::TEXT, a.jenis_aset_id, ja.nama_jenis,
                a.ruangan_id, r.nama_ruangan, r.kode_ruangan,
                p.id as peminjaman_id,
                peminjam.full_name as nama_peminjam,
                p.estimasi_tanggal_kembali,
                a.created_at, a.updated_at
            FROM aset a
            JOIN jenis_aset ja ON a.jenis_aset_id = ja.id
            LEFT JOIN ruangan r ON a.ruangan_id = r.id
            LEFT JOIN peminjaman_aset p ON a.id = p.aset_id AND p.status = 'Dipinjam'
            LEFT JOIN users peminjam ON p.user_peminjam_id = peminjam.id
            WHERE a.ruangan_id = $1
            ORDER BY a.nama_aset ASC
            "#,
        )
        .bind(ruangan_id)
        .map(map_rec_to_aset_detail)
        .fetch_all(pool)
        .await?
    } else {
        // --- Query ASLI ANDA jika tidak ada filter ---
        sqlx::query(
            r#"
            SELECT
                a.id, a.nama_aset, a.kode_aset, a.deskripsi, a.tanggal_pembelian,
                a.kondisi::TEXT, a.jenis_aset_id, ja.nama_jenis,
                a.ruangan_id, r.nama_ruangan, r.kode_ruangan,
                p.id as peminjaman_id,
                peminjam.full_name as nama_peminjam,
                p.estimasi_tanggal_kembali,
                a.created_at, a.updated_at
            FROM aset a
            JOIN jenis_aset ja ON a.jenis_aset_id = ja.id
            LEFT JOIN ruangan r ON a.ruangan_id = r.id
            LEFT JOIN peminjaman_aset p ON a.id = p.aset_id AND p.status = 'Dipinjam'
            LEFT JOIN users peminjam ON p.user_peminjam_id = peminjam.id
            ORDER BY a.nama_aset ASC
            "#,
        )
        .map(map_rec_to_aset_detail)
        .fetch_all(pool)
        .await?
    };
    
    Ok(list)
}


pub async fn get_aset_by_id_repo(pool: &DbPool, id: Uuid) -> Result<AsetDetail, AppError> {
    // Query ini sekarang identik dengan `get_all_aset_repo` tapi dengan tambahan WHERE
    let query_str = r#"
        SELECT
            a.id, a.nama_aset, a.kode_aset, a.deskripsi, a.tanggal_pembelian,
            a.kondisi::TEXT, a.jenis_aset_id, ja.nama_jenis,
            a.ruangan_id, r.nama_ruangan, r.kode_ruangan,
            
            -- Kolom baru dari join ke peminjaman_aset dan users
            p.id as peminjaman_id,
            peminjam.full_name as nama_peminjam,
            p.estimasi_tanggal_kembali,
            
            a.created_at, a.updated_at
        FROM aset a
        JOIN jenis_aset ja ON a.jenis_aset_id = ja.id
        LEFT JOIN ruangan r ON a.ruangan_id = r.id
        LEFT JOIN peminjaman_aset p ON a.id = p.aset_id AND p.status = 'Dipinjam'
        LEFT JOIN users peminjam ON p.user_peminjam_id = peminjam.id
        WHERE a.id = $1
    "#;

    let aset = sqlx::query(query_str)
        .bind(id)
        .map(map_rec_to_aset_detail)
        .fetch_one(pool)
        .await?;
        
    Ok(aset)
}

pub async fn update_aset_repo(
    pool: &DbPool,
    id: Uuid,
    payload: AsetPayload,
) -> Result<AsetDetail, AppError> {
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
    let rows_affected = sqlx::query!("DELETE FROM aset WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}

pub async fn get_kondisi_summary_repo(pool: &DbPool) -> Result<KondisiAsetSummary, AppError> {
    let summary = sqlx::query_as!(
        KondisiAsetSummary,
        r#"
        SELECT
            COUNT(*) FILTER (WHERE kondisi = 'Baik') as "baik!",
            COUNT(*) FILTER (WHERE kondisi = 'Rusak Ringan') as "rusak_ringan!",
            COUNT(*) FILTER (WHERE kondisi = 'Rusak Berat') as "rusak_berat!",
            COUNT(*) FILTER (WHERE kondisi = 'Dalam Perbaikan') as "dalam_perbaikan!",
            COUNT(*) FILTER (WHERE kondisi = 'Dihapuskan') as "dihapuskan!"
        FROM aset
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(summary)
}