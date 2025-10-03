// src/modules/akademik/jadwal_kuliah_repo.rs
use super::jadwal_kuliah_model::{
    CreateJadwalKuliahPayload, DayOfWeek, DosenPengampuDetail, JadwalKuliahDetail,
    JadwalKuliahFilter, PlotJadwalRuanganPayload, UpdateJadwalKuliahPayload,TimeWithOffset
};
use crate::{db::DbPool, errors::AppError};
use sqlx::FromRow;
use time::{Duration,Weekday};
use uuid::Uuid;

pub async fn create_jadwal_kuliah_repo(
    pool: &DbPool,
    payload: CreateJadwalKuliahPayload,
) -> Result<Uuid, AppError> {
    let mut tx = pool.begin().await?;
    let hari_str = payload.hari.as_str();
    
    // Validasi dosen bentrok
    for dosen in &payload.dosen_pengampu {
        // 1. Ambil nama dosen untuk pesan error yang lebih jelas
        let dosen_record =
            sqlx::query!("SELECT nama_dosen FROM dosen WHERE id = $1", dosen.dosen_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| {
                    AppError::Forbidden(format!(
                        "Dosen dengan ID {} tidak ditemukan.",
                        dosen.dosen_id
                    ))
                })?;
        let nama_dosen_bentrok = dosen_record.nama_dosen;
        
        // 2. Cek konflik jadwal
        // Use query() instead of query!() since we're using custom types
        let conflict = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM jadwal_kuliah jk
                JOIN jadwal_dosen_pengampu jdp ON jk.id = jdp.jadwal_kuliah_id
                WHERE jdp.dosen_id = $1
                  AND jk.tahun_akademik_id = $2
                  AND jk.hari::TEXT = $3
                  AND (jk.jam_mulai, jk.jam_selesai) OVERLAPS ($4, $5)
            )
            "#,
        )
        .bind(dosen.dosen_id)
        .bind(payload.tahun_akademik_id)
        .bind(hari_str)
        .bind(&payload.jam_mulai) // Bind TimeWithOffset directly
        .bind(&payload.jam_selesai) // Bind TimeWithOffset directly
        .fetch_one(&mut *tx)
        .await?;
        
        // 3. Jika bentrok, kembalikan pesan error yang spesifik
        if conflict {
            return Err(AppError::Forbidden(format!(
                "Jadwal bentrok untuk dosen '{}' pada hari {} jam tersebut.",
                nama_dosen_bentrok, hari_str
            )));
        }
    }
    
    // Insert jadwal kuliah
    let jadwal_id = sqlx::query_scalar(
        r#"
        INSERT INTO jadwal_kuliah 
        (matakuliah_id, tahun_akademik_id, hari, jam_mulai, jam_selesai, kelas)
        VALUES ($1, $2, $3::"DayOfWeek", $4, $5, $6) RETURNING id
        "#,
    )
    .bind(payload.matakuliah_id)
    .bind(payload.tahun_akademik_id)
    .bind(hari_str)
    .bind(&payload.jam_mulai) // TimeWithOffset implements Encode
    .bind(&payload.jam_selesai) // TimeWithOffset implements Encode
    .bind(payload.kelas)
    .fetch_one(&mut *tx)
    .await?;
    
    for dosen in payload.dosen_pengampu {
        let peran_str = dosen.peran.as_str();
        sqlx::query(
            "INSERT INTO jadwal_dosen_pengampu (jadwal_kuliah_id, dosen_id, peran) VALUES ($1, $2, $3::\"PeranDosenPengampu\")",
        )
        .bind(jadwal_id)
        .bind(dosen.dosen_id)
        .bind(peran_str)
        .execute(&mut *tx)
        .await?;
    }
    
    tx.commit().await?;
    Ok(jadwal_id)
}


