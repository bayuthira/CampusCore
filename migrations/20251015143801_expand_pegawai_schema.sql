-- migrations/YYYY..._expand_pegawai_schema.sql

-- Menambahkan kolom-kolom baru ke tabel `pegawai` yang sudah ada
ALTER TABLE pegawai
    ADD COLUMN no_kk VARCHAR(50),
    ADD COLUMN no_npwp VARCHAR(50),
    ADD COLUMN no_bpjs_kesehatan VARCHAR(50),
    ADD COLUMN no_bpjs_ketenagakerjaan VARCHAR(50),
    ADD COLUMN kota VARCHAR(100),
    ADD COLUMN kode_pos VARCHAR(10),
    ADD COLUMN gol_darah VARCHAR(5),
    ADD COLUMN tanggal_pensiun DATE,
    ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT true;

-- Tabel baru untuk Riwayat SK & Jabatan (One-to-Many)
-- Ini dipisah karena satu pegawai bisa punya banyak SK
CREATE TABLE riwayat_sk (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE CASCADE,
    nomor_sk VARCHAR(255) NOT NULL,
    tanggal_sk DATE NOT NULL,
    jenis_sk VARCHAR(100) NOT NULL, -- Contoh: 'Pengangkatan', 'Jabatan Fungsional'
    jabatan VARCHAR(255),
    keterangan TEXT
);