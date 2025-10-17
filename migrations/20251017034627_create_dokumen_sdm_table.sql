-- migrations/YYYY..._create_dokumen_sdm_table.sql

-- Tipe ENUM untuk jenis entitas induk
CREATE TYPE "SdmEntityType" AS ENUM (
    'Pegawai',
    'RiwayatPendidikan',
    'RiwayatSk'
);

-- Tipe ENUM untuk kategori dokumen
CREATE TYPE "KategoriDokumen" AS ENUM (
    'FotoProfil',
    'KTP',
    'KK',
    'Ijazah',
    'Transkrip',
    'SK',
    'Sertifikat',
    'Lainnya'
);

-- Tabel pusat untuk semua dokumen SDM
CREATE TABLE dokumen_sdm (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE CASCADE,

    -- Kolom untuk polymorphic association
    entity_id UUID NOT NULL,
    entity_type "SdmEntityType" NOT NULL,

    kategori "KategoriDokumen" NOT NULL,
    nama_file_asli VARCHAR(255) NOT NULL,
    path_file VARCHAR(255) NOT NULL UNIQUE,
    tipe_mime VARCHAR(100),

    user_uploader_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index untuk pencarian cepat
CREATE INDEX idx_dokumen_sdm_entity ON dokumen_sdm(entity_id, entity_type);-- Add migration script here
