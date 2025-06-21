// src/repositories/mahasiswa_repo.rs
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::mahasiswa_model::{CreateMahasiswaPayload, MahasiswaDetail};
use crate::models::mahasiswa_model::{MahasiswaCsvRecord, ImportResult};
use uuid::Uuid;
use bytes::Bytes;

// Fungsi utama kita, perhatikan bagaimana kita menggunakan transaksi 'tx'
pub async fn create_mahasiswa_repo(
    pool: &DbPool,
    payload: CreateMahasiswaPayload,
) -> Result<MahasiswaDetail, AppError> {
    // 1. Mulai transaksi
    let mut tx = pool.begin().await?;

    // 2. Buat entri di tabel `users`
    let hashed_password = bcrypt::hash(payload.password, bcrypt::DEFAULT_COST)?;
    let new_user_id = sqlx::query_scalar!(
        "INSERT INTO users (username, password_hash, full_name, email) VALUES ($1, $2, $3, $4) RETURNING id",
        payload.nim, // Gunakan NIM sebagai username
        hashed_password,
        payload.nama_mahasiswa,
        payload.email
    )
    .fetch_one(&mut *tx) // Jalankan di dalam transaksi
    .await?;

    // 3. Berikan peran 'MAHASISWA' ke user baru di tabel `user_roles`
    sqlx::query!(
        "INSERT INTO user_roles (user_id, role_id) VALUES ($1, (SELECT id FROM roles WHERE name = 'MAHASISWA'))",
        new_user_id
    )
    .execute(&mut *tx) // Jalankan di dalam transaksi
    .await?;

    // 4. Buat entri di tabel `mahasiswa` dan hubungkan dengan user_id yang baru
    let new_mahasiswa_id = sqlx::query_scalar!(
        "INSERT INTO mahasiswa (nim, nama_mahasiswa, angkatan, email, prodi_id, user_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        payload.nim,
        payload.nama_mahasiswa,
        payload.angkatan,
        payload.email,
        payload.prodi_id,
        new_user_id
    )
    .fetch_one(&mut *tx) // Jalankan di dalam transaksi
    .await?;

    // 5. Commit transaksi. Jika ini berhasil, semua perubahan akan disimpan.
    // Jika ada error di langkah manapun sebelumnya, `tx` akan di-drop dan perubahan di-rollback.
    tx.commit().await?;

    // Ambil dan kembalikan detail mahasiswa yang baru dibuat
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

// (Anda bisa menambahkan fungsi get_all, update, dan delete dengan pola yang sama)
pub async fn get_all_mahasiswa_repo(pool: &DbPool) -> Result<Vec<MahasiswaDetail>, AppError> {
    let mahasiswa_list = sqlx::query_as!(
        MahasiswaDetail,
        r#"
        SELECT 
            m.id, m.nim, m.nama_mahasiswa, m.angkatan, m.email,
            m.prodi_id, p.nama_prodi,
            m.user_id, u.username
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

    // --- Menggunakan nama variabel baru ---
    let mut data_valid = 0;
    let mut data_tidak_valid = 0;
    let mut rincian_tidak_valid = Vec::new();

    for (index, result) in reader.deserialize::<MahasiswaCsvRecord>().enumerate() {
        let baris_ke = index + 2;

        match result {
            Ok(record) => {
                let prodi = sqlx::query!("SELECT id FROM prodi WHERE kode_prodi = $1", record.kode_prodi)
                    .fetch_optional(&mut *tx)
                    .await?;

                if let Some(prodi) = prodi {
                    let hashed_password = bcrypt::hash(&record.nim, bcrypt::DEFAULT_COST)?;
                    
                    let insert_result = sqlx::query!(
                        r#"
                        WITH new_user AS (
                            INSERT INTO users (username, password_hash, full_name, email)
                            VALUES ($1, $2, $3, $4)
                            RETURNING id
                        ), new_user_role AS (
                            INSERT INTO user_roles (user_id, role_id)
                            VALUES ((SELECT id FROM new_user), (SELECT id FROM roles WHERE name = 'MAHASISWA'))
                        )
                        INSERT INTO mahasiswa (nim, nama_mahasiswa, angkatan, email, prodi_id, user_id)
                        VALUES ($1, $3, $5, $4, $6, (SELECT id FROM new_user))
                        "#,
                        record.nim,
                        hashed_password,
                        record.nama_mahasiswa,
                        record.email,
                        record.angkatan,
                        prodi.id,
                    ).execute(&mut *tx).await;

                    match insert_result {
                        Ok(_) => data_valid += 1, // <-- Diubah
                        Err(e) => {
                            data_tidak_valid += 1; // <-- Diubah
                            rincian_tidak_valid.push(format!("Baris {}: Gagal memasukkan ke DB - {}", baris_ke, e)); // <-- Diubah
                        }
                    }
                } else {
                    data_tidak_valid += 1; // <-- Diubah
                    rincian_tidak_valid.push(format!("Baris {}: Kode prodi '{}' tidak ditemukan.", baris_ke, record.kode_prodi)); // <-- Diubah
                }
            }
            Err(e) => {
                data_tidak_valid += 1; // <-- Diubah
                rincian_tidak_valid.push(format!("Baris {}: Format CSV tidak valid - {}", baris_ke, e)); // <-- Diubah
            }
        }
    }

    let final_report: ImportResult;

    if data_tidak_valid > 0 { // <-- Diubah
        tx.rollback().await?;
        final_report = ImportResult {
            data_valid: 0, // <-- Diubah
            data_tidak_valid, // <-- Diubah
            rincian_tidak_valid: [
                // --- Menggunakan pesan error baru ---
                "Terjadi Data tidak valid, tidak ada data yang diimpor. Perbaiki error di bawah dan coba lagi.".to_string(),
            ].into_iter().chain(rincian_tidak_valid.into_iter()).collect(), // <-- Diubah
        };
    } else {
        tx.commit().await?;
        final_report = ImportResult {
            data_valid, // <-- Diubah
            data_tidak_valid, // <-- Diubah
            rincian_tidak_valid, // <-- Diubah
        };
    }

    Ok(final_report)
}