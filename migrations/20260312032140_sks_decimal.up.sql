-- Mengubah tipe data menjadi NUMERIC (mendukung desimal)
ALTER TABLE jadwal_dosen_pengampu 
ALTER COLUMN sks_substansi_total TYPE NUMERIC(4,2) 
USING sks_substansi_total::numeric;