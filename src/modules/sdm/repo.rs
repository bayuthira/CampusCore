// src/modules/sdm/repo.rs

use super::model::{
    Pegawai, PegawaiPayload, KategoriPegawai,CreateUserForPegawaiPayload
};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

/// Helper function untuk mendapatkan detail satu pegawai berdasarkan ID
pub async fn get_pegawai_by_id_repo(pool: &DbPool, id: Uuid) -> Result<Pegawai, AppError> {
    get_pegawai_by_id_repo_inner(pool, id).await
}

/// Membuat data pegawai baru dan akun user terkait dalam satu transaksi
pub async fn create_pegawai_repo(
    pool: &DbPool,
    payload: PegawaiPayload,
) -> Result<Pegawai, AppError> {
    let mut tx = pool.begin().await?;
    
    // Tahap 1: Tentukan User ID (Cari Berdasarkan Email, atau Buat Baru)
    let new_user_id: Option<Uuid> = if let Some(password) = payload.password {
        // Cek dulu apakah user dengan email ini sudah ada (jika email diisi)
        let existing_user = if let Some(email) = &payload.email {
            sqlx::query!("SELECT id FROM users WHERE email = $1", email)
                .fetch_optional(&mut *tx)
                .await?
        } else {
            None
        };

        if let Some(user) = existing_user {
            // Jika user dengan email tersebut sudah ada, gunakan ID-nya.
            // Abaikan password yang diinput karena akun sudah ada.
            Some(user.id)
        } else {
            // Jika user belum ada, buat baru.
            let hashed_password = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
            match sqlx::query!(
                "INSERT INTO users (username, password_hash, full_name, email) VALUES ($1, $2, $3, $4) RETURNING id",
                &payload.nik, // username tetap pakai NIK yang unik
                hashed_password,
                &payload.nama_lengkap,
                payload.email.as_deref()
            ).fetch_one(&mut *tx).await {
                Ok(rec) => Some(rec.id),
                Err(e) => {
                    // Penanganan error jika NIK duplikat (bukan email)
                    tx.rollback().await?;
                    if let Some(db_err) = e.as_database_error() {
                        if db_err.is_unique_violation() && db_err.constraint().unwrap_or_default().contains("users_username_key") {
                            return Err(AppError::DuplicateEntry(format!("NIK '{}' sudah terdaftar sebagai username.", payload.nik)));
                        }
                    }
                    return Err(e.into());
                }
            }
        }
    } else {
        None
    };

    // Konversi semua enum opsional ke string opsional
    let jenis_kelamin_str = payload.jenis_kelamin.as_ref().map(|e| e.as_str());
    let status_nikah_str = payload.status_nikah.as_ref().map(|e| e.as_str());
    let kategori_pegawai_str = payload.kategori_pegawai.as_ref().map(|e| e.as_str());
    let status_pegawai_str = payload.status_pegawai.as_ref().map(|e| e.as_str());

    // 2. Insert data ke tabel pegawai
    let new_pegawai_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO pegawai (
            user_id, nik, no_ktp, nama_lengkap, gelar_depan, gelar_belakang, tempat_lahir, tanggal_lahir, 
            jenis_kelamin, status_nikah, agama, gol_darah, alamat_domisili, kota, kode_pos, 
            nomor_hp, email, kategori_pegawai, status_pegawai, is_active, unit_kerja, bagian, 
            jabatan, tanggal_masuk, tanggal_pensiun, no_kk, no_npwp, no_bpjs_kesehatan, no_bpjs_ketenagakerjaan
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9::"JenisKelamin", $10::"StatusNikah", $11, $12, $13, $14, $15, 
            $16, $17, $18::"KategoriPegawai", $19::"StatusPegawai", $20, $21, $22, $23, $24, $25, $26, $27, $28, $29
        ) RETURNING id
        "#,
    )
    .bind(new_user_id)
    .bind(&payload.nik)
    .bind(&payload.no_ktp)
    .bind(&payload.nama_lengkap)
    .bind(&payload.gelar_depan)
    .bind(&payload.gelar_belakang)
    .bind(&payload.tempat_lahir)
    .bind(payload.tanggal_lahir)
    .bind(jenis_kelamin_str)
    .bind(status_nikah_str)
    .bind(&payload.agama)
    .bind(&payload.gol_darah)
    .bind(&payload.alamat_domisili)
    .bind(&payload.kota)
    .bind(&payload.kode_pos)
    .bind(&payload.nomor_hp)
    .bind(&payload.email)
    .bind(kategori_pegawai_str)
    .bind(status_pegawai_str)
    .bind(payload.is_active.unwrap_or(true))
    .bind(&payload.unit_kerja)
    .bind(&payload.bagian)
    .bind(&payload.jabatan)
    .bind(&payload.tanggal_masuk)
    .bind(payload.tanggal_pensiun)
    .bind(&payload.no_kk)
    .bind(&payload.no_npwp)
    .bind(&payload.no_bpjs_kesehatan)
    .bind(&payload.no_bpjs_ketenagakerjaan)
    .fetch_one(&mut *tx)
    .await?;

    // 3. Jika Tenaga Pendidik, buat atau tautkan data Dosen
    if let Some(KategoriPegawai::TenagaPendidik) = &payload.kategori_pegawai {
        let nidn = payload.nidn.as_ref().ok_or_else(|| AppError::Forbidden("NIDN wajib diisi untuk Tenaga Pendidik.".to_string()))?;
        let prodi_id = payload.prodi_id.ok_or_else(|| AppError::Forbidden("Prodi ID wajib diisi untuk Tenaga Pendidik.".to_string()))?;
        
        let existing_dosen = sqlx::query!("SELECT id FROM dosen WHERE nidn = $1", nidn)
            .fetch_optional(&mut *tx).await?;

        if let Some(dosen) = existing_dosen {
            sqlx::query!(
                "UPDATE dosen SET pegawai_id = $1, user_id = $2 WHERE id = $3",
                new_pegawai_id, new_user_id, dosen.id
            ).execute(&mut *tx).await?;
        } else {
            sqlx::query!(
                "INSERT INTO dosen (nidn, nama_dosen, email, prodi_id, user_id, pegawai_id) VALUES ($1, $2, $3, $4, $5, $6)",
                nidn, &payload.nama_lengkap, payload.email.as_deref(), prodi_id, new_user_id, new_pegawai_id
            ).execute(&mut *tx).await?;
        }
        
        if let Some(user_id) = new_user_id {
            sqlx::query!(
                "INSERT INTO user_roles (user_id, role_id) VALUES ($1, (SELECT id FROM roles WHERE name = 'DOSEN')) ON CONFLICT DO NOTHING",
                user_id
            ).execute(&mut *tx).await?;
        }
    }

    tx.commit().await?;

    // Ambil dan kembalikan detail pegawai yang baru dibuat
    let new_pegawai = get_pegawai_by_id_repo(pool, new_pegawai_id).await?;
    Ok(new_pegawai)
}




