// src/modules/mahasiswa/repo.rs
use super::model::{
    CreateMahasiswaPayload, ImportResult, MahasiswaCsvRecord, MahasiswaDetail,
    UpdateMahasiswaPayload,
};
use crate::db::DbPool;
use crate::errors::AppError;
use bytes::Bytes;
use uuid::Uuid;

pub async fn create_mahasiswa_repo(
    pool: &DbPool,
    payload: CreateMahasiswaPayload,
) -> Result<MahasiswaDetail, AppError> {
    let mut tx = pool.begin().await?;

    let hashed_password = bcrypt::hash(payload.password, bcrypt::DEFAULT_COST)?;

    // 1. Coba buat user
    let user_insert_result = sqlx::query!(
        "INSERT INTO users (username, password_hash, full_name, email) VALUES ($1, $2, $3, $4) RETURNING id",
        payload.nim, hashed_password, payload.nama_mahasiswa, payload.email
    )
    .fetch_one(&mut *tx)
    .await;

    let new_user_id = match user_insert_result {
        Ok(record) => record.id,
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    let constraint = db_err.constraint().unwrap_or_default();
                    if constraint.contains("users_username_key") {
                        return Err(AppError::DuplicateEntry(format!(
                            "NIM '{}' sudah terdaftar.",
                            payload.nim
                        )));
                    } else if constraint.contains("users_email_key") {
                        return Err(AppError::DuplicateEntry(format!(
                            "Email '{}' sudah terdaftar.",
                            payload.email.unwrap_or_default()
                        )));
                    }
                }
            }
            return Err(e.into());
        }
    };

    // 2. Beri role MAHASISWA
    sqlx::query!(
        "INSERT INTO user_roles (user_id, role_id) VALUES ($1, (SELECT id FROM roles WHERE name = 'MAHASISWA'))",
        new_user_id
    )
    .execute(&mut *tx)
    .await?;

    // 3. Insert ke tabel mahasiswa (BIODATA MURNI)
    let mhs_insert_result = sqlx::query_scalar!(
        r#"
        INSERT INTO mahasiswa (nik, nama_mahasiswa, email, tempat_lahir, tanggal_lahir, nama_ibu_kandung, user_id) 
        VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id
        "#,
        payload.nik, payload.nama_mahasiswa, payload.email, payload.tempat_lahir, payload.tanggal_lahir, payload.nama_ibu_kandung, new_user_id
    )
    .fetch_one(&mut *tx)
    .await;

    let new_mahasiswa_id = match mhs_insert_result {
        Ok(id) => id,
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation()
                    && db_err
                        .constraint()
                        .unwrap_or_default()
                        .contains("mahasiswa_nik_key")
                {
                    return Err(AppError::DuplicateEntry(format!(
                        "NIK '{}' sudah terdaftar di sistem.",
                        payload.nik
                    )));
                }
            }
            return Err(e.into());
        }
    };

    // 4. Insert ke tabel registrasi_mahasiswa (RIWAYAT AKADEMIK)
    sqlx::query!(
        r#"
        INSERT INTO registrasi_mahasiswa (mahasiswa_id, prodi_id, nim, angkatan, periode_masuk) 
        VALUES ($1, $2, $3, $4, $5)
        "#,
        new_mahasiswa_id,
        payload.prodi_id,
        payload.nim,
        payload.angkatan,
        payload.periode_masuk
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let new_mahasiswa = get_mahasiswa_by_id_repo(pool, new_mahasiswa_id).await?;
    Ok(new_mahasiswa)
}

