-- =========================================================================
-- 1. ALTER TABEL MATA KULIAH (UNTUK RPS)
-- =========================================================================
-- Menambahkan kolom untuk menyimpan path file RPS (PDF/Word)
ALTER TABLE mata_kuliah ADD COLUMN file_rps_path VARCHAR(255);

-- =========================================================================
-- 2. CREATE TABEL RENCANA PENILAIAN & KONTRAK KULIAH (PER KELAS/JADWAL)
-- =========================================================================
CREATE TABLE jadwal_rencana_penilaian (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Relasi 1-to-1 dengan jadwal_kuliah. Satu jadwal/kelas hanya punya 1 rencana penilaian
    jadwal_kuliah_id UUID NOT NULL REFERENCES jadwal_kuliah(id) ON DELETE CASCADE UNIQUE,
    
    -- File Kontrak Kuliah
    file_kontrak_path VARCHAR(255),
    
    -- Komponen Bobot Penilaian (Skala 0.00 - 100.00)
    bobot_kehadiran NUMERIC(5,2) NOT NULL DEFAULT 0.00,
    bobot_tugas NUMERIC(5,2) NOT NULL DEFAULT 0.00,
    bobot_uts NUMERIC(5,2) NOT NULL DEFAULT 0.00,
    bobot_uas NUMERIC(5,2) NOT NULL DEFAULT 0.00,
    bobot_praktek NUMERIC(5,2) NOT NULL DEFAULT 0.00,
    
    -- Rencana Praktikum (Hanya diwajibkan jika SKS Praktikum > 0)
    catatan_rencana_praktikum TEXT,
    file_praktikum_path VARCHAR(255),
    
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);