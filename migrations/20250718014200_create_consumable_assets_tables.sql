-- migrations/YYYYMMDDHHMMSS_create_consumable_assets_tables.sql

-- Tabel 1: Katalog Aset Habis Pakai
CREATE TABLE aset_habis_pakai (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    nama_barang VARCHAR(255) NOT NULL UNIQUE, -- Contoh: 'Kertas A4 80gsm', 'Spidol Boardmarker Hitam'
    deskripsi TEXT,
    
    -- Satuan untuk barang ini, e.g., 'rim', 'box', 'buah', 'pak'
    satuan VARCHAR(50) NOT NULL,
    
    -- Stok saat ini. Akan diupdate oleh aplikasi setiap ada transaksi.
    stok INT NOT NULL DEFAULT 0,
    
    -- Batas minimum stok sebelum perlu restock
    batas_minimum_stok INT NOT NULL DEFAULT 0,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tipe data ENUM untuk jenis transaksi stok
CREATE TYPE "TipeTransaksiStok" AS ENUM (
    'Pembelian',  -- Menambah stok
    'Pengambilan', -- Mengurangi stok
    'Stok Opname' -- Penyesuaian stok
);

-- Tabel 2: "Buku Catatan" untuk semua pergerakan stok
CREATE TABLE histori_stok (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Aset mana yang stoknya berubah
    aset_id UUID NOT NULL REFERENCES aset_habis_pakai(id) ON DELETE RESTRICT,
    
    tipe_transaksi "TipeTransaksiStok" NOT NULL,
    
    -- Jumlah yang berubah. Bisa positif (untuk pembelian) atau negatif (untuk pengambilan).
    jumlah INT NOT NULL,
    
    -- Stok sebelum dan sesudah transaksi untuk memudahkan pelacakan
    saldo_sebelum INT NOT NULL,
    saldo_setelah INT NOT NULL,
    
    -- Siapa user yang melakukan aksi ini
    user_aksi_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    
    catatan TEXT,
    
    tanggal_transaksi TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_histori_stok_aset_id ON histori_stok(aset_id);-- Add migration script here
