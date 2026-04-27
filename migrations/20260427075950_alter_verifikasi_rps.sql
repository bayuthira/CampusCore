-- Menambahkan kolom status verifikasi RPS
-- Default 'Belum Upload'. Status lain: 'Menunggu Verifikasi', 'Disetujui', 'Ditolak'
ALTER TABLE mata_kuliah ADD COLUMN status_verifikasi_rps VARCHAR(50) DEFAULT 'Belum Upload';

-- Menambahkan kolom catatan jika RPS ditolak atau butuh revisi
ALTER TABLE mata_kuliah ADD COLUMN catatan_verifikasi_rps TEXT;