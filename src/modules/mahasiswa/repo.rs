// src/modules/mahasiswa/repo.rs
use super::model::{
    CreateMahasiswaPayload, ImportResult, MahasiswaCsvRecord, MahasiswaDetail,
    MahasiswaRombelDetail, MahasiswaRombelFilter, PindahRombelPayload, RenameRombelPayload,
    RombelFilter, RombelSummary, UpdateMahasiswaPayload,
};
use crate::db::DbPool;
use crate::errors::AppError;
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

pub async fn get_mahasiswa_by_id_repo(
    pool: &DbPool,
    id: Uuid,
) -> Result<MahasiswaDetail, AppError> {
    let mhs = sqlx::query_as!(
        MahasiswaDetail,
        r#"
        SELECT 
            m.id, 
            rm.id as "registrasi_id?", 
            m.nik, 
            m.nama_mahasiswa, 
            m.email,
            rm.nim as "nim?", 
            rm.angkatan as "angkatan?", 
            rm.prodi_id as "prodi_id?", 
            rm.status_mahasiswa as "status_mahasiswa?",
            COALESCE(p.nama_prodi, 'Prodi Tidak Ada') as "nama_prodi!",
            
            -- AMBIL DATA DOSEN PA --
            rm.dosen_pa_id as "dosen_pa_id?",
            peg.nama_lengkap as "nama_dosen_pa?",
            
            m.user_id, 
            COALESCE(u.username, 'Akun Tidak Terhubung') as "username!"
        FROM mahasiswa m
        LEFT JOIN registrasi_mahasiswa rm ON rm.mahasiswa_id = m.id
        LEFT JOIN prodi p ON rm.prodi_id = p.id
        LEFT JOIN users u ON m.user_id = u.id
        -- TAMBAHAN JOIN DOSEN PA --
        LEFT JOIN dosen d ON rm.dosen_pa_id = d.id
        LEFT JOIN pegawai peg ON d.pegawai_id = peg.id
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
            m.id, 
            rm.id as "registrasi_id?", 
            m.nik, 
            m.nama_mahasiswa, 
            m.email,
            rm.nim as "nim?", 
            rm.angkatan as "angkatan?", 
            rm.prodi_id as "prodi_id?", 
            rm.status_mahasiswa as "status_mahasiswa?",
            COALESCE(p.nama_prodi, 'Prodi Tidak Ada') as "nama_prodi!",
            
            -- AMBIL DATA DOSEN PA --
            rm.dosen_pa_id as "dosen_pa_id?",
            peg.nama_lengkap as "nama_dosen_pa?",
            
            m.user_id, 
            COALESCE(u.username, 'Akun Tidak Terhubung') as "username!"
        FROM mahasiswa m
        LEFT JOIN registrasi_mahasiswa rm ON rm.mahasiswa_id = m.id
        LEFT JOIN prodi p ON rm.prodi_id = p.id
        LEFT JOIN users u ON m.user_id = u.id
        -- TAMBAHAN JOIN DOSEN PA --
        LEFT JOIN dosen d ON rm.dosen_pa_id = d.id
        LEFT JOIN pegawai peg ON d.pegawai_id = peg.id
        ORDER BY m.nama_mahasiswa ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(mahasiswa_list)
}

pub async fn import_mahasiswa_from_csv_repo(
    pool: &DbPool,
    file_data: bytes::Bytes,
) -> Result<ImportResult, AppError> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(file_data.as_ref());

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    // =========================================================================
    // FASE 1: BACA & PARSING CSV KE MEMORI
    // =========================================================================
    let mut valid_rows = Vec::new();
    for (index, result) in rdr.deserialize::<MahasiswaCsvRecord>().enumerate() {
        let row_number = index + 2; // +2 karena baris 1 adalah header
        match result {
            Ok(row) => valid_rows.push((row_number, row)),
            Err(e) => {
                failed_count += 1;
                errors.push(format!(
                    "Baris {}: Format baris CSV tidak valid - {}",
                    row_number, e
                ));
            }
        }
    }

    // =========================================================================
    // FASE 2: PRE-HASH PASSWORD DI LUAR TRANSAKSI DB (MENCEGAH SERVER FREEZE)
    // =========================================================================
    // Kita gunakan `spawn_blocking` agar perhitungan kriptografi tidak memblokir async runtime
    let hashed_data = tokio::task::spawn_blocking(move || {
        let mut processed = Vec::new();
        for (row_number, row) in valid_rows {
            let nim_clean = row.nim.trim().to_string();
            // Gunakan cost 8 (Bukan default 12). Cost 8 sangat cepat (±10ms per hash)
            // sehingga cocok untuk import massal password default.
            let hash_result = bcrypt::hash(&nim_clean, 8);
            processed.push((row_number, row, hash_result));
        }
        processed
    })
    .await
    .map_err(|e| AppError::AnyhowError(anyhow::anyhow!("Worker thread error: {}", e)))?;

    // =========================================================================
    // FASE 3: BUKA TRANSAKSI DATABASE & EKSEKUSI (SANGAT CEPAT)
    // =========================================================================
    let mut tx = pool.begin().await?;

    // Ambil role_id untuk MAHASISWA sekali saja di luar loop
    let role_id = sqlx::query_scalar!("SELECT id FROM roles WHERE name = 'MAHASISWA'")
        .fetch_optional(&mut *tx)
        .await?;

    for (row_number, row, hash_result) in hashed_data {
        // Buat SAVEPOINT agar jika ada NIK duplikat, tidak membatalkan seluruh proses
        let sp_name = format!("sp_{}", row_number);
        sqlx::query(&format!("SAVEPOINT {}", sp_name))
            .execute(&mut *tx)
            .await?;

        // 1. Cek hasil pre-hash
        let hashed_password = match hash_result {
            Ok(h) => h,
            Err(_) => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                    .execute(&mut *tx)
                    .await?;
                failed_count += 1;
                errors.push(format!(
                    "Baris {}: Gagal mengenkripsi password.",
                    row_number
                ));
                continue;
            }
        };

        // 2. Cari Prodi ID & Nama Prodi berdasarkan Kode Prodi
        let prodi = match sqlx::query!(
            "SELECT id, nama_prodi FROM prodi WHERE kode_prodi = $1",
            row.kode_prodi.trim()
        )
        .fetch_optional(&mut *tx)
        .await?
        {
            Some(p) => p,
            None => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                    .execute(&mut *tx)
                    .await?;
                failed_count += 1;
                errors.push(format!(
                    "Baris {}: Prodi dengan kode '{}' tidak ditemukan.",
                    row_number, row.kode_prodi
                ));
                continue;
            }
        };

        // 3. LOGIKA OPSI 2 (Pembuatan NIK Sementara)
        let nik_final = if row.nik.trim().is_empty() {
            format!("TMP{}", row.nim.trim())
        } else {
            row.nik.trim().to_string()
        };

        // 4. LOGIKA ROMBEL OTOMATIS (nama_jurusan + tahun_masuk)
        let rombel_otomatis = format!(
            "{} {}",
            prodi.nama_prodi.trim().to_lowercase(),
            row.angkatan
        );

        // 5. Buat Akun Login (Users)
        let user_result = sqlx::query_scalar!(
            "INSERT INTO users (username, password_hash, full_name, email) VALUES ($1, $2, $3, $4) RETURNING id",
            row.nim.trim(),
            hashed_password,
            row.nama_mahasiswa.trim(),
            row.email.trim()
        )
        .fetch_one(&mut *tx)
        .await;

        let user_id = match user_result {
            Ok(id) => id,
            Err(e) => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                    .execute(&mut *tx)
                    .await?;
                failed_count += 1;

                let err_msg = e.to_string();
                let alasan = if err_msg.contains("users_email_key") {
                    "Email duplikat"
                } else {
                    "NIM duplikat"
                };
                errors.push(format!(
                    "Baris {}: Gagal simpan User ({}).",
                    row_number, alasan
                ));
                continue;
            }
        };

        // Berikan Role Mahasiswa
        if let Some(r_id) = role_id {
            let _ = sqlx::query!(
                "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                user_id,
                r_id
            )
            .execute(&mut *tx)
            .await;
        }

        // 6. Insert Biodata Mahasiswa (Menggunakan nik_final)
        let mhs_result = sqlx::query_scalar!(
            "INSERT INTO mahasiswa (user_id, nik, nama_mahasiswa) VALUES ($1, $2, $3) RETURNING id",
            user_id,
            nik_final,
            row.nama_mahasiswa.trim()
        )
        .fetch_one(&mut *tx)
        .await;

        let mhs_id = match mhs_result {
            Ok(id) => id,
            Err(_) => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                    .execute(&mut *tx)
                    .await?;
                failed_count += 1;
                errors.push(format!(
                    "Baris {}: Gagal simpan Biodata (NIK {} mungkin duplikat).",
                    row_number, nik_final
                ));
                continue;
            }
        };

        // 7. Insert Registrasi Akademik (Dengan kode_rombel otomatis)
        let reg_result = sqlx::query!(
            "INSERT INTO registrasi_mahasiswa (mahasiswa_id, prodi_id, nim, angkatan, kode_rombel) VALUES ($1, $2, $3, $4, $5)",
            mhs_id,
            prodi.id,
            row.nim.trim(),
            row.angkatan,
            rombel_otomatis
        )
        .execute(&mut *tx)
        .await;

        match reg_result {
            Ok(_) => {
                // Lepaskan Savepoint jika baris ini sukses sepenuhnya
                sqlx::query(&format!("RELEASE SAVEPOINT {}", sp_name))
                    .execute(&mut *tx)
                    .await?;
                success_count += 1;
            }
            Err(_) => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                    .execute(&mut *tx)
                    .await?;
                failed_count += 1;
                errors.push(format!(
                    "Baris {}: Gagal simpan Akademik (NIM {} duplikat di tabel registrasi).",
                    row_number,
                    row.nim.trim()
                ));
            }
        }
    }

    // =========================================================================
    // FASE 4: COMMIT ATAU ROLLBACK TOTAL (All-or-Nothing)
    // =========================================================================
    if failed_count > 0 {
        tx.rollback().await?;
        return Ok(ImportResult {
            status: "GAGAL".to_string(),
            total_baris_dipindai: success_count + failed_count,
            baris_berhasil_disimpan: 0, // 0 karena transaksi dibatalkan
            detail_error: errors,
        });
    }

    tx.commit().await?;

    Ok(ImportResult {
        status: "SUKSES".to_string(),
        total_baris_dipindai: success_count,
        baris_berhasil_disimpan: success_count,
        detail_error: errors,
    })
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

