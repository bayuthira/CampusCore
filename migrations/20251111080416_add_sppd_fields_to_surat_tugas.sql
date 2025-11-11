-- migrations/YYYY..._add_sppd_fields_to_surat_tugas.sql

-- 1. Tambahkan kolom-kolom SPPD ke tabel master
-- Kolom-kolom ini bersifat opsional (NULLABLE)
ALTER TABLE surat_tugas_master
    ADD COLUMN nomor_sppd VARCHAR(255) UNIQUE, -- Nomor SPPD bisa jadi beda format
    ADD COLUMN alat_angkut VARCHAR(255),
    ADD COLUMN tempat_berangkat VARCHAR(255),
    ADD COLUMN lama_perjalanan INT,

    -- Info Anggaran
    ADD COLUMN pembebanan_anggaran_instansi VARCHAR(255),
    ADD COLUMN pembebanan_anggaran_mak VARCHAR(255),

    -- Pejabat
    ADD COLUMN ppk_pegawai_id UUID REFERENCES pegawai(id) ON DELETE RESTRICT,
    ADD COLUMN kpa_pegawai_id UUID REFERENCES pegawai(id) ON DELETE RESTRICT,

    ADD COLUMN keterangan_lain TEXT;

-- 2. Modifikasi tabel penerima untuk mendukung "Pelaksana Utama"
-- Kita tambahkan satu kolom ENUM
CREATE TYPE "PeranPerjalanan" AS ENUM ('Pelaksana Utama', 'Pengikut');

ALTER TABLE surat_tugas_penerima
    ADD COLUMN peran "PeranPerjalanan" NOT NULL DEFAULT 'Pengikut';-- Add migration script here
