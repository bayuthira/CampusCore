-- migrations/YYYY..._add_kondisi_to_aset.sql

-- 1. Buat tipe data ENUM kustom baru untuk kondisi aset
CREATE TYPE "KondisiAset" AS ENUM (
    'Baik',
    'Rusak Ringan',
    'Rusak Berat',
    'Dalam Perbaikan',
    'Dihapuskan'
);

-- 2. Tambahkan kolom baru 'kondisi' ke tabel aset
--    Kita beri nilai default 'Baik' agar data aset yang sudah ada tidak error.
ALTER TABLE aset
ADD COLUMN kondisi "KondisiAset" NOT NULL DEFAULT 'Baik';