/// Mengambil semua data pegawai
pub async fn get_all_pegawai_repo(pool: &DbPool) -> Result<Vec<Pegawai>, AppError> {
    let pegawai_list = sqlx::query_as!(
        Pegawai,
        r#"
        SELECT
            p.id, p.user_id, p.nik, p.no_ktp, p.nama_lengkap, p.gelar_depan, p.gelar_belakang,
            p.tempat_lahir, p.tanggal_lahir, p.jenis_kelamin as "jenis_kelamin: _",
            p.status_nikah as "status_nikah: _", p.agama, p.gol_darah, p.alamat_domisili,
            p.kota, p.kode_pos, p.nomor_hp, p.email, p.kategori_pegawai as "kategori_pegawai: _",
            p.status_pegawai as "status_pegawai: _", p.is_active, p.unit_kerja, p.bagian,
            p.jabatan, p.tanggal_masuk, p.tanggal_pensiun, p.no_kk, p.no_npwp,
            p.no_bpjs_kesehatan, p.no_bpjs_ketenagakerjaan,
            d.nidn, d.prodi_id, prodi.nama_prodi,
            p.created_at, p.updated_at
        FROM pegawai p
        LEFT JOIN dosen d ON p.id = d.pegawai_id
        LEFT JOIN prodi ON d.prodi_id = prodi.id
        ORDER BY nama_lengkap ASC
        "#
    ).fetch_all(pool).await?;

    Ok(pegawai_list)
}

