-- migrations/YYYY..._set_cascade_on_jadwal_ruangan.sql

-- Pertama, hapus foreign key constraint yang lama
-- Nama constraint default biasanya: namaTabel_namaKolom_fkey
ALTER TABLE jadwal_ruangan
DROP CONSTRAINT jadwal_ruangan_jadwal_kuliah_id_fkey;

-- Kedua, tambahkan kembali constraint dengan aturan ON DELETE CASCADE
ALTER TABLE jadwal_ruangan
ADD CONSTRAINT jadwal_ruangan_jadwal_kuliah_id_fkey
    FOREIGN KEY (jadwal_kuliah_id)
    REFERENCES jadwal_kuliah(id)
    ON DELETE CASCADE;-- Add migration script here
