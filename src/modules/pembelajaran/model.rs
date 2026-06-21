use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct KelasPembelajaran {
    pub jadwal_kuliah_id: Uuid,
    pub mata_kuliah_id: Uuid,
    pub kode_mk: String,
    pub nama_mk: String,
    pub kelas: String,
    pub nama_tahun_akademik: String,
    pub hari: String,
    pub jam_mulai: String,
    pub jam_selesai: String,
    pub nama_ruangan: Option<String>,
    pub status_rps: String,
    pub pembelajaran_aktif: bool,
    pub jumlah_pertemuan: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PertemuanKuliah {
    pub id: Uuid,
    pub jadwal_kuliah_id: Uuid,
    pub rps_mingguan_id: Option<Uuid>,
    pub pertemuan_ke: i32,
    pub tanggal: Date,
    pub topik_rencana: Option<String>,
    pub topik_realisasi: Option<String>,
    pub metode_pembelajaran: Option<String>,
    pub bap: Option<String>,
    pub status: String,
    pub dibuka_pada: Option<OffsetDateTime>,
    pub ditutup_pada: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePertemuanPayload {
    pub pertemuan_ke: i32,
    pub tanggal: Date,
    pub rps_mingguan_id: Option<Uuid>,
    pub topik_rencana: Option<String>,
    pub metode_pembelajaran: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBapPayload {
    pub topik_realisasi: Option<String>,
    pub metode_pembelajaran: Option<String>,
    pub bap: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SesiPresensiResponse {
    pub kode: String,
    pub berlaku_sampai: OffsetDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PresensiMahasiswaRow {
    pub enrollment_id: Uuid,
    pub nim: String,
    pub nama_mahasiswa: String,
    pub status: String,
    pub check_in_at: Option<OffsetDateTime>,
    pub sumber: Option<String>,
    pub catatan: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DetailPertemuanResponse {
    pub pertemuan: PertemuanKuliah,
    pub presensi_mahasiswa: Vec<PresensiMahasiswaRow>,
}

#[derive(Debug, Deserialize)]
pub struct ManualPresensiPayload {
    pub status: String,
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckInMahasiswaPayload {
    pub kode: String,
}

#[derive(Debug, Serialize)]
pub struct SuccessMessage {
    pub message: String,
}