pub async fn plot_jadwal_ruangan_repo(
    pool: &DbPool,
    user_pembuat_id: Uuid,
    payload: PlotJadwalRuanganPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    
    // Hapus dulu jadwal
    sqlx::query!(
        "DELETE FROM jadwal_ruangan WHERE jadwal_kuliah_id = $1",
        payload.jadwal_kuliah_id
    )
    .execute(&mut *tx)
    .await?;
    
    // 1. Ambil detail jadwal kuliah dan tahun akademiknya
    #[derive(sqlx::FromRow)]
    struct JadwalInfo {
        hari: String,
        jam_mulai: TimeWithOffset,
        jam_selesai: TimeWithOffset,
        nama_mk: String,
        tanggal_mulai: time::Date,
        tanggal_selesai: time::Date,
    }
    
    let jadwal = sqlx::query_as::<_, JadwalInfo>(
        r#"
        SELECT jk.hari::TEXT as hari, jk.jam_mulai, jk.jam_selesai,
               mk.nama_mk, ta.tanggal_mulai, ta.tanggal_selesai
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON jk.matakuliah_id = mk.id
        JOIN tahun_akademik ta ON jk.tahun_akademik_id = ta.id
        WHERE jk.id = $1
        "#,
    )
    .bind(payload.jadwal_kuliah_id)
    .fetch_one(&mut *tx)
    .await?;
    
    // 2. Hitung semua tanggal pertemuan selama satu semester
    let mut instances_to_create = Vec::new();
    let mut current_date = jadwal.tanggal_mulai;
    let target_weekday = match jadwal.hari.as_str() {
        "Senin" => Weekday::Monday,
        "Selasa" => Weekday::Tuesday,
        "Rabu" => Weekday::Wednesday,
        "Kamis" => Weekday::Thursday,
        "Jumat" => Weekday::Friday,
        "Sabtu" => Weekday::Saturday,
        _ => Weekday::Sunday,
    };
    
    while current_date <= jadwal.tanggal_selesai {
        if current_date.weekday() == target_weekday {
            // Use .time to get the Time part from TimeWithOffset
            let waktu_mulai = current_date
                .with_time(jadwal.jam_mulai.time)
                .assume_offset(jadwal.jam_mulai.offset);
            let waktu_selesai = current_date
                .with_time(jadwal.jam_selesai.time)
                .assume_offset(jadwal.jam_selesai.offset);
            
            instances_to_create.push((waktu_mulai, waktu_selesai));
        }
        current_date += Duration::days(1);
    }
    
    // 3. Cek konflik & insert semua instance
    for (start_time, end_time) in instances_to_create {
        let conflict = sqlx::query_scalar!(
            "SELECT EXISTS (SELECT 1 FROM jadwal_ruangan WHERE ruangan_id = $1 AND (waktu_mulai, waktu_selesai) OVERLAPS ($2, $3))",
            payload.ruangan_id, start_time, end_time
        ).fetch_one(&mut *tx).await?;
        
        if conflict.unwrap_or(false) {
            return Err(AppError::Forbidden(format!(
                "Konflik jadwal untuk ruangan pada {}.",
                start_time.date()
            )));
        }
        
        sqlx::query!(
            "INSERT INTO jadwal_ruangan (ruangan_id, judul_kegiatan, jadwal_kuliah_id, waktu_mulai, waktu_selesai, user_pembuat_id) VALUES ($1, $2, $3, $4, $5, $6)",
            payload.ruangan_id, jadwal.nama_mk, payload.jadwal_kuliah_id, start_time, end_time, user_pembuat_id
        ).execute(&mut *tx).await?;
    }
    
    tx.commit().await?;
    Ok(())
}


pub async fn unplot_jadwal_ruangan_repo(pool: &DbPool, jadwal_kuliah_id: Uuid) -> Result<(), AppError> {
    // Hapus semua entri di `jadwal_ruangan` yang terkait dengan `jadwal_kuliah_id` ini
    sqlx::query!(
        "DELETE FROM jadwal_ruangan WHERE jadwal_kuliah_id = $1",
        jadwal_kuliah_id
    )
    .execute(pool)
    .await?;

    Ok(())
}



