// src/modules/sdm/cuti_repo.rs
use super::cuti_model::{
    ApprovalCutiPayload, CreateJatahCutiPayload, CreatePengajuanCutiPayload, JatahCuti,KuotaCutiDetail,PengajuanCuti, StatusCuti, TipeCuti
};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

pub async fn get_pengajuan_cuti_by_id_repo<'a, E>(executor: E, id: Uuid) -> Result<PengajuanCuti, AppError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let pengajuan = sqlx::query_as!(
        PengajuanCuti,
        r#"
        SELECT id, pegawai_id, tanggal_mulai, tanggal_selesai, jumlah_hari, alasan,
               status as "status: _", tipe_cuti as "tipe_cuti: _", 
               user_approve_id, catatan_approval, created_at
        FROM pengajuan_cuti
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(executor)
    .await?;
    Ok(pengajuan)
}


/// Endpoint Khusus Admin: Membuat/mengatur jatah cuti tahunan untuk pegawai
pub async fn create_jatah_cuti_repo(
    pool: &DbPool,
    payload: CreateJatahCutiPayload,
) -> Result<JatahCuti, AppError> {
    // Gunakan ON CONFLICT untuk update jika jatah di tahun yg sama sudah ada
    let jatah = sqlx::query_as!(
        JatahCuti,
        r#"
        INSERT INTO jatah_cuti (pegawai_id, tahun, kuota_total)
        VALUES ($1, $2, $3)
        ON CONFLICT (pegawai_id, tahun) DO UPDATE SET
            kuota_total = EXCLUDED.kuota_total
        RETURNING *
        "#,
        payload.pegawai_id,
        payload.tahun,
        payload.kuota_total
    )
    .fetch_one(pool)
    .await?;
    Ok(jatah)
}

/// Endpoint Pegawai: Mengajukan cuti baru
pub async fn create_pengajuan_cuti_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    payload: CreatePengajuanCutiPayload,
) -> Result<PengajuanCuti, AppError> {
    let mut tx = pool.begin().await?;
    let tahun_cuti = payload.tanggal_mulai.year() as i16;

    let jatah = sqlx::query_as!(
        JatahCuti,
        "SELECT * FROM jatah_cuti WHERE pegawai_id = $1 AND tahun = $2",
        pegawai_id,
        tahun_cuti
    )
    .fetch_optional(&mut *tx)
    .await?;

    let tipe_cuti_baru;
    if let Some(j) = jatah {
        let sisa_cuti = j.kuota_total - j.kuota_terpakai;
        tipe_cuti_baru = if sisa_cuti >= payload.jumlah_hari { TipeCuti::Paid } else { TipeCuti::Unpaid };
    } else {
        tipe_cuti_baru = TipeCuti::Unpaid;
    }
    
    let tipe_cuti_str = tipe_cuti_baru.as_str();

    // --- PERBAIKAN DI SINI: Ganti query_as! dengan query_scalar() ---
    let new_id = sqlx::query_scalar(
        r#"
        INSERT INTO pengajuan_cuti (pegawai_id, tanggal_mulai, tanggal_selesai, jumlah_hari, alasan, tipe_cuti)
        VALUES ($1, $2, $3, $4, $5, $6::"TipeCuti")
        RETURNING id
        "#,
    )
    .bind(pegawai_id)
    .bind(payload.tanggal_mulai)
    .bind(payload.tanggal_selesai)
    .bind(payload.jumlah_hari)
    .bind(payload.alasan)
    .bind(tipe_cuti_str) // Kirim sebagai string
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    
    // Ambil data lengkap menggunakan helper
    get_pengajuan_cuti_by_id_repo(pool, new_id).await
}