// =========================================================================
// --- REPOSITORY UNTUK FITUR MANAJEMEN ROMBEL ---
// =========================================================================
pub async fn get_rombel_summary_repo(
    pool: &DbPool,
    filter: RombelFilter,
) -> Result<Vec<RombelSummary>, AppError> {
    let mut query = sqlx::QueryBuilder::new(
        r#"
        SELECT 
            rm.prodi_id,
            p.nama_prodi,
            rm.angkatan,
            rm.kode_rombel,
            COUNT(rm.id) as jumlah_mahasiswa,
            rm.dosen_pa_id,
            peg.nama_lengkap as nama_dosen_pa
        FROM registrasi_mahasiswa rm
        JOIN prodi p ON rm.prodi_id = p.id
        LEFT JOIN dosen d ON rm.dosen_pa_id = d.id
        LEFT JOIN pegawai peg ON d.pegawai_id = peg.id
        WHERE 1=1
        "#,
    );

    if let Some(prodi_id) = filter.prodi_id {
        query.push(" AND rm.prodi_id = ");
        query.push_bind(prodi_id);
    }
    if let Some(angkatan) = filter.angkatan {
        query.push(" AND rm.angkatan = ");
        query.push_bind(angkatan);
    }

    // Ingat: Karena kita menambahkan dosen_pa_id dan nama_dosen_pa ke SELECT,
    // kita juga harus menambahkannya ke dalam GROUP BY.
    query.push(" GROUP BY rm.prodi_id, p.nama_prodi, rm.angkatan, rm.kode_rombel, rm.dosen_pa_id, peg.nama_lengkap ORDER BY rm.angkatan DESC, p.nama_prodi ASC, rm.kode_rombel ASC");

    let result = query
        .build_query_as::<RombelSummary>()
        .fetch_all(pool)
        .await?;
    Ok(result)
}

