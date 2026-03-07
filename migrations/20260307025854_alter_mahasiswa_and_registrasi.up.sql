-- Add migration script here
-- =======================================================
-- 1. TAMBAH KOLOM BIODATA & FEEDER KE TABEL MAHASISWA
-- =======================================================
ALTER TABLE mahasiswa ADD COLUMN id_mahasiswa_feeder UUID UNIQUE;
ALTER TABLE mahasiswa ADD COLUMN nik VARCHAR(16) UNIQUE;
ALTER TABLE mahasiswa ADD COLUMN tempat_lahir VARCHAR(100);
ALTER TABLE mahasiswa ADD COLUMN tanggal_lahir DATE;
ALTER TABLE mahasiswa ADD COLUMN jenis_kelamin VARCHAR(1); -- 'L' atau 'P'
ALTER TABLE mahasiswa ADD COLUMN nama_ibu_kandung VARCHAR(255);
ALTER TABLE mahasiswa ADD COLUMN agama_id INTEGER;
ALTER TABLE mahasiswa ADD COLUMN kewarganegaraan VARCHAR(2) DEFAULT 'ID';
ALTER TABLE mahasiswa ADD COLUMN alamat TEXT;
ALTER TABLE mahasiswa ADD COLUMN kelurahan VARCHAR(100);
ALTER TABLE mahasiswa ADD COLUMN id_wilayah_feeder UUID;

-- =======================================================
-- 2. BUAT TABEL REGISTRASI MAHASISWA (AKADEMIK)
-- =======================================================
CREATE TABLE registrasi_mahasiswa (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mahasiswa_id UUID NOT NULL REFERENCES mahasiswa(id) ON DELETE CASCADE,
    prodi_id UUID NOT NULL REFERENCES prodi(id) ON DELETE RESTRICT,
    nim VARCHAR(20) NOT NULL UNIQUE,
    angkatan INTEGER NOT NULL,
    dosen_pa_id UUID REFERENCES dosen(id) ON DELETE SET NULL,
    id_reg_pd_feeder UUID UNIQUE,
    periode_masuk VARCHAR(10), -- Contoh: '20231' untuk Ganjil 2023
    jenis_pendaftaran_id INTEGER DEFAULT 1, -- 1=Baru, 2=Pindahan
    tanggal_daftar DATE,
    status_mahasiswa VARCHAR(20) DEFAULT 'Aktif',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

-- =======================================================
-- 3. MIGRASI DATA AKADEMIK DARI MAHASISWA KE REGISTRASI
-- =======================================================
-- Memindahkan data NIM, Prodi, dll yang sudah ada tanpa menghapusnya
INSERT INTO registrasi_mahasiswa (mahasiswa_id, prodi_id, nim, angkatan, dosen_pa_id)
SELECT id, prodi_id, nim, angkatan, dosen_pa_id FROM mahasiswa;

-- =======================================================
-- 4. UBAH RELASI TABEL ENROLLMENTS (KRS)
-- =======================================================
-- Tambah kolom baru
ALTER TABLE enrollments ADD COLUMN registrasi_id UUID;

-- Mapping data: Isi registrasi_id berdasarkan mahasiswa_id
UPDATE enrollments e
SET registrasi_id = rm.id
FROM registrasi_mahasiswa rm
WHERE e.mahasiswa_id = rm.mahasiswa_id;

-- Jadikan registrasi_id Wajib (Not Null) dan set Foreign Key
ALTER TABLE enrollments ALTER COLUMN registrasi_id SET NOT NULL;
ALTER TABLE enrollments ADD CONSTRAINT enrollments_registrasi_id_fkey FOREIGN KEY (registrasi_id) REFERENCES registrasi_mahasiswa(id) ON DELETE CASCADE;

-- Hapus relasi lama
ALTER TABLE enrollments DROP CONSTRAINT enrollments_mahasiswa_id_fkey;
ALTER TABLE enrollments DROP COLUMN mahasiswa_id;

-- =======================================================
-- 5. BERSIHKAN KOLOM LAMA DI TABEL MAHASISWA
-- =======================================================
ALTER TABLE mahasiswa DROP CONSTRAINT mahasiswa_prodi_id_fkey;
ALTER TABLE mahasiswa DROP CONSTRAINT mahasiswa_dosen_pa_id_fkey;
ALTER TABLE mahasiswa DROP COLUMN prodi_id;
ALTER TABLE mahasiswa DROP COLUMN nim;
ALTER TABLE mahasiswa DROP COLUMN angkatan;
ALTER TABLE mahasiswa DROP COLUMN dosen_pa_id;