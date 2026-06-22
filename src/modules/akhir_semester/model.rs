use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct TahunAkademikQuery {
    pub tahun_akademik_id: Uuid,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StatusAkhirSemester {
    pub tahun_akademik_id: Uuid,
    pub nama: String,
    pub status_penutupan: String,
    pub jumlah_kelas: i64,
    pub kelas_siap: i64,
    pub kelas_belum_siap: i64,
    pub jumlah_mahasiswa: i64,
    pub jumlah_nilai: i64,
    pub ditutup_oleh: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub ditutup_pada: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct KhsMataKuliah {
    pub kode_mk: String,
    pub nama_mk: String,
    pub sks: i32,
    pub nilai_angka: Decimal,
    pub nilai_huruf: String,
    pub nilai_indeks: Decimal,
    pub mutu: Decimal,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RingkasanAkademik {
    pub ips: Decimal,
    pub ipk: Decimal,
    pub sks_semester: i32,
    pub sks_total: i32,
    pub status_mahasiswa: String,
}

#[derive(Debug, Serialize)]
pub struct KhsResponse {
    pub tahun_akademik: String,
    pub nim: String,
    pub nama_mahasiswa: String,
    pub nama_prodi: String,
    pub status_penutupan: String,
    pub ringkasan: Option<RingkasanAkademik>,
    pub mata_kuliah: Vec<KhsMataKuliah>,
}

#[derive(Debug, Serialize)]
pub struct TranskripResponse {
    pub nim: String,
    pub nama_mahasiswa: String,
    pub nama_prodi: String,
    pub ipk: Decimal,
    pub total_sks: i32,
    pub mata_kuliah: Vec<KhsMataKuliah>,
}

#[derive(Debug, Deserialize)]
pub struct AjukanKoreksiNilaiPayload {
    pub enrollment_id: Uuid,
    pub nilai_angka_baru: Decimal,
    pub alasan: String,
}

#[derive(Debug, Deserialize)]
pub struct ReviewKoreksiPayload {
    pub aksi: String,
    pub catatan: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct KoreksiNilaiRow {
    pub id: Uuid,
    pub enrollment_id: Uuid,
    pub nim: String,
    pub nama_mahasiswa: String,
    pub kode_mk: String,
    pub nama_mk: String,
    pub nilai_angka_lama: Option<Decimal>,
    pub nilai_huruf_lama: Option<String>,
    pub nilai_angka_baru: Decimal,
    pub nilai_huruf_baru: String,
    pub alasan: String,
    pub status: String,
    pub diajukan_oleh: String,
    pub catatan_review: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FeederOutboxRow {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub operation: String,
    pub payload: Value,
    pub status: String,
    pub attempts: i32,
    pub last_error: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub synced_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct FeederResultPayload {
    pub berhasil: bool,
    pub feeder_id: Option<Uuid>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}
