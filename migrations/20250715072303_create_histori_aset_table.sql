-- migrations/YYYY..._create_histori_aset_table.sql

-- Tipe data ENUM untuk mencatat jenis kejadian pada aset
CREATE TYPE "AsetHistoriStatus" AS ENUM (
    'Ditempatkan',
    'Dipindahkan',
    'Dipinjam',
    'Dikembalikan',
    'Dalam Perbaikan',
    'Perbaikan Selesai',
    'Dihapuskan'
);

CREATE TABLE histori_aset (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Aset mana yang mengalami kejadian ini
    aset_id UUID NOT NULL REFERENCES aset(id) ON DELETE CASCADE,
    
    -- Opsional: Pindah dari ruangan mana
    dari_ruangan_id UUID REFERENCES ruangan(id) ON DELETE SET NULL,
    
    -- Opsional: Pindah ke ruangan mana
    ke_ruangan_id UUID REFERENCES ruangan(id) ON DELETE SET NULL,
    
    -- Siapa user yang melakukan aksi ini
    user_aksi_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    
    -- Status atau jenis kejadian
    status "AsetHistoriStatus" NOT NULL,
    
    catatan TEXT,
    
    -- Tanggal kejadian
    tanggal_kejadian TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_histori_aset_aset_id ON histori_aset(aset_id);-- Add migration script here
