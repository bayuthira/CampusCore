-- 1. Tambah ID Feeder
ALTER TABLE mata_kuliah ADD COLUMN id_matkul_feeder UUID UNIQUE;

-- 2. Tambah pecahan SKS (Default 0 agar data lama tidak error)
ALTER TABLE mata_kuliah ADD COLUMN sks_tatap_muka INTEGER NOT NULL DEFAULT 0;
ALTER TABLE mata_kuliah ADD COLUMN sks_praktek INTEGER NOT NULL DEFAULT 0;
ALTER TABLE mata_kuliah ADD COLUMN sks_praktek_lapangan INTEGER NOT NULL DEFAULT 0;
ALTER TABLE mata_kuliah ADD COLUMN sks_simulasi INTEGER NOT NULL DEFAULT 0;

-- 3. Tambah Jenis Mata Kuliah
ALTER TABLE mata_kuliah ADD COLUMN jenis_mk VARCHAR(50) NOT NULL DEFAULT 'Wajib';

-- 4. Amankan Data Lama: 
-- Pindahkan nilai dari kolom 'sks' yang lama ke 'sks_tatap_muka' 
-- (Asumsi data SKS lama adalah tatap muka semua)
UPDATE mata_kuliah SET sks_tatap_muka = sks;