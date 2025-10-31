-- migrations/YYYY..._create_ijin_tables.sql

-- ENUM untuk kategori ijin
CREATE TYPE "KategoriIjin" AS ENUM (
    'Sakit',
    'Urusan Keluarga',
    'Lainnya' -- Untuk mencakup "dll"
);

-- ENUM untuk status ijin (mirip dengan cuti)
CREATE TYPE "StatusIjin" AS ENUM (
    'Diajukan',
    'Disetujui',
    'Ditolak'
);

-- Tabel untuk mencatat setiap pengajuan ijin
CREATE TABLE pengajuan_ijin (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE RESTRICT,

    kategori "KategoriIjin" NOT NULL,
    tanggal_mulai DATE NOT NULL,
    tanggal_selesai DATE NOT NULL,
    alasan TEXT NOT NULL,

    status "StatusIjin" NOT NULL DEFAULT 'Diajukan',

    -- Diisi oleh atasan/admin saat approval
    user_approve_id UUID REFERENCES users(id) ON DELETE SET NULL,
    catatan_approval TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_pengajuan_ijin_pegawai_id ON pengajuan_ijin(pegawai_id);
CREATE INDEX idx_pengajuan_ijin_status ON pengajuan_ijin(status);-- Add migration script here