#[derive(FromRow)]
struct JadwalKuliahRow {
    id: Uuid,
    kelas: String,
    hari: DayOfWeek,
    jam_mulai: TimeWithOffset,  
    jam_selesai: TimeWithOffset, 
    matakuliah_id: Uuid,
    nama_mk: String,
    kode_mk: String,
    sks: i32,
    prodi_id: Uuid,
    nama_prodi: String,
    tahun_akademik_id: Uuid,
    nama_tahun_akademik: String,
    ruangan_id: Option<Uuid>,
    nama_ruangan: Option<String>,
}

pub async fn get_all_jadwal_kuliah_repo(
    pool: &DbPool,
    filter: JadwalKuliahFilter,
) -> Result<Vec<JadwalKuliahDetail>, AppError> {
    // Bangun query dasar
    let mut query_builder = sqlx::QueryBuilder::new(
        r#"
        SELECT 
            jk.id, jk.kelas, jk.hari, jk.jam_mulai, jk.jam_selesai,
            mk.id as matakuliah_id, mk.nama_mk, mk.kode_mk, mk.sks,
            p.id as prodi_id, p.nama_prodi,
            ta.id as tahun_akademik_id, ta.nama as nama_tahun_akademik,
            jr.ruangan_id, r.nama_ruangan
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON jk.matakuliah_id = mk.id
        JOIN prodi p ON mk.prodi_id = p.id
        JOIN tahun_akademik ta ON jk.tahun_akademik_id = ta.id
        LEFT JOIN jadwal_ruangan jr ON jk.id = jr.jadwal_kuliah_id
        LEFT JOIN ruangan r ON jr.ruangan_id = r.id
        WHERE 1=1
    "#,
    );
    
    // Tambahkan filter jika ada
    if let Some(prodi_id) = filter.prodi_id {
        query_builder.push(" AND p.id = ");
        query_builder.push_bind(prodi_id);
    }
    if let Some(ta_id) = filter.tahun_akademik_id {
        query_builder.push(" AND ta.id = ");
        query_builder.push_bind(ta_id);
    }
    
    query_builder.push(" GROUP BY jk.id, mk.id, p.id, ta.id, jr.ruangan_id, r.nama_ruangan");
    
    // 1. Eksekusi query utama untuk mendapatkan data jadwal
    let jadwal_rows = query_builder
        .build_query_as::<JadwalKuliahRow>()
        .fetch_all(pool)
        .await?;
    
    let mut jadwal_details = Vec::new();
    
    // 2. Loop untuk setiap jadwal, ambil data dosen pengampunya
    for row in jadwal_rows {
        let dosen_pengampu = sqlx::query_as!(
            DosenPengampuDetail,
            r#"
            SELECT d.id as dosen_id, d.nama_dosen, jdp.peran as "peran: _"
            FROM jadwal_dosen_pengampu jdp
            JOIN dosen d ON jdp.dosen_id = d.id
            WHERE jdp.jadwal_kuliah_id = $1
            "#,
            row.id
        )
        .fetch_all(pool)
        .await?;
        
        jadwal_details.push(JadwalKuliahDetail {
            id: row.id,
            kelas: row.kelas,
            hari: row.hari,
            jam_mulai: row.jam_mulai,
            jam_selesai: row.jam_selesai,
            matakuliah_id: row.matakuliah_id,
            nama_mk: row.nama_mk,
            kode_mk: row.kode_mk,
            sks: row.sks,
            prodi_id: row.prodi_id,
            nama_prodi: row.nama_prodi,
            tahun_akademik_id: row.tahun_akademik_id,
            nama_tahun_akademik: row.nama_tahun_akademik,
            dosen_pengampu,
            ruangan_id: row.ruangan_id,
            nama_ruangan: row.nama_ruangan,
        });
    }
    
    Ok(jadwal_details)
}

