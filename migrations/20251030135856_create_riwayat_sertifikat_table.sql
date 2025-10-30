-- migrations/YYYY..._create_riwayat_sertifikat_table.sql

-- ENUM untuk Kategori (Pelatihan, Seminar, dll.)
CREATE TYPE "KategoriSertifikat" AS ENUM (
    'Pelatihan',
    'BIMTEK',
    'Seminar',
    'Workshop',
    'Rekognisi Dosen'
);

-- ENUM untuk Tingkat (Lokal, Nasional, dll.)
CREATE TYPE "TingkatSertifikat" AS ENUM (
    'Lokal',
    'Nasional',
    'Internasional'
);

-- Tabel utama untuk riwayat sertifikat
CREATE TABLE riwayat_sertifikat (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE CASCADE,

    jenis_sertifikat "KategoriSertifikat" NOT NULL,
    judul_sertifikat TEXT NOT NULL,
    nomor_sertifikat VARCHAR(255),
    tanggal_pelaksanaan DATE NOT NULL,
    tingkat "TingkatSertifikat" NOT NULL,
    penyelenggara VARCHAR(255),
    keterangan TEXT
);

CREATE INDEX idx_riwayat_sertifikat_pegawai_id ON riwayat_sertifikat(pegawai_id);