/// Memperbarui data pegawai
pub async fn update_pegawai_repo(pool: &DbPool, id: Uuid, payload: PegawaiPayload) -> Result<Pegawai, AppError> {
    let mut tx = pool.begin().await?;

    // Tahap 1: Ambil data pegawai saat ini dari database
    let old_pegawai = get_pegawai_by_id_repo_inner(&mut *tx, id).await?;

    // Tahap 2: Cek apakah ada perubahan pada kategori pegawai
    if old_pegawai.kategori_pegawai != payload.kategori_pegawai {
        
        // Skenario 1: Berubah menjadi Tenaga Pendidik
        if let Some(KategoriPegawai::TenagaPendidik) = &payload.kategori_pegawai {
            let nidn = payload.nidn.as_ref().ok_or_else(|| AppError::Forbidden("NIDN wajib diisi untuk mengubah status menjadi Tenaga Pendidik.".to_string()))?;
            let prodi_id = payload.prodi_id.ok_or_else(|| AppError::Forbidden("Prodi ID wajib diisi untuk mengubah status menjadi Tenaga Pendidik.".to_string()))?;

            let existing_dosen = sqlx::query!("SELECT id FROM dosen WHERE nidn = $1", nidn)
                .fetch_optional(&mut *tx).await?;

            if let Some(dosen) = existing_dosen {
                // Jika dosen sudah ada, tautkan ke pegawai ini
                sqlx::query!(
                    "UPDATE dosen SET pegawai_id = $1, user_id = $2 WHERE id = $3",
                    id, old_pegawai.user_id, dosen.id
                ).execute(&mut *tx).await?;
            } else {
                // Jika dosen belum ada, buat baru
                sqlx::query!(
                    "INSERT INTO dosen (nidn, nama_dosen, email, prodi_id, pegawai_id, user_id) VALUES ($1, $2, $3, $4, $5, $6)",
                    nidn,                           // -> $1 (nidn)
                    &payload.nama_lengkap,          // -> $2 (nama_dosen)
                    payload.email.as_deref(),       // -> $3 (email)
                    prodi_id,                       // -> $4 (prodi_id)
                    id,                             // -> $5 (pegawai_id) <-- BENAR
                    old_pegawai.user_id             // -> $6 (user_id)     <-- BENAR
                ).execute(&mut *tx).await?;
            }

            // Pastikan user memiliki peran DOSEN
            if let Some(user_id) = old_pegawai.user_id {
                sqlx::query!(
                    "INSERT INTO user_roles (user_id, role_id) VALUES ($1, (SELECT id FROM roles WHERE name = 'DOSEN')) ON CONFLICT DO NOTHING",
                    user_id
                ).execute(&mut *tx).await?;
            }
        }

        // Skenario 2: Berubah menjadi Tenaga Kependidikan (tidak ada aksi)
        if let Some(KategoriPegawai::TenagaKependidikan) = &payload.kategori_pegawai {
            // No action needed
        }
    }

    // Tahap 3: Lakukan UPDATE utama pada tabel pegawai (tidak berubah)
    let jenis_kelamin_str = payload.jenis_kelamin.as_ref().map(|e| e.as_str());
    let status_nikah_str = payload.status_nikah.as_ref().map(|e| e.as_str());
    let kategori_pegawai_str = payload.kategori_pegawai.as_ref().map(|e| e.as_str());
    let status_pegawai_str = payload.status_pegawai.as_ref().map(|e| e.as_str());
    
    sqlx::query(
        r#"
        UPDATE pegawai SET
            nik = $1, no_ktp = $2, nama_lengkap = $3, gelar_depan = $4, gelar_belakang = $5,
            tempat_lahir = $6, tanggal_lahir = $7, jenis_kelamin = $8::"JenisKelamin", status_nikah = $9::"StatusNikah",
            agama = $10, gol_darah = $11, alamat_domisili = $12, kota = $13, kode_pos = $14, nomor_hp = $15,
            email = $16, kategori_pegawai = $17::"KategoriPegawai", status_pegawai = $18::"StatusPegawai",
            is_active = $19, unit_kerja = $20, bagian = $21, jabatan = $22, tanggal_masuk = $23,
            tanggal_pensiun = $24, no_kk = $25, no_npwp = $26, no_bpjs_kesehatan = $27,
            no_bpjs_ketenagakerjaan = $28, updated_at = now()
        WHERE id = $29
        "#,
    )
    .bind(&payload.nik)
    .bind(&payload.no_ktp)
    .bind(&payload.nama_lengkap)
    .bind(&payload.gelar_depan)
    .bind(&payload.gelar_belakang)
    .bind(&payload.tempat_lahir)
    .bind(payload.tanggal_lahir)
    .bind(jenis_kelamin_str)
    .bind(status_nikah_str)
    .bind(&payload.agama)
    .bind(&payload.gol_darah)
    .bind(&payload.alamat_domisili)
    .bind(&payload.kota)
    .bind(&payload.kode_pos)
    .bind(&payload.nomor_hp)
    .bind(&payload.email)
    .bind(kategori_pegawai_str)
    .bind(status_pegawai_str)
    .bind(payload.is_active.unwrap_or(old_pegawai.is_active))
    .bind(&payload.unit_kerja).bind(&payload.bagian).bind(&payload.jabatan).bind(payload.tanggal_masuk)
    .bind(payload.tanggal_pensiun).bind(&payload.no_kk).bind(&payload.no_npwp)
    .bind(&payload.no_bpjs_kesehatan).bind(&payload.no_bpjs_ketenagakerjaan)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    get_pegawai_by_id_repo_inner(pool, id).await
}

