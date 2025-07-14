-- migrations/YYYYMMDDHHMMSS_add_kelompok_to_jenis_aset.sql

-- 1. Buat tipe data ENUM kustom baru untuk kelompok aset
CREATE TYPE "KelompokAset" AS ENUM (
    'Sarana',
    'Prasarana'
);

-- 2. Tambahkan kolom baru 'kelompok' ke tabel jenis_aset
--    Kita buat NOT NULL dan beri nilai default 'Sarana' agar data lama tidak error.
ALTER TABLE jenis_aset
ADD COLUMN kelompok "KelompokAset" NOT NULL DEFAULT 'Sarana';

-- 3. (Opsional) Buat index untuk mempercepat filter berdasarkan kelompok
CREATE INDEX idx_jenis_aset_kelompok ON jenis_aset(kelompok);