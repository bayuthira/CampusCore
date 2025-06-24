-- Add migration script here
-- migrations/YYYYMMDDHHMMSS_create_enrollments_table.sql
-- (GANTI ISI FILE DENGAN INI)

-- Tabel Master untuk Tahun Akademik / Periode
CREATE TABLE tahun_akademik (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Nama periode yang unik, contoh: 'Ganjil 2025/2026'
    nama VARCHAR(50) NOT NULL UNIQUE,
    
    tanggal_mulai DATE NOT NULL,
    tanggal_selesai DATE NOT NULL,
    
    -- Periode pengisian KRS untuk periode ini
    krs_mulai DATE NOT NULL,
    krs_selesai DATE NOT NULL,
    
    -- Status untuk menandai mana periode yang sedang aktif
    is_active BOOLEAN NOT NULL DEFAULT false,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Constraint canggih: Memastikan hanya boleh ada SATU tahun akademik yang aktif.
-- Ini adalah fitur PostgreSQL yang sangat berguna.
CREATE UNIQUE INDEX only_one_active_tahun_akademik ON tahun_akademik (is_active) WHERE is_active IS TRUE;


-- Membuat tipe data kustom untuk status approval KRS (tidak berubah)
CREATE TYPE "EnrollmentStatus" AS ENUM (
    'Menunggu Persetujuan',
    'Disetujui',
    'Ditolak',
    'Selesai',
    'Mengulang'
);

-- Tabel jembatan 'enrollments' yang sudah diperbarui
CREATE TABLE enrollments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mahasiswa_id UUID NOT NULL REFERENCES mahasiswa(id) ON DELETE CASCADE,
    matakuliah_id UUID NOT NULL REFERENCES mata_kuliah(id) ON DELETE RESTRICT,
    
    -- Foreign Key ke tabel master tahun_akademik
    tahun_akademik_id UUID NOT NULL REFERENCES tahun_akademik(id) ON DELETE RESTRICT,
    
    status_approval "EnrollmentStatus" NOT NULL DEFAULT 'Menunggu Persetujuan',
    nilai_huruf VARCHAR(2),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Kunci Unik: Seorang mahasiswa tidak bisa mengambil mata kuliah yang sama di periode yang sama.
    UNIQUE (mahasiswa_id, matakuliah_id, tahun_akademik_id)
);

-- Index-index yang diperlukan (tidak berubah)
CREATE INDEX idx_enrollments_mahasiswa_id ON enrollments(mahasiswa_id);
CREATE INDEX idx_enrollments_matakuliah_id ON enrollments(matakuliah_id);