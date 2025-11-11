// src/modules/sdm/surat_tugas_repo.rs
use super::{
    surat_tugas_model::{
        CreateSuratTugasPayload, PenerimaTugasDetail, SuratTugas,
        SuratTugasDetail, UpdateSuratTugasPayload
    },
};
use crate::{db::DbPool, errors::AppError};
use time::{OffsetDateTime}; 
use uuid::Uuid;

/// Helper internal untuk generate nomor surat baru (e.g., 001/ST/XI/2025)
/// Fungsi ini harus dipanggil di dalam transaksi
async fn generate_nomor_surat_repo(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    kode: &str, // 'ST' atau 'SPPD'
) -> Result<String, AppError> {
    let now = OffsetDateTime::now_utc();
    let year = now.year() as i16;
    let month_num = now.month() as u8;

    let month_romawi = match month_num {
        1 => "I", 2 => "II", 3 => "III", 4 => "IV", 5 => "V", 6 => "VI",
        7 => "VII", 8 => "VIII", 9 => "IX", 10 => "X", 11 => "XI", 12 => "XII",
        _ => "?",
    };

    let record = sqlx::query!(
        r#"
        INSERT INTO penomoran_surat_counter (kode, tahun, counter)
        VALUES ($1, $2, 1)
        ON CONFLICT (kode, tahun) DO UPDATE
        SET counter = penomoran_surat_counter.counter + 1
        RETURNING counter
        "#,
        kode,
        year
    )
    .fetch_one(&mut **tx)
    .await?;

    let nomor_urut = record.counter;
    // Format nomor surat, e.g., "001/ST/STIKES-R/XI/2025"
    let nomor_surat = format!("{:03}/{}/STIKES-R/{}/{}", nomor_urut, kode, month_romawi, year);

    Ok(nomor_surat)
}

/// Helper internal untuk mengambil detail lengkap satu Surat Tugas
pub async fn get_surat_tugas_detail_repo(
    pool: &DbPool,
    id: Uuid,
) -> Result<SuratTugasDetail, AppError> {
    
    // 1. Ambil data master surat (sebutkan semua kolom)
    let master = sqlx::query_as!(
        SuratTugas,
        r#"
        SELECT 
            id, nomor_surat, dasar_tugas, tugas, tempat_tugas, tanggal_mulai, 
            tanggal_selesai, penandatangan_id, tembusan, user_pembuat_id, 
            created_at, updated_at,
            -- Kolom SPPD baru
            nomor_sppd, alat_angkut, tempat_berangkat, lama_perjalanan,
            pembebanan_anggaran_instansi, pembebanan_anggaran_mak,
            ppk_pegawai_id, kpa_pegawai_id, keterangan_lain
        FROM surat_tugas_master 
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    // 2. Ambil data penandatangan
    let penandatangan = sqlx::query!(
        r#"SELECT p.nik, p.nama_lengkap, pp.jabatan as "jabatan?"
           FROM pegawai p
           LEFT JOIN penempatan_pegawai pp ON p.id = pp.pegawai_id AND pp.tanggal_selesai IS NULL
           WHERE p.id = $1"#,
        master.penandatangan_id
    )
    .fetch_one(pool)
    .await?;

    // 3. Ambil data PPK (jika ada)
    let ppk = if let Some(ppk_id) = master.ppk_pegawai_id {
        sqlx::query!("SELECT nama_lengkap FROM pegawai WHERE id = $1", ppk_id)
            .fetch_one(pool).await.ok()
    } else { None };
    
    // 4. Ambil data KPA (jika ada)
    let kpa = if let Some(kpa_id) = master.kpa_pegawai_id {
        sqlx::query!("SELECT nama_lengkap FROM pegawai WHERE id = $1", kpa_id)
            .fetch_one(pool).await.ok()
    } else { None };

    // 5. Ambil data penerima tugas (sudah termasuk peran)
    let penerima_list = sqlx::query_as!(
        PenerimaTugasDetail,
        r#"
        SELECT 
            p.id as "pegawai_id!",
            p.nama_lengkap as "nama_lengkap!",
            p.nik as "nip!",
            pp.jabatan as "jabatan?",
            NULL as "pangkat_golongan?", -- Placeholder
            stp.peran as "peran: _"
        FROM surat_tugas_penerima stp
        JOIN pegawai p ON stp.pegawai_id = p.id
        LEFT JOIN penempatan_pegawai pp ON p.id = pp.pegawai_id AND pp.tanggal_selesai IS NULL
        WHERE stp.surat_tugas_id = $1
        "#,
        id
    )
    .fetch_all(pool)
    .await?;

    // 6. Gabungkan semua
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
    jabatan_penandatangan: penandatangan.jabatan,
    nip_penandatangan: penandatangan.nik,
        daftar_penerima: penerima_list,
        tembusan: master.tembusan.unwrap_or_default(),
        created_at: master.created_at,
        // Data SPPD
        nomor_sppd: master.nomor_sppd,
        alat_angkut: master.alat_angkut,
        tempat_berangkat: master.tempat_berangkat,
        lama_perjalanan: master.lama_perjalanan,
        pembebanan_anggaran_instansi: master.pembebanan_anggaran_instansi,
        pembebanan_anggaran_mak: master.pembebanan_anggaran_mak,
        ppk_pegawai_id: master.ppk_pegawai_id,
        nama_ppk: ppk.map(|r| r.nama_lengkap),
        kpa_pegawai_id: master.kpa_pegawai_id,
        nama_kpa: kpa.map(|r| r.nama_lengkap),
        keterangan_lain: master.keterangan_lain,
    };

    Ok(detail)
}

