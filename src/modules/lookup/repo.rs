use crate::{
    db::DbPool,
    errors::AppError,
    modules::{user_management::model::UserLookup,aset::ruangan_model::{RuanganLookup, RuanganTersediaFilter} },
};
use time::{Weekday, Duration};

pub async fn search_users_repo(pool: &DbPool, search_term: &str) -> Result<Vec<UserLookup>, AppError> {
    let search_pattern = format!("%{}%", search_term);

    let users = sqlx::query_as!(
        UserLookup,
        "SELECT id, username, full_name FROM users WHERE username ILIKE $1 OR full_name ILIKE $1 ORDER BY full_name LIMIT 10",
        search_pattern
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}

pub async fn search_ruangan_tersedia_repo(
    pool: &DbPool,
    filter: RuanganTersediaFilter,
) -> Result<Vec<RuanganLookup>, AppError> {
    // 1. Ambil detail jadwal & periode tahun akademiknya
    let jadwal = sqlx::query!(
        r#"
        SELECT jk.hari::TEXT as hari, jk.jam_mulai, jk.jam_selesai,
               ta.tanggal_mulai, ta.tanggal_selesai
        FROM jadwal_kuliah jk
        JOIN tahun_akademik ta ON jk.tahun_akademik_id = ta.id
        WHERE jk.id = $1
        "#,
        filter.jadwal_kuliah_id
    )
    .fetch_one(pool)
    .await?;

    // 2. Hitung semua instance waktu yang perlu dicek
    let mut instances = Vec::new();
    let mut current_date = jadwal.tanggal_mulai;
    let target_weekday = match jadwal.hari.as_deref() {
        Some("Senin") => Weekday::Monday,
        Some("Selasa") => Weekday::Tuesday,
        Some("Rabu") => Weekday::Wednesday,
        Some("Kamis") => Weekday::Thursday,
        Some("Jumat") => Weekday::Friday,
        Some("Sabtu") => Weekday::Saturday,
        _ => Weekday::Sunday,
    };

    while current_date <= jadwal.tanggal_selesai {
        if current_date.weekday() == target_weekday {
            let waktu_mulai = current_date.with_time(jadwal.jam_mulai).assume_utc();
            let waktu_selesai = current_date.with_time(jadwal.jam_selesai).assume_utc();
            instances.push((waktu_mulai, waktu_selesai));
        }
        current_date += Duration::days(1);
    }

    // Jika tidak ada instance tanggal yang valid, kembalikan daftar kosong
    if instances.is_empty() {
        return Ok(Vec::new());
    }

    // 3. Bangun query untuk mencari ruangan yang tidak bentrok
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT id, kode_ruangan, nama_ruangan, kapasitas FROM ruangan r WHERE NOT EXISTS (",
    );
    query_builder.push("SELECT 1 FROM jadwal_ruangan jr WHERE jr.ruangan_id = r.id AND (");

    let mut first = true;
    for (start_time, end_time) in instances {
        if !first {
            query_builder.push(" OR ");
        }
        query_builder.push("(jr.waktu_mulai, jr.waktu_selesai) OVERLAPS (");
        query_builder.push_bind(start_time);
        query_builder.push(", ");
        query_builder.push_bind(end_time);
        query_builder.push(")");
        first = false;
    }
    query_builder.push("))");

    if let Some(q) = filter.q {
        query_builder.push(" AND (r.nama_ruangan ILIKE ");
        query_builder.push_bind(format!("%{}%", q));
        query_builder.push(" OR r.kode_ruangan ILIKE ");
        query_builder.push_bind(format!("%{}%", q));
        query_builder.push(")");
    }
    
    query_builder.push(" ORDER BY r.kapasitas DESC, r.kode_ruangan ASC");

    let ruangan_list = query_builder
        .build_query_as::<RuanganLookup>()
        .fetch_all(pool)
        .await?;
        
    Ok(ruangan_list)
}
