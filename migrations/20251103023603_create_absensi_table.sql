-- migrations/YYYY..._create_absensi_table.sql

-- ENUM untuk status kehadiran harian
CREATE TYPE "StatusAbsensi" AS ENUM (
    'Hadir',
    'Sakit',
    'Ijin',
    'Cuti',
    'Alpa'
);

-- ENUM untuk tipe aksi absensi (punch)
CREATE TYPE "TipeAbsensi" AS ENUM (
    'ClockIn',
    'ClockOut'
);

-- Tabel 1: Rekap Absensi Harian (untuk status akhir per hari)
CREATE TABLE rekap_absensi_harian (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE RESTRICT,
    tanggal DATE NOT NULL,
    status "StatusAbsensi" NOT NULL,
    keterangan TEXT, -- Misal: 'Sakit (Surat Dokter)', 'Cuti Tahunan'
    
    -- Satu pegawai hanya punya satu rekap status per tanggal
    UNIQUE (pegawai_id, tanggal)
);

-- Tabel 2: Log Absensi (untuk setiap punch clock-in/out)
CREATE TABLE log_absensi (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE RESTRICT,
    
    waktu_absensi TIMESTAMPTZ NOT NULL, -- Waktu pasti saat menekan tombol
    tipe_absensi "TipeAbsensi" NOT NULL,
    
    -- Data GPS
    latitude NUMERIC(10, 7) NOT NULL,
    longitude NUMERIC(10, 7) NOT NULL,
    alamat_absensi TEXT -- Opsional, untuk hasil reverse-geocoding
);

-- Index untuk pencarian cepat
CREATE INDEX idx_rekap_absensi_pegawai_tanggal ON rekap_absensi_harian(pegawai_id, tanggal);
CREATE INDEX idx_log_absensi_pegawai_waktu ON log_absensi(pegawai_id, waktu_absensi);