-- Kembalikan tabel dosen
ALTER TABLE dosen ADD COLUMN nama_dosen VARCHAR(255);
ALTER TABLE dosen ADD COLUMN email VARCHAR(255);

-- (Opsional: Mengisi kembali nama_dosen dari tabel pegawai jika di-rollback)
UPDATE dosen d SET nama_dosen = p.nama_lengkap, email = p.email FROM pegawai p WHERE d.pegawai_id = p.id;

ALTER TABLE dosen DROP COLUMN id_penugasan_feeder;
ALTER TABLE dosen DROP COLUMN ikatan_kerja;

-- Kembalikan tabel pegawai
ALTER TABLE pegawai DROP COLUMN id_sdm_feeder;
ALTER TABLE pegawai DROP COLUMN nama_ibu_kandung;
ALTER TABLE pegawai DROP COLUMN kewarganegaraan;
ALTER TABLE pegawai DROP COLUMN dusun;
ALTER TABLE pegawai DROP COLUMN rt;
ALTER TABLE pegawai DROP COLUMN rw;
ALTER TABLE pegawai DROP COLUMN kelurahan;
ALTER TABLE pegawai DROP COLUMN id_wilayah_feeder;