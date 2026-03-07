-- 1. Tambah ID Feeder
ALTER TABLE kurikulum ADD COLUMN id_kurikulum_feeder UUID UNIQUE;

-- 2. Tambah Persyaratan SKS Lulus (Default 144 untuk S1)
ALTER TABLE kurikulum ADD COLUMN sks_lulus INTEGER NOT NULL DEFAULT 144;
ALTER TABLE kurikulum ADD COLUMN sks_wajib INTEGER NOT NULL DEFAULT 0;
ALTER TABLE kurikulum ADD COLUMN sks_pilihan INTEGER NOT NULL DEFAULT 0;

-- 3. Tambah ID Semester Mulai (Baku Feeder, misal '20231')
ALTER TABLE kurikulum ADD COLUMN id_semester_mulai VARCHAR(5);