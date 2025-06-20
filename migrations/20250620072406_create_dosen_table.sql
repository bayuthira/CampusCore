-- migrations/YYYYMMDDHHMMSS_create_dosen_table.sql
CREATE TABLE dosen (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Nomor Induk Dosen Nasional, harus unik
    nidn VARCHAR(20) NOT NULL UNIQUE,

    nama_dosen VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE,

    -- Ini adalah Foreign Key
    -- Menandakan bahwa setiap dosen HARUS terhubung ke sebuah prodi yang valid
    prodi_id UUID NOT NULL REFERENCES prodi(id) ON DELETE RESTRICT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Membuat index pada foreign key untuk mempercepat query JOIN
CREATE INDEX idx_dosen_prodi_id ON dosen(prodi_id);-- Add migration script here
