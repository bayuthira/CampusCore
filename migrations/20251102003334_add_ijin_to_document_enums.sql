-- Tambahkan entitas baru untuk Ijin
ALTER TYPE "SdmEntityType" ADD VALUE 'PengajuanIjin';

-- Tambahkan kategori dokumen baru
ALTER TYPE "KategoriDokumen" ADD VALUE 'SuratSakit';
ALTER TYPE "KategoriDokumen" ADD VALUE 'DokumenPendukung';