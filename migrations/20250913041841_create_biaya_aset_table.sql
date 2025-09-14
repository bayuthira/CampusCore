-- migrations/YYYY..._create_biaya_aset_table.sql

-- Tipe data ENUM untuk jenis transaksi biaya
CREATE TYPE "TipeBiaya" AS ENUM (
    'Pembelian',
    'Perawatan',
    'Perbaikan',
    'Upgrade',
    'Lain-lain'
);

-- Tabel untuk mencatat semua biaya terkait aset
CREATE TABLE biaya_aset (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Aset mana yang biayanya dicatat
    aset_id UUID NOT NULL REFERENCES aset(id) ON DELETE CASCADE,

    tipe_biaya "TipeBiaya" NOT NULL,

    deskripsi TEXT NOT NULL, -- Contoh: 'Pembelian awal', 'Ganti layar LCD', 'Servis tahunan'

    -- Gunakan NUMERIC untuk data keuangan agar presisi
    jumlah NUMERIC(15, 2) NOT NULL,

    tanggal_transaksi DATE NOT NULL,

    vendor VARCHAR(255), -- Opsional, nama toko atau penyedia jasa

    -- Siapa user yang mencatat biaya ini
    user_pencatat_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_biaya_aset_aset_id ON biaya_aset(aset_id);