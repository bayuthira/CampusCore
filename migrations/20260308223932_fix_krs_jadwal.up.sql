-- Hapus constraint foreign key lama
ALTER TABLE enrollments DROP CONSTRAINT enrollments_matakuliah_id_fkey;

-- Ubah nama kolom agar nyambung dengan jadwal_kuliah
ALTER TABLE enrollments RENAME COLUMN matakuliah_id TO jadwal_kuliah_id;

-- Tambahkan foreign key baru ke jadwal_kuliah
ALTER TABLE enrollments ADD CONSTRAINT enrollments_jadwal_kuliah_id_fkey 
FOREIGN KEY (jadwal_kuliah_id) REFERENCES jadwal_kuliah(id) ON DELETE RESTRICT;