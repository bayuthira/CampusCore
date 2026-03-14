-- Menambahkan kolom alasan_perjalanan (1: Kunjungan, 2: Tugas Lembaga, 3: Pelatihan)
ALTER TABLE surat_tugas_master ADD COLUMN alasan_perjalanan INTEGER;

-- Menambahkan kolom tujuan_kota
ALTER TABLE surat_tugas_master ADD COLUMN tujuan_kota VARCHAR(255);