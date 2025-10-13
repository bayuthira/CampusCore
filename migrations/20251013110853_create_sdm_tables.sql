-- migrations/YYYY..._create_sdm_tables.sql

-- Tipe data ENUM berdasarkan data di file CSV
CREATE TYPE "JenisKelamin" AS ENUM ('L', 'P');
CREATE TYPE "StatusNikah" AS ENUM ('Menikah', 'Belum Menikah', 'Cerai Hidup', 'Cerai Mati');
CREATE TYPE "KategoriPegawai" AS ENUM ('Tenaga Pendidik', 'Tenaga Kependidikan');
CREATE TYPE "StatusPegawai" AS ENUM ('Tetap', 'Kontrak', 'Honorer');

-- Tabel 1: Data Induk Pegawai (Revisi)
CREATE TABLE pegawai (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE REFERENCES users(id) ON DELETE SET NULL, -- Relasi ke akun login

    nik VARCHAR(50) UNIQUE NOT NULL, -- NIK dari file CSV
    no_ktp VARCHAR(20) UNIQUE,
    gelar_depan VARCHAR(50),
    nama_lengkap VARCHAR(255) NOT NULL,
    gelar_belakang VARCHAR(50),
    
    tempat_lahir VARCHAR(100),
    tanggal_lahir DATE,
    jenis_kelamin "JenisKelamin",
    status_nikah "StatusNikah",
    agama VARCHAR(50),
    
    alamat_domisili TEXT,
    nomor_hp VARCHAR(20),
    email VARCHAR(255) UNIQUE,

    kategori_pegawai "KategoriPegawai",
    status_pegawai "StatusPegawai",
    unit_kerja VARCHAR(100), -- Contoh: 'Program Studi'
    bagian VARCHAR(100),     -- Contoh: 'Sarjana Kebidanan'
    jabatan VARCHAR(255),    -- Contoh: 'Dosen Tetap Yayasan'
    
    tanggal_masuk DATE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tabel 2: Riwayat Pendidikan Pegawai (satu pegawai bisa punya banyak)
-- Desain ini sudah cukup baik dan tidak perlu diubah.
CREATE TABLE riwayat_pendidikan (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE CASCADE,
    jenjang VARCHAR(50) NOT NULL, -- S1, S2, D3, dll.
    institusi VARCHAR(255) NOT NULL,
    jurusan VARCHAR(255),
    tahun_lulus SMALLINT
);

-- Tabel 3: Menghubungkan Dosen ke profil Pegawai (tidak berubah)
ALTER TABLE dosen ADD COLUMN pegawai_id UUID UNIQUE REFERENCES pegawai(id) ON DELETE SET NULL;