pub async fn get_mahasiswa_by_rombel_repo(
    pool: &DbPool,
    filter: MahasiswaRombelFilter,
) -> Result<Vec<MahasiswaRombelDetail>, AppError> {
    let mut query = sqlx::QueryBuilder::new(
        r#"
        SELECT 
            rm.id as registrasi_id,
            rm.mahasiswa_id,
            rm.nim,
            m.nama_mahasiswa,
            rm.kode_rombel
        FROM registrasi_mahasiswa rm
        JOIN mahasiswa m ON rm.mahasiswa_id = m.id
        WHERE rm.prodi_id = "#,
    );
    query.push_bind(filter.prodi_id);
    query.push(" AND rm.angkatan = ");
    query.push_bind(filter.angkatan);

    // Mengakomodasi jika admin ingin mencari mahasiswa yang "Belum punya rombel" (NULL)
    if let Some(rombel) = filter.kode_rombel {
        query.push(" AND rm.kode_rombel = ");
        query.push_bind(rombel);
    } else {
        query.push(" AND rm.kode_rombel IS NULL");
    }

    query.push(" ORDER BY rm.nim ASC");

    let result = query
        .build_query_as::<MahasiswaRombelDetail>()
        .fetch_all(pool)
        .await?;
    Ok(result)
}

