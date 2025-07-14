-- migrations/YYYY..._create_jenis_aset_table.sql
CREATE TABLE jenis_aset (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nama_jenis VARCHAR(255) NOT NULL UNIQUE, -- Contoh: 'Mebel', 'Elektronik', 'Peralatan Medis'
    deskripsi TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);-- Add migration script here
