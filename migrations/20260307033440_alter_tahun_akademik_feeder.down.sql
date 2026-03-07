-- Menghapus kolom jika migrasi di-rollback
ALTER TABLE tahun_akademik 
DROP COLUMN id_semester_feeder;