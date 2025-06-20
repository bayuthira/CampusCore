-- migrations/YYYYMMDDHHMMSS_create_prodi_table.sql
CREATE TABLE prodi (
    -- Menggunakan UUID sebagai primary key adalah praktik yang baik untuk sistem terdistribusi
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Kode unik untuk setiap prodi, misal: 'IF', 'EL', 'TI'
    kode_prodi VARCHAR(10) NOT NULL UNIQUE,
    
    nama_prodi VARCHAR(255) NOT NULL,
    
    -- Timestamp otomatis untuk melacak kapan data dibuat dan diperbarui
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);-- Add migration script here
