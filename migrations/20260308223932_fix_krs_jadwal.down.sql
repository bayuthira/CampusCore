-- Hapus constraint foreign key yang baru
ALTER TABLE enrollments DROP CONSTRAINT enrollments_jadwal_kuliah_id_fkey;

-- Kembalikan nama kolom dari jadwal_kuliah_id menjadi matakuliah_id
ALTER TABLE enrollments RENAME COLUMN jadwal_kuliah_id TO matakuliah_id;

-- Tambahkan kembali foreign key lama yang mengarah ke tabel mata_kuliah
ALTER TABLE enrollments ADD CONSTRAINT enrollments_matakuliah_id_fkey 
FOREIGN KEY (matakuliah_id) REFERENCES mata_kuliah(id) ON DELETE RESTRICT;