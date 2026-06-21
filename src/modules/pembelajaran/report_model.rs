use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::Date;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ReportPembelajaranQuery {
    pub tahun_akademik_id: Uuid,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ReportKelasRow {
    pub jadwal_kuliah_id: Uuid,
    pub kode_mk: String,
    pub nama_mk: String,
    pub kelas: String,
    pub nama_prodi: String,
    pub tahun_akademik: String,
    pub dosen_pengampu: String,
    pub status_rps: String,
    pub target_pertemuan: i64,
    pub jumlah_pertemuan: i64,
    pub pertemuan_ditutup: i64,
    pub bap_lengkap: i64,
    pub presensi_dosen: i64,
    pub jumlah_mahasiswa: i64,
    pub mahasiswa_hadir: i64,
    pub total_slot_presensi: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ReportPertemuanRow {
    pub id: Uuid,
    pub pertemuan_ke: i32,
    pub tanggal: Date,
    pub status: String,
    pub topik_rencana: Option<String>,
    pub topik_realisasi: Option<String>,
    pub bap_lengkap: bool,
    pub hadir: i64,
    pub terlambat: i64,
    pub izin: i64,
    pub sakit: i64,
    pub alpa: i64,
}
