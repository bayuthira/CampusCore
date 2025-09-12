// src/modules/aset/histori_repo.rs
use super::model::{
    AsetDetail, AsetHistoriStatus, CreateHistoriPayload, HistoriAsetDetail, KembalikanAsetPayload,
    KondisiAset, PindahkanAsetPayload, PinjamAsetPayload, UpdateKondisiPayload,
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
            h.catatan,
            h.tanggal_kejadian,
            h.user_aksi_id,
            COALESCE(u_aksi.full_name, 'User Dihapus') as "nama_user_aksi!",
            COALESCE(dari.nama_ruangan, '-') as "dari_ruangan!",
            COALESCE(ke.nama_ruangan, '-') as "ke_ruangan!",
            -- Subquery untuk mengambil nama peminjam HANYA jika statusnya relevan
            (
                SELECT peminjam.full_name
                FROM peminjaman_aset pa
                JOIN users peminjam ON pa.user_peminjam_id = peminjam.id
                WHERE pa.aset_id = h.aset_id 
                  AND pa.tanggal_pinjam = h.tanggal_kejadian
                  AND (h.status = 'Dipinjam' OR h.status = 'Dikembalikan')
                LIMIT 1
            ) as nama_peminjam
        FROM histori_aset h
        JOIN users u_aksi ON h.user_aksi_id = u_aksi.id
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

            HistoriAsetDetail {
                id: rec.id,
                status,
                catatan: rec.catatan,
                tanggal_kejadian: rec.tanggal_kejadian,
                user_aksi_id: rec.user_aksi_id,
                nama_user_aksi: rec.nama_user_aksi,
                dari_ruangan: rec.dari_ruangan,
                ke_ruangan: rec.ke_ruangan,
                nama_peminjam: rec.nama_peminjam, // <-- Ambil field baru dari hasil query
            }
        })
        .collect();

    Ok(histori_list)
}

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
    sqlx::query("UPDATE aset SET kondisi = $1::\"KondisiAset\", updated_at = now() WHERE id = $2")
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

pub async fn create_histori_repo(
    pool: &DbPool,
    aset_id: Uuid,
    user_aksi_id: Uuid,
    payload: CreateHistoriPayload,
) -> Result<AsetDetail, AppError> {
    let mut tx = pool.begin().await?;

    let aset_saat_ini = sqlx::query!(
        "SELECT kondisi::TEXT as kondisi, ruangan_id FROM aset WHERE id = $1 FOR UPDATE",
        aset_id
    )
    .fetch_one(&mut *tx)
    .await?;
    let kondisi_saat_ini_str = aset_saat_ini.kondisi.unwrap_or_default();

    let mut dari_ruangan_id = aset_saat_ini.ruangan_id;
    let mut ke_ruangan_id = aset_saat_ini.ruangan_id;
    let mut kondisi_aset_baru: Option<KondisiAset> = None;

    // --- PERBAIKAN UTAMA DI SINI ---
    match payload.status {
        // Gabungkan logika untuk status yang sama-sama butuh `ke_ruangan_id`
        AsetHistoriStatus::Ditempatkan | AsetHistoriStatus::Dipindahkan => {
            ke_ruangan_id = payload.ke_ruangan_id;
            if ke_ruangan_id.is_none() {
                return Err(AppError::Forbidden(
                    "Ruangan tujuan harus diisi untuk status 'Ditempatkan' atau 'Dipindahkan'."
                        .to_string(),
                ));
            }
        }
        AsetHistoriStatus::DalamPerbaikan => {
            dari_ruangan_id = None;
            ke_ruangan_id = None;
            kondisi_aset_baru = Some(KondisiAset::DalamPerbaikan);
        }
        AsetHistoriStatus::PerbaikanSelesai => {
            if kondisi_saat_ini_str != "Dalam Perbaikan" {
                return Err(AppError::Forbidden(
                    "Aset tidak sedang dalam perbaikan.".to_string(),
                ));
            }
            dari_ruangan_id = None;
            ke_ruangan_id = None;
            kondisi_aset_baru = Some(KondisiAset::Baik);
        }
        AsetHistoriStatus::Dihapuskan => {
            dari_ruangan_id = None;
            ke_ruangan_id = None;
            kondisi_aset_baru = Some(KondisiAset::Dihapuskan);
        }
        _ => {
            // Untuk status lain seperti Dipinjam, Dikembalikan
            dari_ruangan_id = None;
            ke_ruangan_id = None;
        }
    }

    // ... (sisa fungsi untuk update dan insert histori tidak berubah) ...

    // Update tabel aset
    if let Some(kondisi) = kondisi_aset_baru {
        sqlx::query("UPDATE aset SET kondisi = $1::\"KondisiAset\", ruangan_id = $2, updated_at = now() WHERE id = $3")
            .bind(kondisi.as_str()).bind(ke_ruangan_id).bind(aset_id)
            .execute(&mut *tx).await?;
    } else {
        sqlx::query("UPDATE aset SET ruangan_id = $1, updated_at = now() WHERE id = $2")
            .bind(ke_ruangan_id)
            .bind(aset_id)
            .execute(&mut *tx)
            .await?;
    }

    // Selalu buat catatan histori
    sqlx::query(
        r#"
        INSERT INTO histori_aset (aset_id, dari_ruangan_id, ke_ruangan_id, user_aksi_id, status, catatan)
        VALUES ($1, $2, $3, $4, $5::"AsetHistoriStatus", $6)
        "#,
    )
    .bind(aset_id).bind(dari_ruangan_id).bind(ke_ruangan_id)
    .bind(user_aksi_id).bind(payload.status.as_str()).bind(payload.catatan)
    .execute(&mut *tx).await?;

    tx.commit().await?;

    let aset_terbaru = crate::modules::aset::repo::get_aset_by_id_repo(pool, aset_id).await?;
    Ok(aset_terbaru)
}