/// Membuat Surat Tugas baru (termasuk data SPPD)
pub async fn create_surat_tugas_repo(
    pool: &DbPool,
    user_pembuat_id: Uuid,
    payload: CreateSuratTugasPayload,
) -> Result<SuratTugasDetail, AppError> {
    let mut tx = pool.begin().await?;

    // 1. Tentukan jenis surat (ST atau SPPD)
    let is_sppd = payload.ppk_pegawai_id.is_some() || payload.kpa_pegawai_id.is_some();
    
    // 2. Generate Nomor Surat
    let nomor_surat = generate_nomor_surat_repo(&mut tx, "ST").await?;
    let nomor_sppd = if is_sppd {
        Some(generate_nomor_surat_repo(&mut tx, "SPPD").await?)
    } else {
        None
    };

    // 3. Insert ke tabel master (tidak berubah)
    let new_id = sqlx::query_scalar(
        r#"
        INSERT INTO surat_tugas_master (
            nomor_surat, dasar_tugas, tugas, tempat_tugas, tanggal_mulai, 
            tanggal_selesai, penandatangan_id, tembusan, user_pembuat_id,
            nomor_sppd, alat_angkut, tempat_berangkat, lama_perjalanan,
            pembebanan_anggaran_instansi, pembebanan_anggaran_mak,
            ppk_pegawai_id, kpa_pegawai_id, keterangan_lain
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        RETURNING id
        "#,
    )
    .bind(nomor_surat).bind(payload.dasar_tugas).bind(payload.tugas).bind(payload.tempat_tugas)
    .bind(payload.tanggal_mulai).bind(payload.tanggal_selesai).bind(payload.penandatangan_id)
    .bind(payload.tembusan.as_deref()).bind(user_pembuat_id)
    .bind(nomor_sppd).bind(payload.alat_angkut).bind(payload.tempat_berangkat).bind(payload.lama_perjalanan)
    .bind(payload.pembebanan_anggaran_instansi).bind(payload.pembebanan_anggaran_mak)
    .bind(payload.ppk_pegawai_id).bind(payload.kpa_pegawai_id).bind(payload.keterangan_lain)
    .fetch_one(&mut *tx)
    .await?;

    // 4. Insert ke tabel penerima (Many-to-Many dengan Peran)
    // --- PERBAIKAN DI SINI ---
    for penerima in payload.penerima_tugas {
        sqlx::query( // Ganti dari `sqlx::query!` menjadi `sqlx::query()`
            "INSERT INTO surat_tugas_penerima (surat_tugas_id, pegawai_id, peran) VALUES ($1, $2, $3::\"PeranPerjalanan\")"
        )
        .bind(new_id)
        .bind(penerima.pegawai_id)
        .bind(penerima.peran.as_str()) // Bind string
        .execute(&mut *tx)
        .await?;
    }
    // --- AKHIR PERBAIKAN ---

    // 5. Selesaikan transaksi
    tx.commit().await?;
    
    get_surat_tugas_detail_repo(pool, new_id).await
}

/// Mengambil semua Surat Tugas (list ringan)
pub async fn get_all_surat_tugas_repo(pool: &DbPool) -> Result<Vec<SuratTugas>, AppError> {
    // HARUS menyebutkan semua kolom agar `query_as!` tidak gagal
    let list = sqlx::query_as!(
        SuratTugas,
        r#"
        SELECT 
            id, nomor_surat, dasar_tugas, tugas, tempat_tugas, tanggal_mulai, 
            tanggal_selesai, penandatangan_id, tembusan, user_pembuat_id, 
            created_at, updated_at,
            nomor_sppd, alat_angkut, tempat_berangkat, lama_perjalanan,
            pembebanan_anggaran_instansi, pembebanan_anggaran_mak,
            ppk_pegawai_id, kpa_pegawai_id, keterangan_lain
        FROM surat_tugas_master ORDER BY created_at DESC
        "#
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
    // Kita harus menyebutkan semua kolom secara eksplisit karena SELECT * bermasalah
    let old_data = sqlx::query_as!(
        SuratTugas,
        r#"
        SELECT 
            id, nomor_surat, dasar_tugas, tugas, tempat_tugas, tanggal_mulai, 
            tanggal_selesai, penandatangan_id, tembusan, user_pembuat_id, 
            created_at, updated_at,
            nomor_sppd, alat_angkut, tempat_berangkat, lama_perjalanan,
            pembebanan_anggaran_instansi, pembebanan_anggaran_mak,
            ppk_pegawai_id, kpa_pegawai_id, keterangan_lain
        FROM surat_tugas_master 
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(&mut *tx)
    .await?;

    // 2. Lakukan UPDATE dengan semua field
    sqlx::query(
        r#"
        UPDATE surat_tugas_master SET
            dasar_tugas = $1, tugas = $2, tempat_tugas = $3, tanggal_mulai = $4,
            tanggal_selesai = $5, penandatangan_id = $6, tembusan = $7,
            alat_angkut = $8, tempat_berangkat = $9, lama_perjalanan = $10,
            pembebanan_anggaran_instansi = $11, pembebanan_anggaran_mak = $12,
            ppk_pegawai_id = $13, kpa_pegawai_id = $14, keterangan_lain = $15,
            updated_at = now()
        WHERE id = $16
        "#,
    )
    .bind(payload.dasar_tugas.or(old_data.dasar_tugas))
    .bind(payload.tugas.unwrap_or(old_data.tugas)) // <-- Field ini sekarang dibaca
    .bind(payload.tempat_tugas.unwrap_or(old_data.tempat_tugas)) // <-- Field ini sekarang dibaca
    .bind(payload.tanggal_mulai.unwrap_or(old_data.tanggal_mulai)) // <-- Field ini sekarang dibaca
    .bind(payload.tanggal_selesai.unwrap_or(old_data.tanggal_selesai)) // <-- Field ini sekarang dibaca
    .bind(payload.penandatangan_id.unwrap_or(old_data.penandatangan_id)) // <-- Field ini sekarang dibaca
    .bind(payload.tembusan.as_deref()) // <-- Field ini sekarang dibaca
    // Bind SPPD
    .bind(payload.alat_angkut.or(old_data.alat_angkut)) // <-- Field ini sekarang dibaca
    .bind(payload.tempat_berangkat.or(old_data.tempat_berangkat)) // <-- Field ini sekarang dibaca
    .bind(payload.lama_perjalanan.or(old_data.lama_perjalanan)) // <-- Field ini sekarang dibaca
    .bind(payload.pembebanan_anggaran_instansi.or(old_data.pembebanan_anggaran_instansi)) // <-- Field ini sekarang dibaca
    .bind(payload.pembebanan_anggaran_mak.or(old_data.pembebanan_anggaran_mak)) // <-- Field ini sekarang dibaca
    .bind(payload.ppk_pegawai_id.or(old_data.ppk_pegawai_id)) // <-- Field ini sekarang dibaca
    .bind(payload.kpa_pegawai_id.or(old_data.kpa_pegawai_id)) // <-- Field ini sekarang dibaca
    .bind(payload.keterangan_lain.or(old_data.keterangan_lain)) // <-- Field ini sekarang dibaca
    // ID
    .bind(id)
    .execute(&mut *tx)
    .await?;

    // 3. Sinkronkan penerima tugas (jika ada di payload)
    if let Some(penerima_list) = payload.penerima_tugas {
        // Hapus semua penerima lama
        sqlx::query!("DELETE FROM surat_tugas_penerima WHERE surat_tugas_id = $1", id)
            .execute(&mut *tx).await?;
        
        // Tambahkan yang baru
        for penerima in penerima_list {
            sqlx::query(
                "INSERT INTO surat_tugas_penerima (surat_tugas_id, pegawai_id, peran) VALUES ($1, $2, $3::\"PeranPerjalanan\")"
            )
            .bind(id)
            .bind(penerima.pegawai_id)
            .bind(penerima.peran.as_str())
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
        .execute(pool).await?.rows_affected();
    if rows == 0 { return Err(sqlx::Error::RowNotFound.into()); }
    Ok(())
}