async fn get_pegawai_by_id_repo_inner<'a, E>(executor: E, id: Uuid) -> Result<Pegawai, AppError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let pegawai = sqlx::query_as!(
        Pegawai,
        r#"
        SELECT
            p.id, p.user_id, p.nik, p.no_ktp, p.nama_lengkap, p.gelar_depan, p.gelar_belakang,
            p.tempat_lahir, p.tanggal_lahir, p.jenis_kelamin as "jenis_kelamin: _",
            p.status_nikah as "status_nikah: _", p.agama, p.gol_darah, p.alamat_domisili,
            p.kota, p.kode_pos, p.nomor_hp, p.email, p.kategori_pegawai as "kategori_pegawai: _",
            p.status_pegawai as "status_pegawai: _", p.is_active, p.unit_kerja, p.bagian,
            p.jabatan, p.tanggal_masuk, p.tanggal_pensiun, p.no_kk, p.no_npwp,
            p.no_bpjs_kesehatan, p.no_bpjs_ketenagakerjaan,
            d.nidn, d.prodi_id, prodi.nama_prodi,
            p.created_at, p.updated_at
        FROM pegawai p
        LEFT JOIN dosen d ON p.id = d.pegawai_id
        LEFT JOIN prodi ON d.prodi_id = prodi.id
        WHERE p.id = $1
        "#,
        id
    )
    .fetch_one(executor)
    .await?;

    Ok(pegawai)
}

