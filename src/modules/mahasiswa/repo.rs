// src/repositories/mahasiswa_repo.rs
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::mahasiswa_model::{CreateMahasiswaPayload, MahasiswaDetail,MahasiswaCsvRecord, ImportResult, UpdateMahasiswaPayload};
use uuid::Uuid;
use bytes::Bytes;

// Fungsi utama kita, perhatikan bagaimana kita menggunakan transaksi 'tx'
pub async fn create_mahasiswa_repo(
    pool: &DbPool,
    payload: CreateMahasiswaPayload,
) -> Result<MahasiswaDetail, AppError> {
    let mut tx = pool.begin().await?;

    let hashed_password = bcrypt::hash(payload.password, bcrypt::DEFAULT_COST)?;

    // Langkah A: Coba buat user dan tangkap hasilnya
    let user_insert_result = sqlx::query!(
        "INSERT INTO users (username, password_hash, full_name, email) VALUES ($1, $2, $3, $4) RETURNING id",
        payload.nim,
        hashed_password,
        payload.nama_mahasiswa,
        payload.email
    )
    .fetch_one(&mut *tx)
    .await;

    // Langkah B: Periksa hasil pembuatan user
    let new_user_id = match user_insert_result {
        Ok(record) => record.id, // Jika sukses, ambil ID-nya
        Err(e) => {
            // Jika gagal, periksa apakah ini error duplikasi
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    let constraint = db_err.constraint().unwrap_or_default();
                    if constraint.contains("users_username_key") {
                        // Jika ya, kembalikan AppError spesifik kita, BUKAN error generik
                        return Err(AppError::DuplicateEntry(format!("NIM '{}' sudah terdaftar.", payload.nim)));
                    } else if constraint.contains("users_email_key") {
                        return Err(AppError::DuplicateEntry(format!("Email '{}' sudah terdaftar.", payload.email)));
                    }
                }
            }
            // Untuk error lain, teruskan saja
            return Err(e.into());
        }
    };

    // Langkah C: Jika user berhasil dibuat, lanjutkan proses
    sqlx::query!(
        "INSERT INTO user_roles (user_id, role_id) VALUES ($1, (SELECT id FROM roles WHERE name = 'MAHASISWA'))",
        new_user_id
    )
    .execute(&mut *tx)
    .await?;

    let new_mahasiswa_id = sqlx::query_scalar!(
        "INSERT INTO mahasiswa (nim, nama_mahasiswa, angkatan, email, prodi_id, user_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        payload.nim,
        payload.nama_mahasiswa,
        payload.angkatan,
        payload.email,
        payload.prodi_id,
        new_user_id
    )
    .fetch_one(&mut *tx)
    .await?;
    
    tx.commit().await?;

    let new_mahasiswa = get_mahasiswa_by_id_repo(pool, new_mahasiswa_id).await?;
    Ok(new_mahasiswa)
}


