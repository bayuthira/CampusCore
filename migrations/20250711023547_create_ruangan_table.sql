-- migrations/YYYYMMDDHHMMSS_create_ruangan_table.sql
CREATE TABLE ruangan (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Kode unik untuk ruangan, contoh: 'GD501', 'LAB-C'
    kode_ruangan VARCHAR(50) NOT NULL UNIQUE,

    nama_ruangan VARCHAR(255) NOT NULL, -- Contoh: 'Ruang Teori 501', 'Laboratorium Jaringan'

    kapasitas INT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ruangan_kode ON ruangan(kode_ruangan);