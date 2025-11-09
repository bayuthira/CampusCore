// src/modules/sdm/surat_tugas_repo.rs
use super::surat_tugas_model::{
    CreateSuratTugasPayload, PenerimaTugasDetail, SuratTugas, SuratTugasDetail,
    UpdateSuratTugasPayload,
};
use crate::{db::DbPool, errors::AppError};
use time::OffsetDateTime;
use uuid::Uuid;

/// Helper internal untuk generate nomor surat baru (e.g., 001/ST/XI/2025)
/// Fungsi ini harus dipanggil di dalam transaksi
async fn generate_nomor_surat_repo(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<String, AppError> {
    let now = OffsetDateTime::now_utc();
    let year = now.year() as i16;
    let month_num = now.month() as u8;

    // Konversi nomor bulan ke angka Romawi
    let month_romawi = match month_num {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        6 => "VI",
        7 => "VII",
        8 => "VIII",
        9 => "IX",
        10 => "X",
        11 => "XI",
        12 => "XII",
        _ => "?",
    };

    // 1. Kunci baris, ambil counter, dan update +1
    // 'FOR UPDATE' mengunci baris ini sampai transaksi selesai
    let record = sqlx::query!(
        r#"
        INSERT INTO penomoran_surat_counter (kode, tahun, counter)
        VALUES ('ST', $1, 1)
        ON CONFLICT (kode, tahun) DO UPDATE
        SET counter = penomoran_surat_counter.counter + 1
        RETURNING counter
        "#,
        year
    )
    .fetch_one(&mut **tx) // Perhatikan &mut **tx
    .await?;

    // 2. Format nomor surat
    let nomor_urut = record.counter;
    let nomor_surat = format!("{:03}/ST/STIKES-R/{}/{}", nomor_urut, month_romawi, year);

    Ok(nomor_surat)
}

/// Helper internal untuk mengambil detail lengkap satu Surat Tugas
async fn get_surat_tugas_detail_repo(
    pool: &DbPool,
    id: Uuid,
) -> Result<SuratTugasDetail, AppError> {
    // 1. Ambil data master surat
    let master = sqlx::query_as!(
        SuratTugas,
        "SELECT * FROM surat_tugas_master WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await?;

    // 2. Ambil data penandatangan (JOIN ke penempatan_pegawai)
    let penandatangan = sqlx::query!(
        r#"
        SELECT 
            p.nik, 
            p.nama_lengkap, 
            pp.jabatan
        FROM pegawai p
        -- JOIN ke penempatan yang sedang aktif
        LEFT JOIN penempatan_pegawai pp ON p.id = pp.pegawai_id AND pp.tanggal_selesai IS NULL
        WHERE p.id = $1
        "#,
        master.penandatangan_id
    )
    .fetch_one(pool)
    .await?;

    // 3. Ambil data penerima tugas (JOIN ke penempatan_pegawai)
    let penerima_list = sqlx::query_as!(
        PenerimaTugasDetail,
        r#"
        SELECT 
            p.id as "pegawai_id!",
            p.nama_lengkap as "nama_lengkap!",
            p.nik as "nip!",
            pp.jabatan,
            NULL as "pangkat_golongan" -- Placeholder
        FROM surat_tugas_penerima stp
        JOIN pegawai p ON stp.pegawai_id = p.id
        -- JOIN ke penempatan yang sedang aktif
        LEFT JOIN penempatan_pegawai pp ON p.id = pp.pegawai_id AND pp.tanggal_selesai IS NULL
        WHERE stp.surat_tugas_id = $1
        "#,
        id
    )
    .fetch_all(pool)
    .await?;

    // 4. Gabungkan semua
    let detail = SuratTugasDetail {
        id: master.id,
        nomor_surat: master.nomor_surat,
        dasar_tugas: master.dasar_tugas,
        tugas: master.tugas,
        tempat_tugas: master.tempat_tugas,
        tanggal_mulai: master.tanggal_mulai,
        tanggal_selesai: master.tanggal_selesai,
        penandatangan_id: master.penandatangan_id,
        nama_penandatangan: penandatangan.nama_lengkap,
        jabatan_penandatangan: Some(penandatangan.jabatan),
        nip_penandatangan: penandatangan.nik,
        daftar_penerima: penerima_list,
        tembusan: master.tembusan.unwrap_or_default(),
        created_at: master.created_at,
    };

    Ok(detail)
}

/// Membuat Surat Tugas baru
pub async fn create_surat_tugas_repo(
    pool: &DbPool,
    user_pembuat_id: Uuid,
    payload: CreateSuratTugasPayload,
) -> Result<SuratTugasDetail, AppError> {
    let mut tx = pool.begin().await?;

    // 1. Generate Nomor Surat
    let nomor_surat = generate_nomor_surat_repo(&mut tx).await?;

    // 2. Insert ke tabel master
    let new_id = sqlx::query_scalar!(
        r#"
        INSERT INTO surat_tugas_master (
            nomor_surat, dasar_tugas, tugas, tempat_tugas, tanggal_mulai, 
            tanggal_selesai, penandatangan_id, tembusan, user_pembuat_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
        nomor_surat,
        payload.dasar_tugas,
        payload.tugas,
        payload.tempat_tugas,
        payload.tanggal_mulai,
        payload.tanggal_selesai,
        payload.penandatangan_id,
        payload.tembusan.as_deref(), // Konversi Option<Vec<String>> ke Option<&[String]>
        user_pembuat_id
    )
    .fetch_one(&mut *tx)
    .await?;

    // 3. Insert ke tabel penerima (Many-to-Many)
    for pegawai_id in payload.penerima_tugas_ids {
        sqlx::query!(
            "INSERT INTO surat_tugas_penerima (surat_tugas_id, pegawai_id) VALUES ($1, $2)",
            new_id,
            pegawai_id
        )
        .execute(&mut *tx)
        .await?;
    }

    // 4. Selesaikan transaksi
    tx.commit().await?;

    // Ambil dan kembalikan data lengkap
    let detail = get_surat_tugas_detail_repo(pool, new_id).await?;
    Ok(detail)
}

/// Mengambil semua Surat Tugas (list ringan)
pub async fn get_all_surat_tugas_repo(pool: &DbPool) -> Result<Vec<SuratTugas>, AppError> {
    let list = sqlx::query_as!(
        SuratTugas,
        "SELECT * FROM surat_tugas_master ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}

/// Mengupdate Surat Tugas
pub async fn update_surat_tugas_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateSuratTugasPayload,
) -> Result<SuratTugasDetail, AppError> {
    let mut tx = pool.begin().await?;

    // 1. Ambil data lama
    let old_data = sqlx::query_as!(
        SuratTugas,
        "SELECT * FROM surat_tugas_master WHERE id = $1",
        id
    )
    .fetch_one(&mut *tx)
    .await?;

    // 2. Lakukan UPDATE
    sqlx::query!(
        r#"
        UPDATE surat_tugas_master SET
            dasar_tugas = $1,
            tugas = $2,
            tempat_tugas = $3,
            tanggal_mulai = $4,
            tanggal_selesai = $5,
            penandatangan_id = $6,
            tembusan = $7,
            updated_at = now()
        WHERE id = $8
        "#,
        payload
            .dasar_tugas
            .unwrap_or(old_data.dasar_tugas.unwrap_or_default()),
        payload.tugas.unwrap_or(old_data.tugas),
        payload.tempat_tugas.unwrap_or(old_data.tempat_tugas),
        payload.tanggal_mulai.unwrap_or(old_data.tanggal_mulai),
        payload.tanggal_selesai.unwrap_or(old_data.tanggal_selesai),
        payload
            .penandatangan_id
            .unwrap_or(old_data.penandatangan_id),
        payload.tembusan.as_deref(), // Ini akan me-replace
        id
    )
    .execute(&mut *tx)
    .await?;

    // 3. Sinkronkan penerima tugas (jika ada di payload)
    if let Some(penerima_ids) = payload.penerima_tugas_ids {
        // Hapus semua penerima lama
        sqlx::query!(
            "DELETE FROM surat_tugas_penerima WHERE surat_tugas_id = $1",
            id
        )
        .execute(&mut *tx)
        .await?;

        // Tambahkan yang baru
        for pegawai_id in penerima_ids {
            sqlx::query!(
                "INSERT INTO surat_tugas_penerima (surat_tugas_id, pegawai_id) VALUES ($1, $2)",
                id,
                pegawai_id
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    get_surat_tugas_detail_repo(pool, id).await
}

/// Menghapus Surat Tugas
pub async fn delete_surat_tugas_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    // ON DELETE CASCADE akan otomatis menghapus data di surat_tugas_penerima
    let rows = sqlx::query!("DELETE FROM surat_tugas_master WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}