/// Endpoint Atasan/Admin: Menyetujui pengajuan cuti
pub async fn approve_cuti_repo(
    pool: &DbPool,
    id: Uuid,
    user_approve_id: Uuid,
    payload: ApprovalCutiPayload,
) -> Result<PengajuanCuti, AppError> {
    let mut tx = pool.begin().await?;

    // 1. Ambil data pengajuan (gunakan helper _inner)
    let pengajuan = get_pengajuan_cuti_by_id_repo(&mut *tx, id).await?;
    if pengajuan.status != StatusCuti::Diajukan {
        return Err(AppError::Forbidden("Pengajuan cuti tidak ditemukan atau sudah diproses.".to_string()));
    }

    // 2. Jika tipenya "Paid", potong kuota
    if pengajuan.tipe_cuti == TipeCuti::Paid {
        let tahun_cuti = pengajuan.tanggal_mulai.year() as i16;
        
        let jatah = sqlx::query!(
            "SELECT kuota_total, kuota_terpakai FROM jatah_cuti WHERE pegawai_id = $1 AND tahun = $2 FOR UPDATE",
            pengajuan.pegawai_id,
            tahun_cuti
        ).fetch_optional(&mut *tx).await?;

        if let Some(jatah) = jatah {
            if (jatah.kuota_total - jatah.kuota_terpakai) < pengajuan.jumlah_hari {
                return Err(AppError::Forbidden("Gagal menyetujui: Kuota cuti pegawai sudah terpakai.".to_string()));
            }
            sqlx::query!(
                "UPDATE jatah_cuti SET kuota_terpakai = kuota_terpakai + $1 WHERE pegawai_id = $2 AND tahun = $3",
                pengajuan.jumlah_hari, pengajuan.pegawai_id, tahun_cuti
            ).execute(&mut *tx).await?;
        } else {
            return Err(AppError::Forbidden("Gagal menyetujui: Jatah cuti pegawai untuk tahun ini tidak ditemukan.".to_string()));
        }
    }

    // 3. Update status pengajuan
    let status_str = StatusCuti::Disetujui.as_str();
    
    // --- PERBAIKAN DI SINI: Ganti query_as! dengan query() ---
    sqlx::query(
        r#"
        UPDATE pengajuan_cuti
        SET status = $1::"StatusCuti", user_approve_id = $2, catatan_approval = $3, updated_at = now()
        WHERE id = $4 AND status = 'Diajukan'
        "#,
    )
    .bind(status_str)
    .bind(user_approve_id)
    .bind(payload.catatan)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    
    get_pengajuan_cuti_by_id_repo(pool, id).await
}

/// Endpoint Atasan/Admin: Menolak pengajuan cuti
pub async fn reject_cuti_repo(
    pool: &DbPool,
    id: Uuid,
    user_approve_id: Uuid,
    payload: ApprovalCutiPayload,
) -> Result<PengajuanCuti, AppError> {
    let status_str = StatusCuti::Ditolak.as_str();

    // --- PERBAIKAN DI SINI: Ganti query_as! dengan query() ---
    sqlx::query(
        r#"
        UPDATE pengajuan_cuti
        SET status = $1::"StatusCuti", user_approve_id = $2, catatan_approval = $3, updated_at = now()
        WHERE id = $4 AND status = 'Diajukan'
        "#,
    )
    .bind(status_str)
    .bind(user_approve_id)
    .bind(payload.catatan)
    .bind(id)
    .execute(pool) // Tidak perlu transaksi
    .await?;

    get_pengajuan_cuti_by_id_repo(pool, id).await
}

/// Endpoint Atasan/Admin: Melihat semua pengajuan cuti (bisa difilter)
pub async fn get_all_cuti_repo(pool: &DbPool) -> Result<Vec<PengajuanCuti>, AppError> {
    // TODO: Tambahkan filter jika perlu
    let list = sqlx::query_as!(
        PengajuanCuti,
        r#"
        SELECT id, pegawai_id, tanggal_mulai, tanggal_selesai, jumlah_hari, alasan,
               status as "status: _",
               tipe_cuti as "tipe_cuti: _", -- <-- TAMBAHKAN INI
               user_approve_id, catatan_approval, created_at
        FROM pengajuan_cuti
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}

pub async fn get_my_cuti_repo(pool: &DbPool, pegawai_id: Uuid) -> Result<Vec<PengajuanCuti>, AppError> {
    let list = sqlx::query_as!(
        PengajuanCuti,
        r#"
        SELECT id, pegawai_id, tanggal_mulai, tanggal_selesai, jumlah_hari, alasan,
               status as "status: _",
               tipe_cuti as "tipe_cuti: _",
               user_approve_id, catatan_approval, created_at
        FROM pengajuan_cuti
        WHERE pegawai_id = $1
        ORDER BY created_at DESC
        "#,
        pegawai_id
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}

pub async fn get_kuota_cuti_repo(pool: &DbPool, pegawai_id: Uuid, tahun: i16) -> Result<KuotaCutiDetail, AppError> {
    // 1. Coba ambil jatah cuti untuk tahun yang diminta
    let jatah = sqlx::query_as!(
        JatahCuti,
        "SELECT * FROM jatah_cuti WHERE pegawai_id = $1 AND tahun = $2",
        pegawai_id,
        tahun
    )
    .fetch_optional(pool)
    .await?;

    // 2. Buat respons. Jika tidak ada jatah (None), kembalikan 0.
    if let Some(jatah) = jatah {
        Ok(KuotaCutiDetail {
            kuota_total: jatah.kuota_total,
            kuota_terpakai: jatah.kuota_terpakai,
            sisa_cuti: jatah.kuota_total - jatah.kuota_terpakai,
            tahun: jatah.tahun,
        })
    } else {
        // Jika pegawai belum punya jatah di tahun itu, kembalikan data default
        Ok(KuotaCutiDetail {
            kuota_total: 0,
            kuota_terpakai: 0,
            sisa_cuti: 0,
            tahun,
        })
    }
}