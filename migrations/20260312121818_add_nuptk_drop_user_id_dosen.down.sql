-- 1. Kembalikan user_id di tabel Dosen
ALTER TABLE dosen ADD COLUMN user_id UUID;
ALTER TABLE dosen ADD CONSTRAINT dosen_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL;

-- Kembalikan data user_id dari tabel pegawai
UPDATE dosen d SET user_id = p.user_id FROM pegawai p WHERE d.pegawai_id = p.id;

-- 2. Hapus kolom nuptk dari tabel Pegawai
ALTER TABLE pegawai DROP COLUMN nuptk;