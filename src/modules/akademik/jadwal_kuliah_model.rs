// src/modules/akademik/jadwal_kuliah_model.rs
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::{Time, UtcOffset, macros::format_description};
use uuid::Uuid;

// Custom struct for Time with Timezone
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeWithOffset {
    pub time: Time,
    pub offset: UtcOffset,
}

// Serialize/Deserialize implementation
impl Serialize for TimeWithOffset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Format: "08:00+07:00"
        let formatted = format!(
            "{:02}:{:02}{:+03}:{:02}",
            self.time.hour(),
            self.time.minute(),
            self.offset.whole_hours(),
            self.offset.minutes_past_hour().abs()
        );
        serializer.serialize_str(&formatted)
    }
}

impl<'de> Deserialize<'de> for TimeWithOffset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // If input is just "HH:MM", assume WIB (+07:00)
        if !s.contains('+') && !s.contains('-') {
            let time = Time::parse(&s, &format_description!("[hour]:[minute]"))
                .map_err(serde::de::Error::custom)?;
            let offset = UtcOffset::from_hms(7, 0, 0).map_err(serde::de::Error::custom)?;
            return Ok(TimeWithOffset { time, offset });
        }

        // Split by + or -
        let (time_str, offset_str, is_positive) = if let Some(pos) = s.find('+') {
            (&s[..pos], &s[pos + 1..], true)
        } else if let Some(pos) = s.rfind('-') {
            if pos > 0 {
                (&s[..pos], &s[pos + 1..], false)
            } else {
                return Err(serde::de::Error::custom("Invalid time format"));
            }
        } else {
            return Err(serde::de::Error::custom("Invalid time format"));
        };

        // Parse time part
        let time = Time::parse(time_str, &format_description!("[hour]:[minute]"))
            .map_err(serde::de::Error::custom)?;

        // Parse offset part "07:00"
        let offset_parts: Vec<&str> = offset_str.split(':').collect();
        if offset_parts.len() != 2 {
            return Err(serde::de::Error::custom("Invalid offset format"));
        }

        let hours: i8 = offset_parts[0].parse().map_err(serde::de::Error::custom)?;
        let minutes: i8 = offset_parts[1].parse().map_err(serde::de::Error::custom)?;

        // Apply sign
        let (hours, minutes) = if is_positive {
            (hours, minutes)
        } else {
            (-hours, -minutes)
        };

        let offset = UtcOffset::from_hms(hours, minutes, 0).map_err(serde::de::Error::custom)?;

        Ok(TimeWithOffset { time, offset })
    }
}

// SQLx support - store as TIMETZ in PostgreSQL
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TimeWithOffset {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let time_with_tz: sqlx::postgres::types::PgTimeTz =
            <sqlx::postgres::types::PgTimeTz as sqlx::Decode<sqlx::Postgres>>::decode(value)?;

        let time = Time::from_hms(
            time_with_tz.time.hour(),
            time_with_tz.time.minute(),
            time_with_tz.time.second(),
        )?;

        let offset = time_with_tz.offset;

        Ok(TimeWithOffset { time, offset })
    }
}

impl sqlx::Type<sqlx::Postgres> for TimeWithOffset {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("TIMETZ")
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for TimeWithOffset {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let pg_time = sqlx::postgres::types::PgTimeTz {
            time: self.time,
            offset: self.offset,
        };

        <sqlx::postgres::types::PgTimeTz as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(
            &pg_time, buf,
        )
    }
}

// Enum untuk mencerminkan ENUM di DB
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "DayOfWeek")]
pub enum DayOfWeek {
    Senin,
    Selasa,
    Rabu,
    Kamis,
    Jumat,
    Sabtu,
    Minggu,
}

impl DayOfWeek {
    pub fn as_str(&self) -> &'static str {
        match self {
            DayOfWeek::Senin => "Senin",
            DayOfWeek::Selasa => "Selasa",
            DayOfWeek::Rabu => "Rabu",
            DayOfWeek::Kamis => "Kamis",
            DayOfWeek::Jumat => "Jumat",
            DayOfWeek::Sabtu => "Sabtu",
            DayOfWeek::Minggu => "Minggu",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "PeranDosenPengampu")]