pub async fn pinjam_aset_repo(
    pool: &DbPool,
    aset_id: Uuid,
    user_approve_id: Uuid,
    payload: PinjamAsetPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // 1. Cek kondisi aset: Hanya bisa dipinjam jika 'Baik'
    let aset = sqlx::query!(
        "SELECT kondisi::TEXT as kondisi FROM aset WHERE id = $1 FOR UPDATE",
        aset_id
    )
    .fetch_one(&mut *tx)
    .await?;

    if aset.kondisi != Some("Baik".to_string()) {
        return Err(AppError::Forbidden(format!(
            "Aset tidak dalam kondisi 'Baik' dan tidak dapat dipinjam (Kondisi saat ini: {}).",
            aset.kondisi.unwrap_or_default()
        )));
    }

    // 2. Cek apakah aset sudah sedang dipinjam
    let peminjaman_aktif = sqlx::query!(
        "SELECT id FROM peminjaman_aset WHERE aset_id = $1 AND status = 'Dipinjam'",
        aset_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    if peminjaman_aktif.is_some() {
        return Err(AppError::Forbidden(
            "Aset ini sudah sedang dalam status dipinjam.".to_string(),
        ));
    }

    // 3. Buat catatan peminjaman baru (tidak berubah)
    sqlx::query!(
        "INSERT INTO peminjaman_aset (aset_id, user_peminjam_id, estimasi_tanggal_kembali, catatan_pinjam, user_approve_pinjam_id) VALUES ($1, $2, $3, $4, $5)",
        aset_id, payload.user_peminjam_id, payload.estimasi_tanggal_kembali, payload.catatan, user_approve_id
    ).execute(&mut *tx).await?;

    // 4. Buat catatan histori (tidak berubah)
    sqlx::query!(
        "INSERT INTO histori_aset (aset_id, user_aksi_id, status, catatan) VALUES ($1, $2, 'Dipinjam', $3)",
        aset_id, user_approve_id, payload.catatan
    ).execute(&mut *tx).await?;

    // 5. HAPUS logika update kondisi aset menjadi 'Dalam Perbaikan'
    // Logika ini sudah tidak diperlukan lagi.

    tx.commit().await?;
    Ok(())
}

pub async fn kembalikan_aset_repo(
    pool: &DbPool,
    peminjaman_id: Uuid,
    user_approve_id: Uuid,
    payload: KembalikanAsetPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // 1. Ambil aset_id dari data peminjaman yang aktif
    let peminjaman = sqlx::query!(
        "SELECT aset_id FROM peminjaman_aset WHERE id = $1 AND status = 'Dipinjam'",
        peminjaman_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::Forbidden(
            "Transaksi peminjaman tidak ditemukan atau sudah dikembalikan.".to_string(),
        )
    })?;

    // 2. Update tabel peminjaman berdasarkan ID peminjaman
    sqlx::query!(
        "UPDATE peminjaman_aset SET status = 'Dikembalikan', tanggal_kembali_aktual = now(), catatan_kembali = $1, user_approve_kembali_id = $2 WHERE id = $3",
        payload.catatan, user_approve_id, peminjaman_id
    )
    .execute(&mut *tx)
    .await?;

    // 3. Buat catatan histori
    sqlx::query!(
        "INSERT INTO histori_aset (aset_id, user_aksi_id, status, catatan) VALUES ($1, $2, 'Dikembalikan', $3)",
        peminjaman.aset_id, user_approve_id, payload.catatan
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
