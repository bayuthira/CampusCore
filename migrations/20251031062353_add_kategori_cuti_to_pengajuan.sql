-- migrations/YYYY..._add_kategori_cuti_to_pengajuan.sql

-- ENUM baru untuk kategori/jenis pengajuan
CREATE TYPE "KategoriCuti" AS ENUM (
    'Cuti Tahunan',
    'Cuti Melahirkan',
    'Cuti Sakit Berkepanjangan',
    'Cuti Hajatan Keluarga',
    'Cuti Ibadah',
    'Lainnya'
);

-- Tambahkan kolom baru ke tabel pengajuan_cuti
ALTER TABLE pengajuan_cuti
ADD COLUMN kategori "KategoriCuti" NOT NULL DEFAULT 'Cuti Tahunan';-- Add migration script here