pub async fn update_jadwal_kuliah_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateJadwalKuliahPayload,
) -> Result<Uuid, AppError> {
    let mut tx = pool.begin().await?;
    let hari_str = payload.hari.as_str();
    
    // Pastikan tidak ada jadwal lain (selain diri sendiri) yang memiliki kombinasi ini.
    let is_duplicate = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM jadwal_kuliah WHERE matakuliah_id = $1 AND tahun_akademik_id = $2 AND kelas = $3 AND id != $4)",
        payload.matakuliah_id, payload.tahun_akademik_id, payload.kelas, id
    ).fetch_one(&mut *tx).await?;
    
    if is_duplicate.unwrap_or(false) {
        return Err(AppError::DuplicateEntry(
            "Jadwal untuk mata kuliah, tahun akademik, dan kelas ini sudah ada.".to_string(),
        ));
    }
    
    // Validasi dosen bentrok
    for dosen in &payload.dosen_pengampu {
        // Ambil nama dosen
        let dosen_record =
            sqlx::query!("SELECT nama_dosen FROM dosen WHERE id = $1", dosen.dosen_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| {
                    AppError::Forbidden(format!(
                        "Dosen dengan ID {} tidak ditemukan.",
                        dosen.dosen_id
                    ))
                })?;
        let nama_dosen_bentrok = dosen_record.nama_dosen;
        
        // Cek konflik jadwal - use query() instead of query!()
        let conflict = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM jadwal_kuliah jk
                JOIN jadwal_dosen_pengampu jdp ON jk.id = jdp.jadwal_kuliah_id
                WHERE jdp.dosen_id = $1
                  AND jk.tahun_akademik_id = $2
                  AND jk.hari::TEXT = $3
                  AND (jk.jam_mulai, jk.jam_selesai) OVERLAPS ($4, $5)
                  AND jk.id != $6
            )
            "#,
        )
        .bind(dosen.dosen_id)
        .bind(payload.tahun_akademik_id)
        .bind(hari_str)
        .bind(&payload.jam_mulai)  // Bind TimeWithOffset
        .bind(&payload.jam_selesai) // Bind TimeWithOffset
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        
        if conflict {
            return Err(AppError::Forbidden(format!(
                "Jadwal bentrok untuk dosen '{}' pada hari {} jam tersebut.",
                nama_dosen_bentrok, hari_str
            )));
        }
    }
    
    // 1. Update data utama
    sqlx::query(
        r#"
        UPDATE jadwal_kuliah
        SET matakuliah_id=$1, tahun_akademik_id=$2, hari=$3::"DayOfWeek",
            jam_mulai=$4, jam_selesai=$5, kelas=$6, updated_at=now()
        WHERE id=$7
        "#,
    )
    .bind(payload.matakuliah_id)
    .bind(payload.tahun_akademik_id)
    .bind(hari_str)
    .bind(&payload.jam_mulai)
    .bind(&payload.jam_selesai)
    .bind(payload.kelas)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    
    // 2. Sinkronisasi dosen pengampu (hapus yang lama)
    sqlx::query!(
        "DELETE FROM jadwal_dosen_pengampu WHERE jadwal_kuliah_id = $1",
        id
    )
    .execute(&mut *tx)
    .await?;
    
    // 3. Masukkan yang baru
    for dosen in payload.dosen_pengampu {
        let peran_str = dosen.peran.as_str();
        sqlx::query(
            "INSERT INTO jadwal_dosen_pengampu (jadwal_kuliah_id, dosen_id, peran) VALUES ($1, $2, $3::\"PeranDosenPengampu\")",
        )
        .bind(id)
        .bind(dosen.dosen_id)
        .bind(peran_str)
        .execute(&mut *tx)
        .await?;
    }
    
    tx.commit().await?;
    Ok(id)
}

pub async fn delete_jadwal_kuliah_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    // ON DELETE CASCADE akan otomatis menghapus relasi di jadwal_dosen_pengampu
    let rows_affected = sqlx::query!("DELETE FROM jadwal_kuliah WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}
