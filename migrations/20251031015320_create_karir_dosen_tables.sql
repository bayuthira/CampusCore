-- migrations/YYYY..._create_karir_dosen_tables.sql

-- --- 1. JABATAN AKADEMIK DOSEN (JAD) ---

-- ENUM untuk Jabatan Akademik
CREATE TYPE "JabatanAkademik" AS ENUM (
    'Asisten Ahli',
    'Lektor',
    'Lektor Kepala',
    'Guru Besar'
);

-- ENUM untuk Pangkat/Golongan
CREATE TYPE "PangkatGolongan" AS ENUM (
    'III/a', 'III/b', -- Asisten Ahli
    'III/c', 'III/d', -- Lektor
    'IV/a', 'IV/b', 'IV/c', -- Lektor Kepala
    'IV/d', 'IV/e'  -- Guru Besar
);

-- Tabel untuk Riwayat JAD
CREATE TABLE riwayat_jad (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE CASCADE,

    jabatan_akademik "JabatanAkademik" NOT NULL,
    pangkat_golongan "PangkatGolongan" NOT NULL,
    nomor_sk VARCHAR(255) NOT NULL,
    tmt DATE NOT NULL, -- Terhitung Mulai Tanggal
    kompetensi_mk TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- --- 2. SERTIFIKASI DOSEN (SERDOS) ---

-- Tabel untuk Riwayat SERDOS
CREATE TABLE riwayat_serdos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE CASCADE,

    nomor_sertifikat VARCHAR(255) NOT NULL,
    tanggal_terbit DATE NOT NULL,
    keterangan TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index untuk pencarian cepat
CREATE INDEX idx_riwayat_jad_pegawai_id ON riwayat_jad(pegawai_id);
CREATE INDEX idx_riwayat_serdos_pegawai_id ON riwayat_serdos(pegawai_id);-- Add migration script here
