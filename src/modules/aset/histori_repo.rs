// src/repositories/histori_aset_repo.rs
use super::{
    model::{HistoriAsetDetail,PindahkanAsetPayload,AsetDetail,AsetHistoriStatus,UpdateKondisiPayload,KondisiAset},
};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

pub async fn pindahkan_aset_repo(
    pool: &DbPool,
    aset_id: Uuid,
    user_aksi_id: Uuid,
    payload: PindahkanAsetPayload,
) -> Result<AsetDetail, AppError> {
    let mut tx = pool.begin().await?;

    // 1. Ambil lokasi ruangan saat ini sebelum dipindahkan
    let aset_sebelumnya = sqlx::query!("SELECT ruangan_id FROM aset WHERE id = $1", aset_id)
        .fetch_one(&mut *tx)
        .await?;

    // 2. Buat catatan histori baru
    sqlx::query!(
        r#"
        INSERT INTO histori_aset (aset_id, dari_ruangan_id, ke_ruangan_id, user_aksi_id, status, catatan)
        VALUES ($1, $2, $3, $4, 'Dipindahkan', $5)
        "#,
        aset_id,
        aset_sebelumnya.ruangan_id,
        payload.ke_ruangan_id,
        user_aksi_id,
        payload.catatan
    )
    .execute(&mut *tx)
    .await?;

    // 3. Update lokasi baru di tabel aset
    sqlx::query!(
        "UPDATE aset SET ruangan_id = $1, updated_at = now() WHERE id = $2",
        payload.ke_ruangan_id,
        aset_id
    )
    .execute(&mut *tx)
    .await?;

    // 4. Commit transaksi
    tx.commit().await?;

    // Ambil dan kembalikan detail aset terbaru
    let aset_terbaru = crate::modules::aset::repo::get_aset_by_id_repo(pool, aset_id).await?;
    Ok(aset_terbaru)
}


pub async fn get_histori_by_aset_id_repo(
    pool: &DbPool,
    aset_id: Uuid,
) -> Result<Vec<HistoriAsetDetail>, AppError> {
    let records = sqlx::query!(
        r#"
        SELECT
            h.id,
            h.status::TEXT as status,
            COALESCE(h.catatan, '') as "catatan!", -- Jamin tidak NULL
            h.tanggal_kejadian,
            h.user_aksi_id,
            COALESCE(u.full_name, 'User Dihapus') as "nama_user_aksi!", -- Jamin tidak NULL
            COALESCE(dari.nama_ruangan, '-') as "dari_ruangan!", -- Jamin tidak NULL
            COALESCE(ke.nama_ruangan, '-') as "ke_ruangan!"   -- Jamin tidak NULL
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

    let histori_list = records
        .into_iter()
        .map(|rec| {
            let status = match rec.status.as_deref() {
                Some("Ditempatkan") => AsetHistoriStatus::Ditempatkan,
                Some("Dipindahkan") => AsetHistoriStatus::Dipindahkan,
                Some("Dipinjam") => AsetHistoriStatus::Dipinjam,
                Some("Dikembalikan") => AsetHistoriStatus::Dikembalikan,
                Some("Dalam Perbaikan") => AsetHistoriStatus::DalamPerbaikan,
                Some("Perbaikan Selesai") => AsetHistoriStatus::PerbaikanSelesai,
                _ => AsetHistoriStatus::Dihapuskan,
            };

            // Tidak ada lagi .unwrap() atau Some()
            // Semua field sudah dijamin ada oleh query
            HistoriAsetDetail {
                id: rec.id,
                status,
                catatan: rec.catatan,
                tanggal_kejadian: rec.tanggal_kejadian,
                user_aksi_id: rec.user_aksi_id,
                nama_user_aksi: rec.nama_user_aksi,
                dari_ruangan: rec.dari_ruangan,
                ke_ruangan: rec.ke_ruangan,
            }
        })
        .collect();

    Ok(histori_list)
}

// src/modules/aset/histori_repo.rs

pub async fn update_kondisi_aset_repo(
    pool: &DbPool,
    aset_id: Uuid,
    user_aksi_id: Uuid,
    payload: UpdateKondisiPayload,
) -> Result<AsetDetail, AppError> {
    let mut tx = pool.begin().await?;

    // Tentukan string status histori secara manual
    let histori_status_str = match payload.kondisi {
        KondisiAset::DalamPerbaikan => "Dalam Perbaikan",
        KondisiAset::Baik => "Perbaikan Selesai",
        KondisiAset::Dihapuskan => "Dihapuskan",
        // Fallback untuk kondisi lain jika diperlukan
        _ => "Ditempatkan",
    };

    // Tentukan string kondisi aset secara manual
    let kondisi_str = match payload.kondisi {
        KondisiAset::Baik => "Baik",
        KondisiAset::RusakRingan => "Rusak Ringan",
        KondisiAset::RusakBerat => "Rusak Berat",
        KondisiAset::DalamPerbaikan => "Dalam Perbaikan",
        KondisiAset::Dihapuskan => "Dihapuskan",
    };

    // 1. Buat catatan histori baru menggunakan string
    sqlx::query(
        r#"
        INSERT INTO histori_aset (aset_id, user_aksi_id, status, catatan)
        VALUES ($1, $2, $3::"AsetHistoriStatus", $4)
        "#,
    )
    .bind(aset_id)
    .bind(user_aksi_id)
    .bind(histori_status_str) // <-- Gunakan string
    .bind(payload.catatan)
    .execute(&mut *tx)
    .await?;

    // 2. Update kondisi di tabel aset menggunakan string
    sqlx::query(
        "UPDATE aset SET kondisi = $1::\"KondisiAset\", updated_at = now() WHERE id = $2",
    )
    .bind(kondisi_str) // <-- Gunakan string
    .bind(aset_id)
    .execute(&mut *tx)
    .await?;

    // 3. Commit transaksi
    tx.commit().await?;

    // Ambil dan kembalikan detail aset terbaru
    let aset_terbaru = crate::modules::aset::repo::get_aset_by_id_repo(pool, aset_id).await?;
    Ok(aset_terbaru)
}