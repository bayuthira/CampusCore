use crate::{
    db::DbPool,
    errors::AppError,
    modules::fleet::kendaraan_model::{
        AvailableVehicleFilter, Kendaraan, KendaraanLookup, KendaraanPayload, KendaraanSummary,
        SummaryFilter,
    },
};
use rust_decimal::{Decimal, prelude::Zero};
use sqlx::Row;
use uuid::Uuid;

pub async fn create_repo(pool: &DbPool, payload: KendaraanPayload) -> Result<Kendaraan, AppError> {
    let jenis_str = payload.jenis.as_str();

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO kendaraan (jenis, nama, nomor_polisi, merk, model, tahun) 
        VALUES ($1::"JenisKendaraan", $2, $3, $4, $5, $6) RETURNING id
        "#,
    )
    .bind(jenis_str)
    .bind(payload.nama)
    .bind(payload.nomor_polisi)
    .bind(payload.merk)
    .bind(payload.model)
    .bind(payload.tahun)
    .fetch_one(pool)
    .await?;

    let new_item = get_by_id_repo(pool, id).await?;
    Ok(new_item)
}

pub async fn get_all_repo(pool: &DbPool) -> Result<Vec<Kendaraan>, AppError> {
    let list = sqlx::query_as!(
        Kendaraan,
        r#"SELECT id, jenis as "jenis: _", nama, nomor_polisi, merk, model, tahun, status as "status: _", created_at, updated_at 
        FROM kendaraan ORDER BY nama ASC"#
    ).fetch_all(pool).await?;
    Ok(list)
}

pub async fn get_by_id_repo(pool: &DbPool, id: Uuid) -> Result<Kendaraan, AppError> {
    let item = sqlx::query_as!(
        Kendaraan,
        r#"SELECT id, jenis as "jenis: _", nama, nomor_polisi, merk, model, tahun, status as "status: _", created_at, updated_at 
        FROM kendaraan WHERE id = $1"#,
        id
    ).fetch_one(pool).await?;
    Ok(item)
}

pub async fn update_repo(
    pool: &DbPool,
    id: Uuid,
    payload: KendaraanPayload,
) -> Result<Kendaraan, AppError> {
    let jenis_str = payload.jenis.as_str();
    sqlx::query(
        r#"
        UPDATE kendaraan SET jenis = $1::"JenisKendaraan", nama = $2, nomor_polisi = $3, 
        merk = $4, model = $5, tahun = $6, updated_at = now() 
        WHERE id = $7
        "#,
    )
    .bind(jenis_str)
    .bind(payload.nama)
    .bind(payload.nomor_polisi)
    .bind(payload.merk)
    .bind(payload.model)
    .bind(payload.tahun)
    .bind(id)
    .execute(pool)
    .await?;

    let updated_item = get_by_id_repo(pool, id).await?;
    Ok(updated_item)
}

pub async fn delete_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM kendaraan WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}

pub async fn search_available_vehicles_repo(
    pool: &DbPool,
    filter: AvailableVehicleFilter,
) -> Result<Vec<KendaraanLookup>, AppError> {
    let available_vehicles = sqlx::query_as!(
        KendaraanLookup,
        r#"
        SELECT k.id, k.jenis as "jenis: _", k.nama, k.nomor_polisi
        FROM kendaraan k
        WHERE k.status = 'Tersedia' AND NOT EXISTS (
            SELECT 1
            FROM booking_kendaraan bk
            WHERE bk.kendaraan_id = k.id
            AND bk.status IN ('Disetujui', 'Berlangsung', 'Diajukan')
            AND (bk.waktu_berangkat, bk.estimasi_waktu_kembali) OVERLAPS ($1, $2)
        )
        ORDER BY k.nama
        "#,
        filter.start,
        filter.end
    )
    .fetch_all(pool)
    .await?;

    Ok(available_vehicles)
}

pub async fn get_vehicle_summary_repo(
    pool: &DbPool,
    kendaraan_id: Uuid,
    filter: SummaryFilter,
) -> Result<KendaraanSummary, AppError> {
    // 1. Hitung total biaya servis dengan filter tanggal
    let mut biaya_query = sqlx::QueryBuilder::new(
        "SELECT SUM(biaya) as total FROM servis_kendaraan WHERE kendaraan_id = ",
    );
    biaya_query.push_bind(kendaraan_id);
    if let (Some(start), Some(end)) = (filter.start_date, filter.end_date) {
        biaya_query.push(" AND tanggal_servis BETWEEN ");
        biaya_query.push_bind(start);
        biaya_query.push(" AND ");
        biaya_query.push_bind(end);
    }
    let total_biaya_servis: Decimal = biaya_query
        .build()
        .fetch_one(pool)
        .await?
        .try_get("total")
        .unwrap_or(Decimal::zero());

    // 2. Hitung total jarak tempuh dengan filter tanggal
    let mut jarak_query = sqlx::QueryBuilder::new(
        "SELECT SUM(odometer_akhir - odometer_awal) as total FROM log_penggunaan WHERE booking_id IN (SELECT id FROM booking_kendaraan WHERE kendaraan_id = ",
    );
    jarak_query.push_bind(kendaraan_id);
    jarak_query.push(")");

    if let (Some(start), Some(end)) = (filter.start_date, filter.end_date) {
        // Filter berdasarkan tanggal aktual kembali
        jarak_query.push(" AND DATE(waktu_aktual_kembali) BETWEEN ");
        jarak_query.push_bind(start);
        jarak_query.push(" AND ");
        jarak_query.push_bind(end);
    }
    let total_jarak_tempuh: i64 = jarak_query
        .build()
        .fetch_one(pool)
        .await?
        .try_get("total")
        .unwrap_or(0);

    // 3. Hitung biaya per km
    let biaya_per_km = if total_jarak_tempuh > 0 {
        (total_biaya_servis / Decimal::from(total_jarak_tempuh)).round_dp(2)
    } else {
        Decimal::zero()
    };

    Ok(KendaraanSummary {
        total_biaya_servis,
        total_jarak_tempuh,
        biaya_per_km,
    })
}
