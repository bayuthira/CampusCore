-- migrations/YYYY..._create_struktur_organisasi_tables.sql

-- 1. Tabel Master untuk Unit Kerja (Struktur Organisasi)
-- Menyimpan daftar semua departemen, fakultas, prodi, biro, dll.
CREATE TABLE unit_kerja (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 'induk_unit_id' adalah self-reference untuk struktur pohon (tree)
    -- Contoh: Prodi "Sarjana Kebidanan" (anak) punya induk "Fakultas Kesehatan" (induk)
    induk_unit_id UUID REFERENCES unit_kerja(id) ON DELETE SET NULL,

    kode_unit VARCHAR(50) UNIQUE,
    nama_unit VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true -- Untuk menonaktifkan unit kerja lama
);

-- 2. Tabel Riwayat Penempatan Pegawai (Jabatan & Unit Kerja)
-- Ini adalah tabel histori yang melacak perpindahan pegawai.
CREATE TABLE penempatan_pegawai (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pegawai_id UUID NOT NULL REFERENCES pegawai(id) ON DELETE CASCADE,
    unit_kerja_id UUID NOT NULL REFERENCES unit_kerja(id) ON DELETE RESTRICT,

    jabatan VARCHAR(255) NOT NULL, -- Jabatan pegawai di unit tersebut
    nomor_sk VARCHAR(255),
    tanggal_mulai DATE NOT NULL,

    -- Jika tanggal_selesai NULL, berarti ini adalah penempatan AKTIF
    tanggal_selesai DATE, 

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 3. Hapus kolom-kolom lama dari tabel `pegawai`
-- Data ini sekarang akan disimpan di `penempatan_pegawai`
ALTER TABLE pegawai
    DROP COLUMN unit_kerja,
    DROP COLUMN bagian,
    DROP COLUMN jabatan;

-- Index untuk pencarian cepat
CREATE INDEX idx_penempatan_pegawai_id ON penempatan_pegawai(pegawai_id);
CREATE INDEX idx_penempatan_unit_kerja_id ON penempatan_pegawai(unit_kerja_id);