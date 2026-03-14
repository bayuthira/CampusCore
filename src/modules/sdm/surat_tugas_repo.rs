// src/modules/sdm/surat_tugas_repo.rs
use super::surat_tugas_model::{
    CreateSuratTugasPayload, PenerimaTugasDetail, SuratTugas, SuratTugasDetail,
    UpdateSuratTugasPayload,
};
use crate::{db::DbPool, errors::AppError};
use time::OffsetDateTime;
use uuid::Uuid;

async fn generate_nomor_surat_repo(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    kode: &str, // 'ST' atau 'SPPD'
) -> Result<String, AppError> {
    let now = OffsetDateTime::now_utc();
    let year = now.year() as i16;
    let month_num = now.month() as u8;

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
    let nomor_surat = format!(
        "{:03}/{}/STIKES-R/{}/{}",
        nomor_urut, kode, month_romawi, year
    );

    Ok(nomor_surat)
}

pub async fn get_surat_tugas_detail_repo(
    pool: &DbPool,
    id: Uuid,
) -> Result<SuratTugasDetail, AppError> {
    let master = sqlx::query_as!(
        SuratTugas,
        r#"
        SELECT 
            id, nomor_surat, dasar_tugas, tugas, tempat_tugas, tanggal_mulai, 
            tanggal_selesai, penandatangan_id, tembusan, user_pembuat_id, 
            created_at, updated_at,
            nomor_sppd, alasan_perjalanan, tujuan_kota, alat_angkut, tempat_berangkat, lama_perjalanan,
            pembebanan_anggaran_instansi, pembebanan_anggaran_mak,
            ppk_pegawai_id, kpa_pegawai_id, keterangan_lain
        FROM surat_tugas_master 
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    let penandatangan = sqlx::query!(
        r#"SELECT p.nik, p.nama_lengkap, pp.jabatan as "jabatan?"
           FROM pegawai p
           LEFT JOIN penempatan_pegawai pp ON p.id = pp.pegawai_id AND pp.tanggal_selesai IS NULL
           WHERE p.id = $1"#,
        master.penandatangan_id
    )
    .fetch_one(pool)
    .await?;

    let ppk = if let Some(ppk_id) = master.ppk_pegawai_id {
        sqlx::query!("SELECT nama_lengkap FROM pegawai WHERE id = $1", ppk_id)
            .fetch_one(pool)
            .await
            .ok()
    } else {
        None
    };

    let kpa = if let Some(kpa_id) = master.kpa_pegawai_id {
        sqlx::query!("SELECT nama_lengkap FROM pegawai WHERE id = $1", kpa_id)
            .fetch_one(pool)
            .await
            .ok()
    } else {
        None
    };

    let penerima_list = sqlx::query_as!(
        PenerimaTugasDetail,
        r#"
        SELECT 
            p.id as "pegawai_id!",
            p.nama_lengkap as "nama_lengkap!",
            p.nik as "nip!",
            pp.jabatan as "jabatan?",
            uk.nama_unit as "unit_kerja?",
            NULL as "pangkat_golongan?",
            stp.peran as "peran: _"
        FROM surat_tugas_penerima stp
        JOIN pegawai p ON stp.pegawai_id = p.id
        LEFT JOIN penempatan_pegawai pp ON p.id = pp.pegawai_id AND pp.tanggal_selesai IS NULL
        LEFT JOIN unit_kerja uk ON pp.unit_kerja_id = uk.id
        WHERE stp.surat_tugas_id = $1
        "#,
        id
    )
    .fetch_all(pool)
    .await?;

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
        nomor_sppd: master.nomor_sppd,
        alasan_perjalanan: master.alasan_perjalanan,
        tujuan_kota: master.tujuan_kota,
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

pub async fn create_surat_tugas_repo(
    pool: &DbPool,
    user_pembuat_id: Uuid,
    payload: CreateSuratTugasPayload,
) -> Result<SuratTugasDetail, AppError> {
    let mut tx = pool.begin().await?;

    // 1. Tentukan jenis surat (ST atau SPPD)
    let is_sppd = payload.ppk_pegawai_id.is_some() || payload.kpa_pegawai_id.is_some();

    let (nomor_surat, nomor_sppd) = if is_sppd {
        (
            None,
            Some(generate_nomor_surat_repo(&mut tx, "SPPD").await?),
        )
    } else {
        (Some(generate_nomor_surat_repo(&mut tx, "ST").await?), None)
    };

    let new_id = sqlx::query_scalar(
        r#"
        INSERT INTO surat_tugas_master (
            nomor_surat, dasar_tugas, tugas, tempat_tugas, tanggal_mulai, 
            tanggal_selesai, penandatangan_id, tembusan, user_pembuat_id,
            nomor_sppd, alasan_perjalanan, tujuan_kota, alat_angkut, tempat_berangkat, lama_perjalanan,
            pembebanan_anggaran_instansi, pembebanan_anggaran_mak,
            ppk_pegawai_id, kpa_pegawai_id, keterangan_lain
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        RETURNING id
        "#,
    )
    .bind(nomor_surat).bind(payload.dasar_tugas).bind(payload.tugas).bind(payload.tempat_tugas)
    .bind(payload.tanggal_mulai).bind(payload.tanggal_selesai).bind(payload.penandatangan_id)
    .bind(payload.tembusan.as_deref()).bind(user_pembuat_id)
    .bind(nomor_sppd).bind(payload.alasan_perjalanan).bind(payload.tujuan_kota)
    .bind(payload.alat_angkut).bind(payload.tempat_berangkat).bind(payload.lama_perjalanan)
    .bind(payload.pembebanan_anggaran_instansi).bind(payload.pembebanan_anggaran_mak)
    .bind(payload.ppk_pegawai_id).bind(payload.kpa_pegawai_id).bind(payload.keterangan_lain)
    .fetch_one(&mut *tx)
    .await?;

    for penerima in payload.penerima_tugas {
        sqlx::query(
            "INSERT INTO surat_tugas_penerima (surat_tugas_id, pegawai_id, peran) VALUES ($1, $2, $3::\"PeranPerjalanan\")"
        )
        .bind(new_id)
        .bind(penerima.pegawai_id)
        .bind(penerima.peran.as_str())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    get_surat_tugas_detail_repo(pool, new_id).await
}

pub async fn get_all_surat_tugas_repo(pool: &DbPool) -> Result<Vec<SuratTugas>, AppError> {
    let list = sqlx::query_as!(
        SuratTugas,
        r#"
        SELECT 
            id, nomor_surat, dasar_tugas, tugas, tempat_tugas, tanggal_mulai, 
            tanggal_selesai, penandatangan_id, tembusan, user_pembuat_id, 
            created_at, updated_at,
            nomor_sppd, alasan_perjalanan, tujuan_kota, alat_angkut, tempat_berangkat, lama_perjalanan,
            pembebanan_anggaran_instansi, pembebanan_anggaran_mak,
            ppk_pegawai_id, kpa_pegawai_id, keterangan_lain
        FROM surat_tugas_master ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}

pub async fn update_surat_tugas_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateSuratTugasPayload,
) -> Result<SuratTugasDetail, AppError> {
    let mut tx = pool.begin().await?;

    let old_data = sqlx::query_as!(
        SuratTugas,
        r#"
        SELECT 
            id, nomor_surat, dasar_tugas, tugas, tempat_tugas, tanggal_mulai, 
            tanggal_selesai, penandatangan_id, tembusan, user_pembuat_id, 
            created_at, updated_at,
            nomor_sppd, alasan_perjalanan, tujuan_kota, alat_angkut, tempat_berangkat, lama_perjalanan,
            pembebanan_anggaran_instansi, pembebanan_anggaran_mak,
            ppk_pegawai_id, kpa_pegawai_id, keterangan_lain
        FROM surat_tugas_master 
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE surat_tugas_master SET
            dasar_tugas = $1, tugas = $2, tempat_tugas = $3, tanggal_mulai = $4,
            tanggal_selesai = $5, penandatangan_id = $6, tembusan = $7,
            alasan_perjalanan = $8, tujuan_kota = $9, alat_angkut = $10, tempat_berangkat = $11, lama_perjalanan = $12,
            pembebanan_anggaran_instansi = $13, pembebanan_anggaran_mak = $14,
            ppk_pegawai_id = $15, kpa_pegawai_id = $16, keterangan_lain = $17,
            updated_at = now()
        WHERE id = $18
        "#,
    )
    .bind(payload.dasar_tugas.or(old_data.dasar_tugas))
    .bind(payload.tugas.unwrap_or(old_data.tugas))
    .bind(payload.tempat_tugas.unwrap_or(old_data.tempat_tugas))
    .bind(payload.tanggal_mulai.unwrap_or(old_data.tanggal_mulai))
    .bind(payload.tanggal_selesai.unwrap_or(old_data.tanggal_selesai))
    .bind(payload.penandatangan_id.unwrap_or(old_data.penandatangan_id))
    .bind(payload.tembusan.as_deref())
    .bind(payload.alasan_perjalanan.or(old_data.alasan_perjalanan))
    .bind(payload.tujuan_kota.or(old_data.tujuan_kota))
    .bind(payload.alat_angkut.or(old_data.alat_angkut))
    .bind(payload.tempat_berangkat.or(old_data.tempat_berangkat))
    .bind(payload.lama_perjalanan.or(old_data.lama_perjalanan))
    .bind(payload.pembebanan_anggaran_instansi.or(old_data.pembebanan_anggaran_instansi))
    .bind(payload.pembebanan_anggaran_mak.or(old_data.pembebanan_anggaran_mak))
    .bind(payload.ppk_pegawai_id.or(old_data.ppk_pegawai_id))
    .bind(payload.kpa_pegawai_id.or(old_data.kpa_pegawai_id))
    .bind(payload.keterangan_lain.or(old_data.keterangan_lain))
    .bind(id)
    .execute(&mut *tx)
    .await?;

    if let Some(penerima_list) = payload.penerima_tugas {
        sqlx::query!(
            "DELETE FROM surat_tugas_penerima WHERE surat_tugas_id = $1",
            id
        )
        .execute(&mut *tx)
        .await?;

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

pub async fn delete_surat_tugas_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows = sqlx::query!("DELETE FROM surat_tugas_master WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}