// Query untuk mengambil detail mahasiswa menggunakan JOIN 3 tabel
pub async fn get_mahasiswa_by_id_repo(pool: &DbPool, id: Uuid) -> Result<MahasiswaDetail, AppError> {
    let mhs = sqlx::query_as!(
        MahasiswaDetail,
        r#"
        SELECT 
            m.id, m.nim, m.nama_mahasiswa, m.angkatan, m.email,
            m.prodi_id, p.nama_prodi,
            m.user_id, u.username
        FROM mahasiswa m
        LEFT JOIN prodi p ON m.prodi_id = p.id
        LEFT JOIN users u ON m.user_id = u.id
        WHERE m.id = $1
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
            m.id, m.nim, m.nama_mahasiswa, m.angkatan, m.email,
            m.prodi_id,
            -- Jika nama prodi NULL, gunakan string default. Tanda '!' memberitahu sqlx hasilnya tidak akan NULL.
            COALESCE(p.nama_prodi, 'Prodi Tidak Ada') as "nama_prodi!",
            m.user_id,
            -- Jika username NULL (karena user_id NULL), gunakan string default.
            COALESCE(u.username, 'Akun Tidak Terhubung') as "username!"
        FROM mahasiswa m
        LEFT JOIN prodi p ON m.prodi_id = p.id
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

    // enumerate() akan menghitung jumlah baris yang kita proses
    for (index, result) in reader.deserialize::<MahasiswaCsvRecord>().enumerate() {
        rows_processed += 1; // Hitung setiap baris yang coba kita proses
        let baris_ke = index + 2; // +2 untuk header dan index 0

        match result {
            Ok(record) => {
                let prodi = sqlx::query!("SELECT id FROM prodi WHERE kode_prodi = $1", record.kode_prodi)
                    .fetch_optional(&mut *tx)
                    .await?;

                if prodi.is_none() {
                    first_fatal_error = Some(format!("Baris {}: Kode prodi '{}' tidak ditemukan.", baris_ke, record.kode_prodi));
                    break;
                }

                let prodi_id = prodi.unwrap().id;
                let hashed_password = bcrypt::hash(&record.nim, bcrypt::DEFAULT_COST)?;
                
                let insert_result = sqlx::query!(
                    r#"
                    WITH new_user AS (
                        INSERT INTO users (username, password_hash, full_name, email)
                        VALUES ($1, $2, $3, $4) RETURNING id
                    ), new_user_role AS (
                        INSERT INTO user_roles (user_id, role_id)
                        VALUES ((SELECT id FROM new_user), (SELECT id FROM roles WHERE name = 'MAHASISWA'))
                    )
                    INSERT INTO mahasiswa (nim, nama_mahasiswa, angkatan, email, prodi_id, user_id)
                    VALUES ($1, $3, $5, $4, $6, (SELECT id FROM new_user))
                    "#,
                    record.nim, hashed_password, record.nama_mahasiswa, record.email, record.angkatan, prodi_id,
                ).execute(&mut *tx).await;

                if let Err(e) = insert_result {
                    let e_str = e.to_string();
                    let err_msg = if e_str.contains("users_username_key") || e_str.contains("mahasiswa_nim_key") {
                        format!("Baris {}: NIM '{}' sudah terdaftar.", baris_ke, record.nim)
                    } else if e_str.contains("users_email_key") || e_str.contains("mahasiswa_email_key") {
                        format!("Baris {}: Email '{}' sudah terdaftar.", baris_ke, record.email)
                    } else {
                        format!("Baris {}: Gagal memasukkan ke DB - {}", baris_ke, e_str)
                    };
                    first_fatal_error = Some(err_msg);
                    break;
                }
            }
            Err(e) => {
                first_fatal_error = Some(format!("Baris {}: Format CSV tidak valid - {}", baris_ke, e));
                break;
            }
        }
    }

    let final_report: ImportResult;
    if let Some(err_msg) = first_fatal_error {
        tx.rollback().await?; // Batalkan transaksi
        final_report = ImportResult {
            status: "GAGAL_DIBATALKAN".to_string(),
            total_baris_dipindai: rows_processed,
            baris_berhasil_disimpan: 0, // Karena di-rollback, tidak ada yang disimpan
            detail_error: vec![err_msg],
        };
    } else {
        tx.commit().await?; // Simpan transaksi
        final_report = ImportResult {
            status: "SUKSES".to_string(),
            total_baris_dipindai: rows_processed,
            baris_berhasil_disimpan: rows_processed, // Semua yang dipindai berhasil disimpan
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
    // Mulai transaksi untuk memastikan konsistensi
    let mut tx = pool.begin().await?;

    // 1. Lakukan update untuk data yang "normal" (selain NIM)
    sqlx::query!(
        "UPDATE mahasiswa SET nama_mahasiswa = $1, angkatan = $2, email = $3, prodi_id = $4, updated_at = now() WHERE id = $5",
        payload.nama_mahasiswa,
        payload.angkatan,
        payload.email,
        payload.prodi_id,
        id
    )
    .execute(&mut *tx)
    .await?;
    
    // Juga update tabel users yang terhubung (kecuali username/NIM)
    sqlx::query!(
        "UPDATE users SET full_name = $1, email = $2, updated_at = now() WHERE id = (SELECT user_id FROM mahasiswa WHERE id = $3)",
        payload.nama_mahasiswa,
        payload.email,
        id
    ).execute(&mut *tx).await?;

    // 2. Lakukan update untuk NIM secara kondisional
    if let Some(new_nim) = payload.nim {
        // Update NIM di tabel mahasiswa
        sqlx::query!("UPDATE mahasiswa SET nim = $1 WHERE id = $2", new_nim, id)
            .execute(&mut *tx)
            .await?;

        // Update username di tabel users
        sqlx::query!(
            "UPDATE users SET username = $1 WHERE id = (SELECT user_id FROM mahasiswa WHERE id = $2)",
            new_nim,
            id
        ).execute(&mut *tx).await?;
    }

    // 3. Commit semua perubahan
    tx.commit().await?;

    // Ambil dan kembalikan data terbaru
    let updated_mahasiswa = get_mahasiswa_by_id_repo(pool, id).await?;
    Ok(updated_mahasiswa)
}

pub async fn delete_mahasiswa_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    // Mulai transaksi
    let mut tx = pool.begin().await?;

    // 1. Ambil user_id dari mahasiswa yang akan dihapus
    let user_id_result = sqlx::query!("SELECT user_id FROM mahasiswa WHERE id = $1", id)
        .fetch_optional(&mut *tx)
        .await?;

    // Jika mahasiswa tidak ditemukan, kembalikan error
    let user_id = match user_id_result {
        Some(record) => record.user_id,
        None => return Err(sqlx::Error::RowNotFound.into()),
    };

    // 2. Hapus data dari tabel mahasiswa
    sqlx::query!("DELETE FROM mahasiswa WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    // 3. Jika ada user_id yang terhubung, hapus juga user tersebut
    if let Some(user_id) = user_id {
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&mut *tx)
            .await?;
    }
    
    // 4. Commit transaksi
    tx.commit().await?;

    Ok(())
}