use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AsesmenQuery {
    pub tahun_akademik_id: Uuid,
}

#[derive(Debug, Serialize, FromRow)]
pub struct JadwalAsesmenOption {
    pub id: Uuid,
    pub kode_mk: String,
    pub nama_mk: String,
    pub kelas: String,
    pub nama_prodi: String,
    pub can_create: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpsertAsesmenPayload {
    pub jadwal_kuliah_id: Uuid,
    pub jenis: String,
    pub judul: String,
    pub mode: String,
    pub bobot: Decimal,
    pub durasi_menit: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub mulai_terjadwal: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub selesai_terjadwal: OffsetDateTime,
    pub online_url: Option<String>,
    pub instruksi: Option<String>,
    pub sifat_ujian: Option<String>,
    pub hitung_sebagai_pertemuan: bool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AsesmenListRow {
    pub id: Uuid,
    pub jadwal_kuliah_id: Uuid,
    pub kode_mk: String,
    pub nama_mk: String,
    pub kelas: String,
    pub nama_prodi: String,
    pub jenis: String,
    pub judul: String,
    pub mode: String,
    pub bobot: Decimal,
    pub durasi_menit: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub mulai_terjadwal: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub selesai_terjadwal: OffsetDateTime,
    pub status: String,
    pub jumlah_dokumen: i64,
    pub status_penggandaan: Option<String>,
    pub can_edit: bool,
    pub can_review: bool,
    pub can_production: bool,
    pub can_execute: bool,
    pub can_grade: bool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AsesmenRecord {
    pub id: Uuid,
    pub jadwal_kuliah_id: Uuid,
    pub jenis: String,
    pub judul: String,
    pub mode: String,
    pub bobot: Decimal,
    pub durasi_menit: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub mulai_terjadwal: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub selesai_terjadwal: OffsetDateTime,
    pub online_url: Option<String>,
    pub instruksi: Option<String>,
    pub sifat_ujian: Option<String>,
    pub hitung_sebagai_pertemuan: bool,
    pub status: String,
    pub catatan_review: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DokumenAsesmen {
    pub id: Uuid,
    pub jenis: String,
    pub versi: i32,
    pub nama_file_asli: String,
    pub mime_type: String,
    pub ukuran_bytes: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ReviewAsesmen {
    pub aksi: String,
    pub catatan: Option<String>,
    pub reviewer: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PenggandaanAsesmen {
    pub jumlah_utama: i32,
    pub jumlah_cadangan: i32,
    pub status: String,
    pub catatan: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PelaksanaanAsesmen {
    #[serde(with = "time::serde::rfc3339")]
    pub mulai_aktual: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub selesai_aktual: Option<OffsetDateTime>,
    pub pengawas: String,
    pub versi_soal: Option<String>,
    pub jumlah_lembar_diterima: Option<i32>,
    pub bap: Option<String>,
    pub insiden: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RosterAsesmen {
    pub enrollment_id: Uuid,
    pub nim: String,
    pub nama_mahasiswa: String,
    pub status_presensi: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub check_in_at: Option<OffsetDateTime>,
    pub sumber: Option<String>,
    pub nilai: Option<Decimal>,
    pub attempt: Option<i32>,
    pub umpan_balik: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AsesmenDetailResponse {
    pub asesmen: AsesmenRecord,
    pub dokumen: Vec<DokumenAsesmen>,
    pub review: Vec<ReviewAsesmen>,
    pub penggandaan: Option<PenggandaanAsesmen>,
    pub pelaksanaan: Option<PelaksanaanAsesmen>,
    pub roster: Vec<RosterAsesmen>,
    pub sesi_presensi: Option<SesiAsesmenResponse>,
    pub jumlah_peserta: i64,
}

#[derive(Debug, Deserialize)]
pub struct ReviewPayload {
    pub aksi: String,
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PenggandaanPayload {
    pub jumlah_utama: i32,
    pub jumlah_cadangan: i32,
    pub status: String,
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FinishAsesmenPayload {
    pub versi_soal: Option<String>,
    pub jumlah_lembar_diterima: Option<i32>,
    pub bap: String,
    pub insiden: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PresensiAsesmenPayload {
    pub status: String,
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NilaiAsesmenPayload {
    pub attempt: Option<i32>,
    pub nilai: Decimal,
    pub umpan_balik: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckInAsesmenPayload {
    pub kode: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SesiAsesmenResponse {
    pub kode: String,
    #[serde(with = "time::serde::rfc3339")]
    pub berlaku_sampai: OffsetDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AsesmenMahasiswaRow {
    pub id: Uuid,
    pub kode_mk: String,
    pub nama_mk: String,
    pub kelas: String,
    pub jenis: String,
    pub judul: String,
    pub mode: String,
    #[serde(with = "time::serde::rfc3339")]
    pub mulai_terjadwal: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub selesai_terjadwal: OffsetDateTime,
    pub durasi_menit: i32,
    pub instruksi: Option<String>,
    pub status: String,
    pub online_url: Option<String>,
    pub status_presensi: Option<String>,
    pub nilai: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct KelasNilaiAkhir {
    pub jadwal_kuliah_id: Uuid,
    pub prodi_id: Uuid,
    pub kode_mk: String,
    pub nama_mk: String,
    pub kelas: String,
    pub nama_prodi: String,
    pub status: String,
    pub total_bobot: Decimal,
    pub jumlah_asesmen: i64,
    pub jumlah_asesmen_dikunci: i64,
    pub can_submit: bool,
    pub can_review: bool,
    pub can_publish: bool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct KomponenNilaiAkhir {
    pub id: Uuid,
    pub jenis: String,
    pub judul: String,
    pub bobot: Decimal,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct NilaiKomponenMahasiswa {
    pub asesmen_id: Uuid,
    pub nilai: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct MahasiswaNilaiAkhir {
    pub enrollment_id: Uuid,
    pub nim: String,
    pub nama_mahasiswa: String,
    pub komponen: Vec<NilaiKomponenMahasiswa>,
    pub lengkap: bool,
    pub nilai_akhir: Option<Decimal>,
    pub nilai_huruf: Option<String>,
    pub nilai_indeks: Option<Decimal>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RiwayatNilaiAkhir {
    pub aksi: String,
    pub catatan: Option<String>,
    pub dilakukan_oleh: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct NilaiAkhirDetail {
    pub kelas: KelasNilaiAkhir,
    pub komponen: Vec<KomponenNilaiAkhir>,
    pub mahasiswa: Vec<MahasiswaNilaiAkhir>,
    pub riwayat: Vec<RiwayatNilaiAkhir>,
    pub skala_tersedia: bool,
}

#[derive(Debug, Deserialize)]
pub struct ReviewNilaiAkhirPayload {
    pub aksi: String,
    pub catatan: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SkalaNilaiRow {
    pub id: Uuid,
    pub prodi_id: Option<Uuid>,
    pub scope: String,
    pub nilai_huruf: String,
    pub nilai_indeks: Decimal,
    pub bobot_minimum: Decimal,
    pub bobot_maksimum: Decimal,
    pub tanggal_mulai_efektif: String,
    pub tanggal_akhir_efektif: Option<String>,
    pub dari_feeder: bool,
    pub is_locked: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSkalaNilaiItem {
    pub id: Option<Uuid>,
    pub nilai_huruf: String,
    pub nilai_indeks: Decimal,
    pub bobot_minimum: Decimal,
    pub bobot_maksimum: Decimal,
    pub tanggal_mulai_efektif: String,
    pub tanggal_akhir_efektif: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSkalaNilaiPayload {
    pub items: Vec<UpsertSkalaNilaiItem>,
}

#[derive(Debug, Serialize)]
pub struct KomponenNilaiMahasiswa {
    pub asesmen_id: Uuid,
    pub jenis: String,
    pub judul: String,
    pub bobot: Decimal,
    pub nilai: Decimal,
    pub kontribusi: Decimal,
}

#[derive(Debug, Serialize)]
pub struct NilaiMataKuliahMahasiswa {
    pub enrollment_id: Uuid,
    pub kode_mk: String,
    pub nama_mk: String,
    pub kelas: String,
    pub nama_prodi: String,
    pub sks: i32,
    pub nilai_angka: Option<Decimal>,
    pub nilai_huruf: Option<String>,
    pub nilai_indeks: Option<Decimal>,
    pub komponen: Vec<KomponenNilaiMahasiswa>,
}
