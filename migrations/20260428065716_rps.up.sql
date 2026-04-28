-- =========================================================================
-- 1. TABEL HEADER RPS (Relasi 1-to-1 dengan mata_kuliah)
-- =========================================================================
CREATE TABLE mata_kuliah_rps (
    mata_kuliah_id UUID PRIMARY KEY REFERENCES mata_kuliah(id) ON DELETE CASCADE,
    
    deskripsi_singkat TEXT,
    capaian_pembelajaran TEXT, -- Bisa menyimpan format JSON atau Text panjang berisi CPL dan CPMK
    pustaka_utama TEXT,
    pustaka_pendukung TEXT,
    matakuliah_syarat TEXT,
    
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

-- =========================================================================
-- 2. TABEL MATRIKS MINGGUAN RPS (Relasi 1-to-Many)
-- =========================================================================
-- Tabel ini yang akan disandingkan (cross-check) dengan Jurnal Mengajar / BAP
CREATE TABLE mata_kuliah_rps_mingguan (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mata_kuliah_id UUID NOT NULL REFERENCES mata_kuliah(id) ON DELETE CASCADE,
    
    minggu_ke INT NOT NULL CHECK (minggu_ke BETWEEN 1 AND 16),
    
    -- Kemampuan Akhir (Sub-CPMK)
    kemampuan_akhir_diharapkan TEXT, 
    
    -- Bahan Kajian (Ini yang akan ditarik otomatis saat Dosen buka Jurnal BAP)
    bahan_kajian TEXT, 
    
    -- Metode Pembelajaran (Misal: Project Based Learning, Diskusi Kelompok)
    metode_pembelajaran TEXT, 
    
    waktu_belajar TEXT,
    kriteria_penilaian TEXT,
    
    bobot_penilaian NUMERIC(5,2) DEFAULT 0.00,
    
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    
    -- Mencegah duplikasi minggu pada mata kuliah yang sama
    UNIQUE(mata_kuliah_id, minggu_ke) 
);