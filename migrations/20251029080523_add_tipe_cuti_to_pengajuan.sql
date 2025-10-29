-- Tipe ENUM baru untuk jenis cuti
CREATE TYPE "TipeCuti" AS ENUM (
    'Paid',
    'Unpaid'
);

-- Tambahkan kolom baru ke tabel pengajuan_cuti
ALTER TABLE pengajuan_cuti
ADD COLUMN tipe_cuti "TipeCuti" NOT NULL DEFAULT 'Paid';