pub async fn pindah_rombel_repo(
    pool: &DbPool,
    payload: PindahRombelPayload,
) -> Result<u64, AppError> {
    if payload.registrasi_ids.is_empty() {
        return Ok(0);
    }

    let mut tx = pool.begin().await?;
    let mut rows_affected = 0;

    // Loop sangat cepat menggunakan Transaksi
    for reg_id in payload.registrasi_ids {
        let res = sqlx::query!(
            "UPDATE registrasi_mahasiswa SET kode_rombel = $1, updated_at = now() WHERE id = $2",
            payload.kode_rombel_baru,
            reg_id
        )
        .execute(&mut *tx)
        .await?;
        rows_affected += res.rows_affected();
    }

    tx.commit().await?;
    Ok(rows_affected)
}

pub async fn rename_rombel_repo(
    pool: &DbPool,
    payload: RenameRombelPayload,
) -> Result<u64, AppError> {
    let mut query = sqlx::QueryBuilder::new("UPDATE registrasi_mahasiswa SET kode_rombel = ");
    query.push_bind(payload.kode_rombel_baru);
    query.push(", updated_at = now() WHERE prodi_id = ");
    query.push_bind(payload.prodi_id);
    query.push(" AND angkatan = ");
    query.push_bind(payload.angkatan);

    if let Some(lama) = payload.kode_rombel_lama {
        query.push(" AND kode_rombel = ");
        query.push_bind(lama);
    } else {
        query.push(" AND kode_rombel IS NULL");
    }

    let res = query.build().execute(pool).await?;
    Ok(res.rows_affected())
}
