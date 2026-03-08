// src/modules/sdm/repo.rs

use super::model::{
    CreateUserForPegawaiPayload, JenisKelamin, KategoriPegawai, Pegawai, PegawaiPayload,
    StatusNikah, StatusPegawai,
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

    let new_user_id: Option<Uuid> = if let Some(password) = payload.password {
        let existing_user = if let Some(email) = &payload.email {
            sqlx::query!("SELECT id FROM users WHERE email = $1", email)
                .fetch_optional(&mut *tx)
                .await?
        } else {
            None
        };

        if let Some(user) = existing_user {
            Some(user.id)
        } else {
            let hashed_password = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
            match sqlx::query!(
                "INSERT INTO users (username, password_hash, full_name, email) VALUES ($1, $2, $3, $4) RETURNING id",
                &payload.nik, 
                hashed_password,
                &payload.nama_lengkap,
                payload.email.as_deref()
            ).fetch_one(&mut *tx).await {
                Ok(rec) => Some(rec.id),
                Err(e) => {
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

    let jenis_kelamin_str = payload.jenis_kelamin.as_ref().map(|e| e.as_str());
    let status_nikah_str = payload.status_nikah.as_ref().map(|e| e.as_str());
    let kategori_pegawai_str = payload.kategori_pegawai.as_ref().map(|e| e.as_str());
    let status_pegawai_str = payload.status_pegawai.as_ref().map(|e| e.as_str());

    // Insert data ke tabel pegawai (Dengan kolom-kolom feeder baru)
    let new_pegawai_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO pegawai (
            user_id, nik, no_ktp, nama_lengkap, gelar_depan, gelar_belakang, tempat_lahir, tanggal_lahir, 
            jenis_kelamin, status_nikah, agama, gol_darah, alamat_domisili, kota, kode_pos, 
            nomor_hp, email, kategori_pegawai, status_pegawai, is_active, 
            tanggal_masuk, tanggal_pensiun, no_kk, no_npwp, no_bpjs_kesehatan, no_bpjs_ketenagakerjaan,
            id_sdm_feeder, nama_ibu_kandung, kewarganegaraan, dusun, rt, rw, kelurahan, id_wilayah_feeder
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9::"JenisKelamin", $10::"StatusNikah", $11, $12, $13, $14, $15, 
            $16, $17, $18::"KategoriPegawai", $19::"StatusPegawai", $20, 
            $21, $22, $23, $24, $25, $26,
            $27, $28, $29, $30, $31, $32, $33, $34
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
    .bind(payload.tanggal_masuk)
    .bind(payload.tanggal_pensiun)
    .bind(&payload.no_kk)
    .bind(&payload.no_npwp)
    .bind(&payload.no_bpjs_kesehatan)
    .bind(&payload.no_bpjs_ketenagakerjaan)
    .bind(payload.id_sdm_feeder)
    .bind(&payload.nama_ibu_kandung)
    .bind(payload.kewarganegaraan.as_deref().unwrap_or("ID"))
    .bind(&payload.dusun)
    .bind(&payload.rt)
    .bind(&payload.rw)
    .bind(&payload.kelurahan)
    .bind(payload.id_wilayah_feeder)
    .fetch_one(&mut *tx)
    .await?;

    // Insert ke tabel penempatan_pegawai jika data unit_kerja_id dikirim
    if let (Some(unit_id), Some(jabatan_nama)) = (payload.unit_kerja_id, &payload.jabatan) {
        let tgl_mulai = payload.tanggal_masuk.unwrap_or_else(|| time::OffsetDateTime::now_utc().date());
        sqlx::query!(
            "INSERT INTO penempatan_pegawai (pegawai_id, unit_kerja_id, jabatan, tanggal_mulai) VALUES ($1, $2, $3, $4)",
            new_pegawai_id,
            unit_id,
            jabatan_nama,
            tgl_mulai
        )
        .execute(&mut *tx)
        .await?;
    }

    // Jika Tenaga Pendidik, buat data Dosen
    if let Some(KategoriPegawai::TenagaPendidik) = &payload.kategori_pegawai {
        let nidn = payload.nidn.as_ref().ok_or_else(|| {
            AppError::Forbidden("NIDN wajib diisi untuk Tenaga Pendidik.".to_string())
        })?;
        let prodi_id = payload.prodi_id.ok_or_else(|| {
            AppError::Forbidden("Prodi ID wajib diisi untuk Tenaga Pendidik.".to_string())
        })?;

        let existing_dosen = sqlx::query!("SELECT id FROM dosen WHERE nidn = $1", nidn)
            .fetch_optional(&mut *tx)
            .await?;

        if let Some(dosen) = existing_dosen {
            sqlx::query!(
                "UPDATE dosen SET pegawai_id = $1, user_id = $2, id_penugasan_feeder = $3, ikatan_kerja = $4 WHERE id = $5",
                new_pegawai_id,
                new_user_id,
                payload.id_penugasan_feeder,
                payload.ikatan_kerja,
                dosen.id
            )
            .execute(&mut *tx)
            .await?;
        } else {
            // PERHATIKAN: kolom nama_dosen dan email sudah tidak ada!
            sqlx::query!(
                "INSERT INTO dosen (nidn, prodi_id, user_id, pegawai_id, id_penugasan_feeder, ikatan_kerja) VALUES ($1, $2, $3, $4, $5, $6)",
                nidn, prodi_id, new_user_id, new_pegawai_id, payload.id_penugasan_feeder, payload.ikatan_kerja
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
    get_pegawai_by_id_repo(pool, new_pegawai_id).await
}

/// Mengambil semua data pegawai
pub async fn get_all_pegawai_repo(pool: &DbPool) -> Result<Vec<Pegawai>, AppError> {
    let records = sqlx::query!(
        r#"
        SELECT
            p.id, p.user_id, p.nik, p.no_ktp, p.nama_lengkap, p.gelar_depan, p.gelar_belakang,
            p.tempat_lahir, p.tanggal_lahir, p.jenis_kelamin as "jenis_kelamin: JenisKelamin",
            p.status_nikah as "status_nikah: StatusNikah", p.agama, p.gol_darah, p.alamat_domisili,
            p.kota, p.kode_pos, p.nomor_hp, p.email,
            p.kategori_pegawai as "kategori_pegawai: KategoriPegawai",
            p.status_pegawai as "status_pegawai: StatusPegawai",
            p.is_active, p.tanggal_masuk, p.tanggal_pensiun, p.no_kk, p.no_npwp,
            p.no_bpjs_kesehatan, p.no_bpjs_ketenagakerjaan,
            p.id_sdm_feeder, p.nama_ibu_kandung, p.kewarganegaraan, p.dusun, p.rt, p.rw, p.kelurahan, p.id_wilayah_feeder,
            d.nidn as "nidn?", d.prodi_id as "prodi_id?", pr.nama_prodi as "nama_prodi?",
            d.id_penugasan_feeder as "id_penugasan_feeder?", d.ikatan_kerja as "ikatan_kerja?",
            p.created_at, p.updated_at
        FROM pegawai p
        LEFT JOIN dosen d ON p.id = d.pegawai_id
        LEFT JOIN prodi pr ON d.prodi_id = pr.id
        ORDER BY p.nama_lengkap ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    let pegawai_list = records
        .into_iter()
        .map(|rec| Pegawai {
            id: rec.id, user_id: rec.user_id, nik: rec.nik, no_ktp: rec.no_ktp,
            nama_lengkap: rec.nama_lengkap, gelar_depan: rec.gelar_depan, gelar_belakang: rec.gelar_belakang,
            tempat_lahir: rec.tempat_lahir, tanggal_lahir: rec.tanggal_lahir, jenis_kelamin: rec.jenis_kelamin,
            status_nikah: rec.status_nikah, agama: rec.agama, gol_darah: rec.gol_darah, alamat_domisili: rec.alamat_domisili,
            kota: rec.kota, kode_pos: rec.kode_pos, nomor_hp: rec.nomor_hp, email: rec.email,
            kategori_pegawai: rec.kategori_pegawai, status_pegawai: rec.status_pegawai, is_active: rec.is_active,
            tanggal_masuk: rec.tanggal_masuk, tanggal_pensiun: rec.tanggal_pensiun, no_kk: rec.no_kk,
            no_npwp: rec.no_npwp, no_bpjs_kesehatan: rec.no_bpjs_kesehatan, no_bpjs_ketenagakerjaan: rec.no_bpjs_ketenagakerjaan,
            id_sdm_feeder: rec.id_sdm_feeder, nama_ibu_kandung: rec.nama_ibu_kandung, kewarganegaraan: rec.kewarganegaraan,
            dusun: rec.dusun, rt: rec.rt, rw: rec.rw, kelurahan: rec.kelurahan, id_wilayah_feeder: rec.id_wilayah_feeder,
            nidn: rec.nidn, prodi_id: rec.prodi_id, nama_prodi: rec.nama_prodi,
            id_penugasan_feeder: rec.id_penugasan_feeder, ikatan_kerja: rec.ikatan_kerja,
            created_at: rec.created_at, updated_at: rec.updated_at,
        })
        .collect();

    Ok(pegawai_list)
}



/// Memperbarui data pegawai
pub async fn update_pegawai_repo(
    pool: &DbPool,
    id: Uuid,
    payload: PegawaiPayload,
) -> Result<Pegawai, AppError> {
    let mut tx = pool.begin().await?;

    let old_pegawai = get_pegawai_by_id_repo_inner(&mut *tx, id).await?;

    // 1. UPDATE DOSEN
    if old_pegawai.kategori_pegawai != payload.kategori_pegawai {
        if let Some(KategoriPegawai::TenagaPendidik) = &payload.kategori_pegawai {
            let nidn = payload.nidn.as_ref().ok_or_else(|| AppError::Forbidden("NIDN wajib.".to_string()))?;
            let prodi_id = payload.prodi_id.ok_or_else(|| AppError::Forbidden("Prodi ID wajib.".to_string()))?;

            let existing_dosen = sqlx::query!("SELECT id FROM dosen WHERE nidn = $1", nidn).fetch_optional(&mut *tx).await?;

            if let Some(dosen) = existing_dosen {
                sqlx::query!(
                    "UPDATE dosen SET pegawai_id = $1, user_id = $2, id_penugasan_feeder = $3, ikatan_kerja = $4 WHERE id = $5",
                    id, old_pegawai.user_id, payload.id_penugasan_feeder, payload.ikatan_kerja, dosen.id
                ).execute(&mut *tx).await?;
            } else {
                // TANPA nama_dosen dan email
                sqlx::query!(
                    "INSERT INTO dosen (nidn, prodi_id, pegawai_id, user_id, id_penugasan_feeder, ikatan_kerja) VALUES ($1, $2, $3, $4, $5, $6)",
                    nidn, prodi_id, id, old_pegawai.user_id, payload.id_penugasan_feeder, payload.ikatan_kerja
                ).execute(&mut *tx).await?;
            }

            if let Some(user_id) = old_pegawai.user_id {
                sqlx::query!("INSERT INTO user_roles (user_id, role_id) VALUES ($1, (SELECT id FROM roles WHERE name = 'DOSEN')) ON CONFLICT DO NOTHING", user_id).execute(&mut *tx).await?;
            }
        }
    } else if let Some(KategoriPegawai::TenagaPendidik) = &old_pegawai.kategori_pegawai {
        let existing_dosen = sqlx::query!("SELECT id FROM dosen WHERE pegawai_id = $1", id).fetch_optional(&mut *tx).await?;
        if let Some(dosen) = existing_dosen {
            sqlx::query!(
                "UPDATE dosen SET prodi_id = COALESCE($1, prodi_id), id_penugasan_feeder = $2, ikatan_kerja = $3 WHERE id = $4",
                payload.prodi_id, payload.id_penugasan_feeder, payload.ikatan_kerja, dosen.id
            ).execute(&mut *tx).await?;
        }
    }

    // 2. UPDATE PEGAWAI
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
            is_active = $19, tanggal_masuk = $20,
            tanggal_pensiun = $21, no_kk = $22, no_npwp = $23, no_bpjs_kesehatan = $24, no_bpjs_ketenagakerjaan = $25,
            id_sdm_feeder = $26, nama_ibu_kandung = $27, kewarganegaraan = $28, dusun = $29, rt = $30, rw = $31, kelurahan = $32, id_wilayah_feeder = $33,
            updated_at = now()
        WHERE id = $34
        "#,
    )
    .bind(&payload.nik).bind(&payload.no_ktp).bind(&payload.nama_lengkap).bind(&payload.gelar_depan).bind(&payload.gelar_belakang)
    .bind(&payload.tempat_lahir).bind(payload.tanggal_lahir).bind(jenis_kelamin_str).bind(status_nikah_str)
    .bind(&payload.agama).bind(&payload.gol_darah).bind(&payload.alamat_domisili).bind(&payload.kota).bind(&payload.kode_pos)
    .bind(&payload.nomor_hp).bind(&payload.email).bind(kategori_pegawai_str).bind(status_pegawai_str)
    .bind(payload.is_active.unwrap_or(old_pegawai.is_active)).bind(payload.tanggal_masuk).bind(payload.tanggal_pensiun)
    .bind(&payload.no_kk).bind(&payload.no_npwp).bind(&payload.no_bpjs_kesehatan).bind(&payload.no_bpjs_ketenagakerjaan)
    // Feeder binding
    .bind(payload.id_sdm_feeder).bind(&payload.nama_ibu_kandung).bind(payload.kewarganegaraan.as_deref().unwrap_or("ID"))
    .bind(&payload.dusun).bind(&payload.rt).bind(&payload.rw).bind(&payload.kelurahan).bind(payload.id_wilayah_feeder)
    .bind(id)
    .execute(&mut *tx).await?;

    // 3. UPDATE PENEMPATAN
    if let (Some(new_unit_id), Some(new_jabatan)) = (payload.unit_kerja_id, &payload.jabatan) {
        let active_penempatan = sqlx::query!("SELECT id FROM penempatan_pegawai WHERE pegawai_id = $1 AND tanggal_selesai IS NULL", id).fetch_optional(&mut *tx).await?;

        if let Some(penempatan) = active_penempatan {
            sqlx::query!("UPDATE penempatan_pegawai SET unit_kerja_id = $1, jabatan = $2, updated_at = now() WHERE id = $3", new_unit_id, new_jabatan, penempatan.id).execute(&mut *tx).await?;
        } else {
            let tgl_mulai = payload.tanggal_masuk.unwrap_or_else(|| time::OffsetDateTime::now_utc().date());
            sqlx::query!("INSERT INTO penempatan_pegawai (pegawai_id, unit_kerja_id, jabatan, tanggal_mulai) VALUES ($1, $2, $3, $4)", id, new_unit_id, new_jabatan, tgl_mulai).execute(&mut *tx).await?;
        }
    }

    tx.commit().await?;
    get_pegawai_by_id_repo_inner(pool, id).await
}


async fn get_pegawai_by_id_repo_inner<'a, E>(executor: E, id: Uuid) -> Result<Pegawai, AppError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let rec = sqlx::query!(
        r#"
        SELECT
            p.id, p.user_id, p.nik, p.no_ktp, p.nama_lengkap, p.gelar_depan, p.gelar_belakang,
            p.tempat_lahir, p.tanggal_lahir, p.jenis_kelamin as "jenis_kelamin: JenisKelamin",
            p.status_nikah as "status_nikah: StatusNikah", p.agama, p.gol_darah, p.alamat_domisili,
            p.kota, p.kode_pos, p.nomor_hp, p.email,
            p.kategori_pegawai as "kategori_pegawai: KategoriPegawai",
            p.status_pegawai as "status_pegawai: StatusPegawai",
            p.is_active, p.tanggal_masuk, p.tanggal_pensiun, p.no_kk, p.no_npwp,
            p.no_bpjs_kesehatan, p.no_bpjs_ketenagakerjaan,
            p.id_sdm_feeder, p.nama_ibu_kandung, p.kewarganegaraan, p.dusun, p.rt, p.rw, p.kelurahan, p.id_wilayah_feeder,
            d.nidn as "nidn?", d.prodi_id as "prodi_id?", pr.nama_prodi as "nama_prodi?",
            d.id_penugasan_feeder as "id_penugasan_feeder?", d.ikatan_kerja as "ikatan_kerja?",
            p.created_at, p.updated_at
        FROM pegawai p
        LEFT JOIN dosen d ON p.id = d.pegawai_id
        LEFT JOIN prodi pr ON d.prodi_id = pr.id
        WHERE p.id = $1
        "#,
        id
    )
    .fetch_one(executor)
    .await?;

    Ok(Pegawai {
        id: rec.id, user_id: rec.user_id, nik: rec.nik, no_ktp: rec.no_ktp,
        nama_lengkap: rec.nama_lengkap, gelar_depan: rec.gelar_depan, gelar_belakang: rec.gelar_belakang,
        tempat_lahir: rec.tempat_lahir, tanggal_lahir: rec.tanggal_lahir, jenis_kelamin: rec.jenis_kelamin,
        status_nikah: rec.status_nikah, agama: rec.agama, gol_darah: rec.gol_darah, alamat_domisili: rec.alamat_domisili,
        kota: rec.kota, kode_pos: rec.kode_pos, nomor_hp: rec.nomor_hp, email: rec.email,
        kategori_pegawai: rec.kategori_pegawai, status_pegawai: rec.status_pegawai, is_active: rec.is_active,
        tanggal_masuk: rec.tanggal_masuk, tanggal_pensiun: rec.tanggal_pensiun, no_kk: rec.no_kk,
        no_npwp: rec.no_npwp, no_bpjs_kesehatan: rec.no_bpjs_kesehatan, no_bpjs_ketenagakerjaan: rec.no_bpjs_ketenagakerjaan,
        id_sdm_feeder: rec.id_sdm_feeder, nama_ibu_kandung: rec.nama_ibu_kandung, kewarganegaraan: rec.kewarganegaraan,
        dusun: rec.dusun, rt: rec.rt, rw: rec.rw, kelurahan: rec.kelurahan, id_wilayah_feeder: rec.id_wilayah_feeder,
        nidn: rec.nidn, prodi_id: rec.prodi_id, nama_prodi: rec.nama_prodi,
        id_penugasan_feeder: rec.id_penugasan_feeder, ikatan_kerja: rec.ikatan_kerja,
        created_at: rec.created_at, updated_at: rec.updated_at,
    })
}

/// Menghapus data pegawai
pub async fn delete_pegawai_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let pegawai_to_delete = get_pegawai_by_id_repo_inner(&mut *tx, id).await?;

    if let Some(KategoriPegawai::TenagaPendidik) = pegawai_to_delete.kategori_pegawai {
        let dosen_id_rec = sqlx::query!("SELECT id FROM dosen WHERE pegawai_id = $1", id).fetch_optional(&mut *tx).await?;

        if let Some(rec) = dosen_id_rec {
            let is_linked = sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM jadwal_dosen_pengampu WHERE dosen_id = $1)", rec.id).fetch_one(&mut *tx).await?.unwrap_or(false);
            if is_linked {
                tx.rollback().await?;
                return Err(AppError::Forbidden("Pegawai ini tidak dapat dihapus karena terikat dengan jadwal akademik.".to_string()));
            } else {
                sqlx::query!("DELETE FROM dosen WHERE id = $1", rec.id).execute(&mut *tx).await?;
            }
        }
    }

    sqlx::query!("DELETE FROM pegawai WHERE id = $1", id).execute(&mut *tx).await?;

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

    let pegawai = get_pegawai_by_id_repo_inner(&mut *tx, pegawai_id).await?;
    if pegawai.user_id.is_some() {
        return Err(AppError::Forbidden("Pegawai ini sudah memiliki akun user.".to_string()));
    }

    let hashed_password = bcrypt::hash(payload.password, bcrypt::DEFAULT_COST)?;
    let new_user_id = sqlx::query_scalar!(
        "INSERT INTO users (username, password_hash, full_name, email) VALUES ($1, $2, $3, $4) RETURNING id",
        pegawai.nik, hashed_password, pegawai.nama_lengkap, pegawai.email
    ).fetch_one(&mut *tx).await?;

    sqlx::query!("UPDATE pegawai SET user_id = $1 WHERE id = $2", new_user_id, pegawai_id).execute(&mut *tx).await?;
    sqlx::query!("INSERT INTO user_roles (user_id, role_id) VALUES ($1, (SELECT id FROM roles WHERE name = 'KARYAWAN')) ON CONFLICT DO NOTHING", new_user_id).execute(&mut *tx).await?;

    if let Some(KategoriPegawai::TenagaPendidik) = pegawai.kategori_pegawai {
        sqlx::query!("UPDATE dosen SET user_id = $1 WHERE pegawai_id = $2", new_user_id, pegawai_id).execute(&mut *tx).await?;
        sqlx::query!("INSERT INTO user_roles (user_id, role_id) VALUES ($1, (SELECT id FROM roles WHERE name = 'DOSEN')) ON CONFLICT DO NOTHING", new_user_id).execute(&mut *tx).await?;
    }

    tx.commit().await?;
    get_pegawai_by_id_repo_inner(pool, pegawai_id).await
}

pub async fn get_pegawai_id_from_user_id_repo(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Uuid, AppError> {
    let pegawai_id = sqlx::query_scalar!("SELECT id FROM pegawai WHERE user_id = $1", user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::Forbidden("Tidak ada data pegawai yang terhubung dengan akun Anda.".to_string()),
            _ => e.into(),
        })?;
    Ok(pegawai_id)
}