/// Menghapus data pegawai
pub async fn delete_pegawai_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // 1. Ambil detail pegawai yang akan dihapus menggunakan FUNGSI INNER
    let pegawai_to_delete = get_pegawai_by_id_repo_inner(&mut *tx, id).await?;

    // 2. LOGIKA BARU: Jika pegawai adalah dosen, periksa keterkaitan data
    if let Some(KategoriPegawai::TenagaPendidik) = pegawai_to_delete.kategori_pegawai {
        // Cari ID dosen yang terkait dengan pegawai ini
        let dosen_id_rec = sqlx::query!("SELECT id FROM dosen WHERE pegawai_id = $1", id)
            .fetch_optional(&mut *tx).await?;

        if let Some(rec) = dosen_id_rec {
            // Periksa apakah dosen_id ini digunakan di jadwal_dosen_pengampu
            let is_linked = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM jadwal_dosen_pengampu WHERE dosen_id = $1)",
                rec.id
            ).fetch_one(&mut *tx).await?.unwrap_or(false);

            if is_linked {
                // Jika terikat, batalkan penghapusan
                tx.rollback().await?;
                return Err(AppError::Forbidden("Pegawai ini tidak dapat dihapus karena terikat dengan data jadwal akademik.".to_string()));
            } else {
                // Jika tidak terikat, hapus dari tabel dosen terlebih dahulu
                sqlx::query!("DELETE FROM dosen WHERE id = $1", rec.id).execute(&mut *tx).await?;
            }
        }
    }

    // 3. Hapus dari tabel pegawai
    sqlx::query!("DELETE FROM pegawai WHERE id = $1", id).execute(&mut *tx).await?;

    // 4. Hapus user terkait jika ada
    if let Some(user_id) = pegawai_to_delete.user_id {
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id).execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn create_user_for_pegawai_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    payload: CreateUserForPegawaiPayload,
) -> Result<Pegawai, AppError> {
    let mut tx = pool.begin().await?;

    // 1. Ambil data pegawai dan pastikan belum punya user
    let pegawai = get_pegawai_by_id_repo_inner(&mut *tx, pegawai_id).await?;
    if pegawai.user_id.is_some() {
        return Err(AppError::Forbidden("Pegawai ini sudah memiliki akun user.".to_string()));
    }

    // 2. Buat user baru
    let hashed_password = bcrypt::hash(payload.password, bcrypt::DEFAULT_COST)?;
    let new_user_id = sqlx::query_scalar!(
        "INSERT INTO users (username, password_hash, full_name, email) VALUES ($1, $2, $3, $4) RETURNING id",
        pegawai.nik, // Gunakan NIK dari data pegawai sebagai username
        hashed_password,
        pegawai.nama_lengkap,
        pegawai.email
    ).fetch_one(&mut *tx).await?;

    // 3. Tautkan user_id baru ke data pegawai
    sqlx::query!(
        "UPDATE pegawai SET user_id = $1 WHERE id = $2",
        new_user_id,
        pegawai_id
    ).execute(&mut *tx).await?;

    // 4. Jika pegawai adalah Dosen, tautkan juga user_id ke data dosen
    if let Some(KategoriPegawai::TenagaPendidik) = pegawai.kategori_pegawai {
        sqlx::query!(
            "UPDATE dosen SET user_id = $1 WHERE pegawai_id = $2",
            new_user_id,
            pegawai_id
        ).execute(&mut *tx).await?;
        
        // Berikan peran DOSEN
        sqlx::query!(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, (SELECT id FROM roles WHERE name = 'DOSEN')) ON CONFLICT DO NOTHING",
            new_user_id
        ).execute(&mut *tx).await?;
    }
    // Anda bisa tambahkan logika `else` untuk memberikan peran default 'PEGAWAI' jika perlu

    tx.commit().await?;

    // Ambil dan kembalikan data pegawai terbaru yang sudah memiliki user_id
    get_pegawai_by_id_repo_inner(pool, pegawai_id).await
}