// Query untuk mengambil detail mahasiswa menggunakan JOIN 4 tabel
pub async fn get_mahasiswa_by_id_repo(
    pool: &DbPool,
    id: Uuid,
) -> Result<MahasiswaDetail, AppError> {
    let mhs = sqlx::query_as!(
        MahasiswaDetail,
        r#"
        SELECT 
            m.id, rm.id as registrasi_id, m.nik, m.nama_mahasiswa, m.email,
            rm.nim, rm.angkatan, rm.prodi_id, rm.status_mahasiswa,
            COALESCE(p.nama_prodi, 'Prodi Tidak Ada') as "nama_prodi!",
            m.user_id, COALESCE(u.username, 'Akun Tidak Terhubung') as "username!"
        FROM mahasiswa m
        LEFT JOIN registrasi_mahasiswa rm ON rm.mahasiswa_id = m.id
        LEFT JOIN prodi p ON rm.prodi_id = p.id
        LEFT JOIN users u ON m.user_id = u.id
        WHERE m.id = $1
        ORDER BY rm.created_at DESC LIMIT 1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(mhs)
}

pub async fn get_all_mahasiswa_repo(pool: &DbPool) -> Result<Vec<MahasiswaDetail>, AppError> {
    let mahasiswa_list = sqlx::query_as!(
        MahasiswaDetail,
        r#"
        SELECT 
            m.id, rm.id as registrasi_id, m.nik, m.nama_mahasiswa, m.email,
            rm.nim, rm.angkatan, rm.prodi_id, rm.status_mahasiswa,
            COALESCE(p.nama_prodi, 'Prodi Tidak Ada') as "nama_prodi!",
            m.user_id, COALESCE(u.username, 'Akun Tidak Terhubung') as "username!"
        FROM mahasiswa m
        LEFT JOIN registrasi_mahasiswa rm ON rm.mahasiswa_id = m.id
        LEFT JOIN prodi p ON rm.prodi_id = p.id
        LEFT JOIN users u ON m.user_id = u.id
        ORDER BY m.nama_mahasiswa ASC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(mahasiswa_list)
}

pub async fn import_mahasiswa_from_csv_repo(
    pool: &DbPool,
    file_data: Bytes,
) -> Result<ImportResult, AppError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(file_data.as_ref());

    let mut tx = pool.begin().await?;

    let mut rows_processed = 0;
    let mut first_fatal_error: Option<String> = None;

    for (index, result) in reader.deserialize::<MahasiswaCsvRecord>().enumerate() {
        rows_processed += 1;
        let baris_ke = index + 2;

        match result {
            Ok(record) => {
                let prodi = sqlx::query!(
                    "SELECT id FROM prodi WHERE kode_prodi = $1",
                    record.kode_prodi
                )
                .fetch_optional(&mut *tx)
                .await?;

                if prodi.is_none() {
                    first_fatal_error = Some(format!(
                        "Baris {}: Kode prodi '{}' tidak ditemukan.",
                        baris_ke, record.kode_prodi
                    ));
                    break;
                }

                let prodi_id = prodi.unwrap().id;
                let hashed_password = bcrypt::hash(&record.nim, bcrypt::DEFAULT_COST)?;

                // Query raksasa untuk Insert ke 3 tabel sekaligus (Users, Mahasiswa, Registrasi_Mahasiswa)
                let insert_result = sqlx::query!(
                    r#"
                    WITH new_user AS (
                        INSERT INTO users (username, password_hash, full_name, email)
                        VALUES ($1, $2, $3, $4) RETURNING id
                    ), new_user_role AS (
                        INSERT INTO user_roles (user_id, role_id)
                        VALUES ((SELECT id FROM new_user), (SELECT id FROM roles WHERE name = 'MAHASISWA'))
                    ), new_mhs AS (
                        INSERT INTO mahasiswa (nik, nama_mahasiswa, email, user_id)
                        VALUES ($5, $3, $4, (SELECT id FROM new_user)) RETURNING id
                    )
                    INSERT INTO registrasi_mahasiswa (mahasiswa_id, prodi_id, nim, angkatan)
                    VALUES ((SELECT id FROM new_mhs), $6, $1, $7)
                    "#,
                    record.nim, hashed_password, record.nama_mahasiswa, record.email, record.nik, prodi_id, record.angkatan
                ).execute(&mut *tx).await;

                if let Err(e) = insert_result {
                    let e_str = e.to_string();
                    let err_msg = if e_str.contains("users_username_key")
                        || e_str.contains("registrasi_mahasiswa_nim_key")
                    {
                        format!("Baris {}: NIM '{}' sudah terdaftar.", baris_ke, record.nim)
                    } else if e_str.contains("mahasiswa_nik_key") {
                        format!("Baris {}: NIK '{}' sudah terdaftar.", baris_ke, record.nik)
                    } else {
                        format!("Baris {}: Gagal memasukkan ke DB - {}", baris_ke, e_str)
                    };
                    first_fatal_error = Some(err_msg);
                    break;
                }
            }
            Err(e) => {
                first_fatal_error = Some(format!(
                    "Baris {}: Format CSV tidak valid - {}",
                    baris_ke, e
                ));
                break;
            }
        }
    }

    let final_report: ImportResult;
    if let Some(err_msg) = first_fatal_error {
        tx.rollback().await?;
        final_report = ImportResult {
            status: "GAGAL_DIBATALKAN".to_string(),
            total_baris_dipindai: rows_processed,
            baris_berhasil_disimpan: 0,
            detail_error: vec![err_msg],
        };
    } else {
        tx.commit().await?;
        final_report = ImportResult {
            status: "SUKSES".to_string(),
            total_baris_dipindai: rows_processed,
            baris_berhasil_disimpan: rows_processed,
            detail_error: vec![],
        };
    }

    Ok(final_report)
}

pub async fn update_mahasiswa_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateMahasiswaPayload,
) -> Result<MahasiswaDetail, AppError> {
    let mut tx = pool.begin().await?;

    // 1. Update Tabel Mahasiswa (Biodata)
    sqlx::query!(
        r#"
        UPDATE mahasiswa SET 
            nama_mahasiswa = COALESCE($1, nama_mahasiswa),
            email = COALESCE($2, email),
            nik = COALESCE($3, nik),
            tempat_lahir = COALESCE($4, tempat_lahir),
            tanggal_lahir = COALESCE($5, tanggal_lahir),
            nama_ibu_kandung = COALESCE($6, nama_ibu_kandung),
            updated_at = now() 
        WHERE id = $7
        "#,
        payload.nama_mahasiswa,
        payload.email,
        payload.nik,
        payload.tempat_lahir,
        payload.tanggal_lahir,
        payload.nama_ibu_kandung,
        id
    )
    .execute(&mut *tx)
    .await?;

    // 2. Update Tabel Registrasi Mahasiswa (Akademik)
    sqlx::query!(
        r#"
        UPDATE registrasi_mahasiswa SET 
            angkatan = COALESCE($1, angkatan),
            prodi_id = COALESCE($2, prodi_id),
            nim = COALESCE($3, nim),
            status_mahasiswa = COALESCE($4, status_mahasiswa),
            updated_at = now()
        WHERE mahasiswa_id = $5
        "#,
        payload.angkatan,
        payload.prodi_id,
        payload.nim,
        payload.status_mahasiswa,
        id
    )
    .execute(&mut *tx)
    .await?;

    // 3. Update Tabel Users (Sinkronisasi profil akun)
    sqlx::query!(
        r#"
        UPDATE users SET 
            full_name = COALESCE($1, full_name), 
            email = COALESCE($2, email),
            username = COALESCE($3, username),
            updated_at = now() 
        WHERE id = (SELECT user_id FROM mahasiswa WHERE id = $4)
        "#,
        payload.nama_mahasiswa,
        payload.email,
        payload.nim,
        id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let updated_mahasiswa = get_mahasiswa_by_id_repo(pool, id).await?;
    Ok(updated_mahasiswa)
}

pub async fn delete_mahasiswa_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    let user_id_result = sqlx::query!("SELECT user_id FROM mahasiswa WHERE id = $1", id)
        .fetch_optional(&mut *tx)
        .await?;

    let user_id = match user_id_result {
        Some(record) => record.user_id,
        None => return Err(sqlx::Error::RowNotFound.into()),
    };

    // Karena di SQL Migration kita sudah pasang ON DELETE CASCADE untuk registrasi_mahasiswa
    // maka kita hanya perlu DELETE dari tabel utama mahasiswa saja.
    sqlx::query!("DELETE FROM mahasiswa WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    if let Some(user_id) = user_id {
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}
