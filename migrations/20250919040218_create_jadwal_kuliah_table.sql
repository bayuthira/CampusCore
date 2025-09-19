-- migrations/YYYY..._create_jadwal_kuliah_table.sql

-- Tipe ENUM untuk nama hari
CREATE TYPE "DayOfWeek" AS ENUM (
    'Senin', 'Selasa', 'Rabu', 'Kamis', 'Jumat', 'Sabtu', 'Minggu'
);

-- Tabel 1: Tabel master untuk Jadwal Kuliah (Template)
-- PERHATIKAN: kolom dosen_id sudah dihapus dari sini.
CREATE TABLE jadwal_kuliah (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    matakuliah_id UUID NOT NULL REFERENCES mata_kuliah(id) ON DELETE RESTRICT,
    tahun_akademik_id UUID NOT NULL REFERENCES tahun_akademik(id) ON DELETE RESTRICT,
    hari "DayOfWeek" NOT NULL,
    jam_mulai TIME NOT NULL,
    jam_selesai TIME NOT NULL,
    kelas VARCHAR(20) NOT NULL DEFAULT 'A',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (matakuliah_id, tahun_akademik_id, kelas)
);

-- Tipe ENUM untuk peran dosen pengampu
CREATE TYPE "PeranDosenPengampu" AS ENUM (
    'Koordinator',
    'Anggota'
);

-- Tabel 2: Tabel perantara (pivot) untuk Dosen Pengampu (Many-to-Many)
CREATE TABLE jadwal_dosen_pengampu (
    jadwal_kuliah_id UUID NOT NULL REFERENCES jadwal_kuliah(id) ON DELETE CASCADE,
    dosen_id UUID NOT NULL REFERENCES dosen(id) ON DELETE RESTRICT,
    peran "PeranDosenPengampu" NOT NULL,
    PRIMARY KEY (jadwal_kuliah_id, dosen_id)
);

-- Tabel 3: Modifikasi tabel jadwal_ruangan yang sudah ada
ALTER TABLE jadwal_ruangan
ADD COLUMN jadwal_kuliah_id UUID REFERENCES jadwal_kuliah(id) ON DELETE SET NULL;