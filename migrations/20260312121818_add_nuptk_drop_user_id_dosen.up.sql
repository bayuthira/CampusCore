-- 1. Tambah NUPTK di tabel Pegawai (Berlaku untuk Dosen dan Staf)
ALTER TABLE pegawai ADD COLUMN nuptk VARCHAR(50);

-- 2. Hapus kolom user_id dari tabel Dosen karena redundan
ALTER TABLE dosen DROP CONSTRAINT dosen_user_id_fkey;
ALTER TABLE dosen DROP COLUMN user_id;

-- 3. Jadikan kolom NIDN opsional (Boleh NULL) mengikuti regulasi terbaru
ALTER TABLE dosen ALTER COLUMN nidn DROP NOT NULL;