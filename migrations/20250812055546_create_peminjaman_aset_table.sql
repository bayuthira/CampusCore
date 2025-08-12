-- migrations/YYYY..._create_peminjaman_aset_table.sql

-- Tipe data ENUM untuk status peminjaman
CREATE TYPE "PeminjamanStatus" AS ENUM (
    'Dipinjam',
    'Dikembalikan',
    'Terlambat'
);

-- Tabel utama untuk mencatat setiap transaksi peminjaman
CREATE TABLE peminjaman_aset (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Aset mana yang dipinjam
    aset_id UUID NOT NULL REFERENCES aset(id) ON DELETE RESTRICT,

    -- Siapa user yang meminjam
    user_peminjam_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    tanggal_pinjam TIMESTAMPTZ NOT NULL DEFAULT now(),
    estimasi_tanggal_kembali TIMESTAMPTZ NOT NULL,

    -- Diisi saat aset sudah dikembalikan
    tanggal_kembali_aktual TIMESTAMPTZ,

    status "PeminjamanStatus" NOT NULL DEFAULT 'Dipinjam',

    catatan_pinjam TEXT,
    catatan_kembali TEXT,

    -- Siapa user (staf BAUM) yang menyetujui peminjaman dan pengembalian
    user_approve_pinjam_id UUID REFERENCES users(id) ON DELETE SET NULL,
    user_approve_kembali_id UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_peminjaman_aset_aset_id ON peminjaman_aset(aset_id);
CREATE INDEX idx_peminjaman_aset_user_peminjam_id ON peminjaman_aset(user_peminjam_id);-- Add migration script here
