// src/modules/sdm/print_model.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct PersonelPrint {
    pub nama: String,
    pub jabatan: String,
    pub unit: String,
}

#[derive(Serialize)]
pub struct SppdTemplateContext {
    pub nomor_sppd: String,
    pub personel: Vec<PersonelPrint>,
    pub alasan: String,
    pub tujuan_kota: String,
    pub alamat: String,
    pub nama_kegiatan: String,
    pub tgl_berangkat: String,
    pub tgl_kembali: String,
    pub penandatangan_tempat: String,
    pub penandatangan_tanggal: String,
    pub penandatangan_jabatan: String,
    pub penandatangan_nama: String,
    pub penandatangan_nik: String,
    pub empty_row_count: usize,
    pub tembusan: Vec<String>,
}
