-- migrations/YYYY..._create_aset_table.sql
CREATE TABLE aset (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nama_aset VARCHAR(255) NOT NULL, -- Contoh: 'Kursi Dosen', 'Proyektor Epson EB-X500'
    kode_aset VARCHAR(100) UNIQUE,  -- Kode inventaris, bisa jadi opsional

    -- Foreign Key ke Jenis Aset
    jenis_aset_id UUID NOT NULL REFERENCES jenis_aset(id) ON DELETE RESTRICT,

    -- Foreign Key ke Ruangan (aset ini berada di ruangan mana)
    -- Dibuat opsional karena mungkin ada aset yang belum ditempatkan
    ruangan_id UUID REFERENCES ruangan(id) ON DELETE SET NULL,

    -- Deskripsi/spesifikasi lain
    deskripsi TEXT,
    tanggal_pembelian DATE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);