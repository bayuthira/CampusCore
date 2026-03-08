-- ==========================================
-- 1. UPDATE TABEL PEGAWAI (BIODATA & FEEDER)
-- ==========================================
ALTER TABLE pegawai ADD COLUMN id_sdm_feeder UUID UNIQUE;
ALTER TABLE pegawai ADD COLUMN nama_ibu_kandung VARCHAR(255);
ALTER TABLE pegawai ADD COLUMN kewarganegaraan VARCHAR(2) DEFAULT 'ID';
ALTER TABLE pegawai ADD COLUMN dusun VARCHAR(100);
ALTER TABLE pegawai ADD COLUMN rt VARCHAR(5);
ALTER TABLE pegawai ADD COLUMN rw VARCHAR(5);
ALTER TABLE pegawai ADD COLUMN kelurahan VARCHAR(100);
ALTER TABLE pegawai ADD COLUMN id_wilayah_feeder UUID;

-- ==========================================
-- 2. UPDATE TABEL DOSEN (HAPUS REDUDANSI & TAMBAH FEEDER)
-- ==========================================
-- Hapus kolom yang duplikat dengan tabel pegawai
ALTER TABLE dosen DROP COLUMN nama_dosen;
ALTER TABLE dosen DROP COLUMN email;

-- Tambah kolom khusus ikatan kerja dosen
ALTER TABLE dosen ADD COLUMN id_penugasan_feeder UUID UNIQUE;
ALTER TABLE dosen ADD COLUMN ikatan_kerja VARCHAR(10);