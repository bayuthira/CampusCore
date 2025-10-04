-- migrations/YYYY..._create_fleet_management_tables.sql

-- Tipe data ENUM untuk jenis dan status kendaraan
CREATE TYPE "JenisKendaraan" AS ENUM ('Mobil', 'Motor', 'Bus');
CREATE TYPE "StatusKendaraan" AS ENUM ('Tersedia', 'Digunakan', 'Perawatan');

-- Tabel 1: Data Master Kendaraan
CREATE TABLE kendaraan (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jenis "JenisKendaraan" NOT NULL,
    nama VARCHAR(255) NOT NULL, -- Contoh: 'Toyota Avanza Hitam', 'Honda Vario 125'
    nomor_polisi VARCHAR(15) NOT NULL UNIQUE,
    merk VARCHAR(100),
    model VARCHAR(100),
    tahun SMALLINT,
    status "StatusKendaraan" NOT NULL DEFAULT 'Tersedia',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tipe data ENUM untuk status booking
CREATE TYPE "StatusBooking" AS ENUM ('Diajukan', 'Disetujui', 'Ditolak', 'Dibatalkan', 'Berlangsung', 'Selesai');

-- Tabel 2: Tabel untuk mencatat setiap pemesanan kendaraan
CREATE TABLE booking_kendaraan (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kendaraan_id UUID NOT NULL REFERENCES kendaraan(id) ON DELETE RESTRICT,
    user_pemesan_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    tujuan TEXT NOT NULL,
    waktu_berangkat TIMESTAMPTZ NOT NULL,
    estimasi_waktu_kembali TIMESTAMPTZ NOT NULL,

    status "StatusBooking" NOT NULL DEFAULT 'Diajukan',

    -- Diisi oleh admin saat approval
    user_approve_id UUID REFERENCES users(id) ON DELETE SET NULL,
    catatan_approval TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tabel 3: Log penggunaan aktual setelah booking disetujui
CREATE TABLE log_penggunaan (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booking_id UUID NOT NULL REFERENCES booking_kendaraan(id) ON DELETE CASCADE,

    -- Dicatat saat kunci diserahkan
    odometer_awal INT,
    waktu_aktual_berangkat TIMESTAMPTZ,

    -- Dicatat saat kunci dikembalikan
    odometer_akhir INT,
    waktu_aktual_kembali TIMESTAMPTZ,
    bahan_bakar_diisi NUMERIC(5,2), -- Contoh: 20.50 liter

    catatan_kondisi_kembali TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);