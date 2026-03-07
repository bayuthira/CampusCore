-- Add migration script here
-- =======================================================
-- 1. KEMBALIKAN KOLOM LAMA KE TABEL MAHASISWA
-- =======================================================
ALTER TABLE mahasiswa ADD COLUMN prodi_id UUID;
ALTER TABLE mahasiswa ADD COLUMN nim VARCHAR(20);
ALTER TABLE mahasiswa ADD COLUMN angkatan INTEGER;
ALTER TABLE mahasiswa ADD COLUMN dosen_pa_id UUID;

-- =======================================================
-- 2. KEMBALIKAN DATA DARI REGISTRASI KE MAHASISWA
-- =======================================================
UPDATE mahasiswa m
SET prodi_id = rm.prodi_id,
    nim = rm.nim,
    angkatan = rm.angkatan,
    dosen_pa_id = rm.dosen_pa_id
FROM registrasi_mahasiswa rm
WHERE m.id = rm.mahasiswa_id;

-- Set constraints lama
ALTER TABLE mahasiswa ADD CONSTRAINT mahasiswa_prodi_id_fkey FOREIGN KEY (prodi_id) REFERENCES prodi(id) ON DELETE RESTRICT;
ALTER TABLE mahasiswa ADD CONSTRAINT mahasiswa_dosen_pa_id_fkey FOREIGN KEY (dosen_pa_id) REFERENCES dosen(id) ON DELETE SET NULL;
ALTER TABLE mahasiswa ADD CONSTRAINT mahasiswa_nim_key UNIQUE (nim);

-- =======================================================
-- 3. KEMBALIKAN TABEL ENROLLMENTS (KRS)
-- =======================================================
ALTER TABLE enrollments ADD COLUMN mahasiswa_id UUID;

UPDATE enrollments e
SET mahasiswa_id = rm.mahasiswa_id
FROM registrasi_mahasiswa rm
WHERE e.registrasi_id = rm.id;

ALTER TABLE enrollments ALTER COLUMN mahasiswa_id SET NOT NULL;
ALTER TABLE enrollments ADD CONSTRAINT enrollments_mahasiswa_id_fkey FOREIGN KEY (mahasiswa_id) REFERENCES mahasiswa(id) ON DELETE CASCADE;

ALTER TABLE enrollments DROP CONSTRAINT enrollments_registrasi_id_fkey;
ALTER TABLE enrollments DROP COLUMN registrasi_id;

-- =======================================================
-- 4. HAPUS TABEL REGISTRASI & KOLOM BIODATA
-- =======================================================
DROP TABLE registrasi_mahasiswa;

ALTER TABLE mahasiswa DROP COLUMN id_mahasiswa_feeder;
ALTER TABLE mahasiswa DROP COLUMN nik;
ALTER TABLE mahasiswa DROP COLUMN tempat_lahir;
ALTER TABLE mahasiswa DROP COLUMN tanggal_lahir;
ALTER TABLE mahasiswa DROP COLUMN jenis_kelamin;
ALTER TABLE mahasiswa DROP COLUMN nama_ibu_kandung;
ALTER TABLE mahasiswa DROP COLUMN agama_id;
ALTER TABLE mahasiswa DROP COLUMN kewarganegaraan;
ALTER TABLE mahasiswa DROP COLUMN alamat;
ALTER TABLE mahasiswa DROP COLUMN kelurahan;
ALTER TABLE mahasiswa DROP COLUMN id_wilayah_feeder;