pub enum PeranDosenPengampu {
    Koordinator,
    Anggota,
}

impl PeranDosenPengampu {
    pub fn as_str(&self) -> &'static str {
        match self {
            PeranDosenPengampu::Koordinator => "Koordinator",
            PeranDosenPengampu::Anggota => "Anggota",
        }
    }
}

// Payload untuk relasi dosen pengampu
#[derive(Debug, Deserialize)]
pub struct DosenPengampuPayload {
    pub dosen_id: Uuid,
    pub peran: PeranDosenPengampu,

    // --- TAMBAHAN FEEDER AKTIVITAS MENGAJAR ---
    pub id_aktivitas_mengajar_feeder: Option<Uuid>,
    pub sks_substansi_total: Option<i32>,
    pub rencana_tatap_muka: Option<i32>,
    pub realisasi_tatap_muka: Option<i32>,
}

// Payload untuk membuat jadwal kuliah baru
#[derive(Debug, Deserialize)]
pub struct CreateJadwalKuliahPayload {
    pub matakuliah_id: Uuid,
    pub tahun_akademik_id: Uuid,
    pub hari: DayOfWeek,
    pub jam_mulai: TimeWithOffset,
    pub jam_selesai: TimeWithOffset,
    pub kelas: String, // Kode kelas internal

    // --- TAMBAHAN FEEDER KELAS KULIAH ---
    pub id_kelas_kuliah_feeder: Option<Uuid>,
    pub nama_kelas_kuliah: Option<String>, // Jika kosong, backend otomatis memakai value dari "kelas"

    pub dosen_pengampu: Vec<DosenPengampuPayload>,
}

#[derive(Debug, Deserialize)]
pub struct PlotJadwalRuanganPayload {
    pub jadwal_kuliah_id: Uuid,
    pub ruangan_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct DosenPengampuDetail {
    pub dosen_id: Uuid,
    pub nama_dosen: String,
    pub peran: PeranDosenPengampu,

    // --- TAMBAHAN FEEDER AKTIVITAS MENGAJAR ---
    pub id_aktivitas_mengajar_feeder: Option<Uuid>,
    pub sks_substansi_total: Option<i32>,
    pub rencana_tatap_muka: Option<i32>,
    pub realisasi_tatap_muka: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct JadwalKuliahDetail {
    pub id: Uuid,
    pub kelas: String,

    // --- TAMBAHAN FEEDER KELAS KULIAH ---
    pub id_kelas_kuliah_feeder: Option<Uuid>,
    pub nama_kelas_kuliah: Option<String>,

    pub hari: DayOfWeek,
    pub jam_mulai: TimeWithOffset,
    pub jam_selesai: TimeWithOffset,
    pub matakuliah_id: Uuid,
    pub nama_mk: String,
    pub kode_mk: String,
    pub sks: i32,
    pub prodi_id: Uuid,
    pub nama_prodi: String,
    pub tahun_akademik_id: Uuid,
    pub nama_tahun_akademik: String,
    pub dosen_pengampu: Vec<DosenPengampuDetail>,
    pub ruangan_id: Option<Uuid>,
    pub nama_ruangan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JadwalKuliahFilter {
    pub tahun_akademik_id: Option<Uuid>,
    pub prodi_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJadwalKuliahPayload {
    pub matakuliah_id: Uuid,
    pub tahun_akademik_id: Uuid,
    pub hari: DayOfWeek,
    pub jam_mulai: TimeWithOffset,
    pub jam_selesai: TimeWithOffset,
    pub kelas: String,

    // --- TAMBAHAN FEEDER KELAS KULIAH ---
    pub id_kelas_kuliah_feeder: Option<Uuid>,
    pub nama_kelas_kuliah: Option<String>,

    pub dosen_pengampu: Vec<DosenPengampuPayload>,
}
