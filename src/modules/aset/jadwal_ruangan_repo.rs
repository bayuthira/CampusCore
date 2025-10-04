use crate::{
    db::DbPool,
    errors::AppError,
};
use super::jadwal_ruangan_model::{CreateJadwalPayload, JadwalRuangan, TipePerulangan,JadwalRuanganFilter};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

pub async fn create_jadwal_repo(
    pool: &DbPool,
    user_pembuat_id: Uuid,
    payload: CreateJadwalPayload,
) -> Result<Vec<JadwalRuangan>, AppError> {
    let mut instances_to_create: Vec<OffsetDateTime> = vec![payload.waktu_mulai];
    let recurring_event_id: Option<Uuid>;

    // 1. Hitung semua tanggal kejadian dan tentukan recurring_event_id
    if let (Some(tipe), Some(end_date)) = (payload.tipe_perulangan, payload.tanggal_akhir_perulangan) {
        recurring_event_id = Some(Uuid::new_v4()); // <-- Isi nilai di cabang `if`
        let mut current_start_time = payload.waktu_mulai;
        let duration_to_add = match tipe {
            TipePerulangan::Harian => Duration::days(1),
            TipePerulangan::Mingguan => Duration::weeks(1),
        };

        loop {
            current_start_time += duration_to_add;
            if current_start_time.date() > end_date {
                break;
            }
            instances_to_create.push(current_start_time);
        }
    } else {
        recurring_event_id = None; // <-- Isi nilai di cabang `else`
    }

    let mut tx = pool.begin().await?;
    let mut created_jadwal_ids: Vec<Uuid> = Vec::new();

    // 2. Loop melalui setiap instance dan lakukan pengecekan & insert
    for start_time in instances_to_create {
        let end_time = start_time + (payload.waktu_selesai - payload.waktu_mulai);

        // Cek konflik jadwal
        let conflict = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM jadwal_ruangan
                WHERE ruangan_id = $1 AND (waktu_mulai, waktu_selesai) OVERLAPS ($2, $3)
            )
            "#,
            payload.ruangan_id,
            start_time,
            end_time
        )
        .fetch_one(&mut *tx)
        .await?;

        if conflict.unwrap_or(false) {
            return Err(AppError::Forbidden(format!(
                "Konflik jadwal untuk ruangan pada {}.",
                start_time.date()
            )));
        }

        // Jika tidak ada konflik, insert ke database
        let new_id = sqlx::query_scalar!(
            "INSERT INTO jadwal_ruangan (ruangan_id, judul_kegiatan, deskripsi, waktu_mulai, waktu_selesai, recurring_event_id, user_pembuat_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            payload.ruangan_id, payload.judul_kegiatan, payload.deskripsi, start_time, end_time, recurring_event_id, user_pembuat_id
        ).fetch_one(&mut *tx).await?;

        created_jadwal_ids.push(new_id);
    }

    tx.commit().await?;
    
    // (Sisa fungsi untuk mengambil data kembali tidak berubah)
    let new_jadwals = sqlx::query_as!(
        JadwalRuangan,
        r#"
        SELECT 
            j.id, j.ruangan_id, j.judul_kegiatan, j.deskripsi,
            j.waktu_mulai, j.waktu_selesai, j.recurring_event_id,j.jadwal_kuliah_id,
            j.user_pembuat_id, u.full_name as "nama_pembuat!"
        FROM jadwal_ruangan j
        JOIN users u ON j.user_pembuat_id = u.id
        WHERE j.id = ANY($1)
        "#,
        &created_jadwal_ids
    ).fetch_all(pool).await?;
    
    Ok(new_jadwals)
}

pub async fn get_jadwal_by_ruangan_repo(
    pool: &DbPool,
    ruangan_id: Uuid,
    filter: JadwalRuanganFilter,
) -> Result<Vec<JadwalRuangan>, AppError> {
    let jadwal_list = sqlx::query_as!(
        JadwalRuangan,
        r#"
        SELECT 
            j.id, j.ruangan_id, j.judul_kegiatan, j.deskripsi,
            j.waktu_mulai, j.waktu_selesai, j.recurring_event_id,j.jadwal_kuliah_id,
            j.user_pembuat_id, u.full_name as "nama_pembuat!"
        FROM jadwal_ruangan j
        JOIN users u ON j.user_pembuat_id = u.id
        WHERE j.ruangan_id = $1 AND j.waktu_mulai >= $2 AND j.waktu_selesai <= $3
        ORDER BY j.waktu_mulai ASC
        "#,
        ruangan_id,
        filter.start,
        filter.end
    )
    .fetch_all(pool)
    .await?;

    Ok(jadwal_list)
}

pub async fn delete_jadwal_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM jadwal_ruangan WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }

    Ok(())
}

pub async fn delete_recurring_jadwal_repo(pool: &DbPool, recurring_id: Uuid) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM jadwal_ruangan WHERE recurring_event_id = $1",
        recurring_id
    )
    .execute(pool)
    .await?;

    Ok(())
}