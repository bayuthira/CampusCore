-- migrations/YYYY..._create_cuti_tables.sql

-- Tipe ENUM untuk status pengajuan cuti
CREATE TYPE "StatusCuti" AS ENUM (
    'Diajukan',
    'Disetujui',
    'Ditolak'
);

-- Tabel untuk menyimpan kuota cuti tahunan setiap pegawai
CREATE TABLE jatah_cuti (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE CASCADE,
    tahun SMALLINT NOT NULL,
    kuota_total INT NOT NULL,
    kuota_terpakai INT NOT NULL DEFAULT 0,
    
    -- Satu pegawai hanya punya satu jatah cuti per tahun
    UNIQUE (pegawai_id, tahun)
);

-- Tabel untuk mencatat setiap pengajuan cuti
CREATE TABLE pengajuan_cuti (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE RESTRICT,
    
    tanggal_mulai DATE NOT NULL,
    tanggal_selesai DATE NOT NULL,
    jumlah_hari INT NOT NULL, -- Jumlah hari kerja, bukan hari kalender
    alasan TEXT NOT NULL,
    
    status "StatusCuti" NOT NULL DEFAULT 'Diajukan',
    
    -- Diisi oleh atasan/admin saat approval
    user_approve_id UUID REFERENCES users(id) ON DELETE SET NULL,
    catatan_approval TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index untuk pencarian cepat
CREATE INDEX idx_pengajuan_cuti_pegawai_id ON pengajuan_cuti(pegawai_id);
CREATE INDEX idx_pengajuan_cuti_status ON pengajuan_cuti(status);