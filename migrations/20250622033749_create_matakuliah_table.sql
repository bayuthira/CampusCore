-- migrations/YYYYMMDDHHMMSS_create_matakuliah_table.sql
CREATE TABLE mata_kuliah (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Kode Mata Kuliah, harus unik
    kode_mk VARCHAR(20) NOT NULL UNIQUE,

    nama_mk VARCHAR(255) NOT NULL,

    -- Satuan Kredit Semester
    sks INT NOT NULL,

    -- Semester di mana MK ini idealnya diambil (misal: 1, 2, 3, ...)
    semester_target INT NOT NULL,

    -- Foreign Key ke tabel Prodi
    prodi_id UUID NOT NULL REFERENCES prodi(id) ON DELETE RESTRICT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index untuk mempercepat pencarian berdasarkan kode dan prodi
CREATE INDEX idx_matakuliah_kode_mk ON mata_kuliah(kode_mk);
CREATE INDEX idx_matakuliah_prodi_id ON mata_kuliah(prodi_id);-- Add migration script here
