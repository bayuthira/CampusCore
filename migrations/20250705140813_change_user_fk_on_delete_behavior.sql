-- migrations/YYYYMMDDHHMMSS_change_user_fk_on_delete_behavior.sql

-- Mengubah constraint untuk tabel mahasiswa
-- Pertama, hapus constraint yang lama (nama defaultnya biasanya tabel_kolom_fkey)
ALTER TABLE mahasiswa DROP CONSTRAINT mahasiswa_user_id_fkey;
-- Tambahkan kembali dengan aturan ON DELETE RESTRICT
ALTER TABLE mahasiswa ADD CONSTRAINT mahasiswa_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT;

-- Mengubah constraint untuk tabel dosen
-- Hapus constraint yang lama
ALTER TABLE dosen DROP CONSTRAINT dosen_user_id_fkey;
-- Tambahkan kembali dengan aturan ON DELETE RESTRICT
ALTER TABLE dosen ADD CONSTRAINT dosen_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT;