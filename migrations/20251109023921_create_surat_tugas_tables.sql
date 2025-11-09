-- migrations/YYYY..._create_surat_tugas_tables.sql

-- 1. Tabel Master untuk Surat Tugas
CREATE TABLE surat_tugas_master (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Nomor surat yang sudah di-generate, e.g., "090/ST/STIKES-R/XI/2025"
    nomor_surat VARCHAR(255) NOT NULL UNIQUE,
    
    dasar_tugas TEXT, -- Latar belakang/alasan (konsideran)
    tugas TEXT NOT NULL, -- Deskripsi tugas yang diberikan
    tempat_tugas VARCHAR(255) NOT NULL,
    tanggal_mulai DATE NOT NULL,
    tanggal_selesai DATE NOT NULL,
    
    -- ID Pegawai yang bertindak sebagai penandatangan
    penandatangan_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE RESTRICT,
    
    -- Tembusan (CC) bisa disimpan sebagai array teks
    tembusan TEXT[],
    
    -- Metadata
    user_pembuat_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2. Tabel Perantara untuk Penerima Tugas (Many-to-Many)
CREATE TABLE surat_tugas_penerima (
    surat_tugas_id UUID NOT NULL REFERENCES surat_tugas_master(id) ON DELETE CASCADE,
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE RESTRICT,
    PRIMARY KEY (surat_tugas_id, pegawai_id)
);

-- 3. Tabel Counter untuk Penomoran Surat Otomatis
CREATE TABLE penomoran_surat_counter (
    kode VARCHAR(20) NOT NULL, -- e.g., 'ST' (Surat Tugas)
    tahun SMALLINT NOT NULL,
    counter INT NOT NULL,
    PRIMARY KEY (kode, tahun)
);

-- Inisialisasi counter untuk Surat Tugas tahun ini (opsional)
INSERT INTO penomoran_surat_counter(kode, tahun, counter) VALUES ('ST', 2025, 0);