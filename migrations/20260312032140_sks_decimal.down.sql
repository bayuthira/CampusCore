-- Mengembalikan ke tipe INTEGER (membulatkan nilai desimal ke bilangan bulat terdekat)
ALTER TABLE jadwal_dosen_pengampu 
ALTER COLUMN sks_substansi_total TYPE INTEGER 
USING ROUND(sks_substansi_total)::integer;