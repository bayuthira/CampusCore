CREATE TYPE "StatusNilaiAkhir" AS ENUM (
    'Draft',
    'Diajukan',
    'PerluRevisi',
    'Disetujui',
    'Dipublikasikan'
);

CREATE TABLE nilai_akhir_kuliah (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jadwal_kuliah_id UUID NOT NULL UNIQUE REFERENCES jadwal_kuliah(id) ON DELETE CASCADE,
    status "StatusNilaiAkhir" NOT NULL DEFAULT 'Draft',
    catatan TEXT,
    diajukan_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    diajukan_pada TIMESTAMPTZ,
    disetujui_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    disetujui_pada TIMESTAMPTZ,
    dipublikasikan_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    dipublikasikan_pada TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE riwayat_nilai_akhir (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nilai_akhir_kuliah_id UUID NOT NULL REFERENCES nilai_akhir_kuliah(id) ON DELETE CASCADE,
    aksi VARCHAR(30) NOT NULL,
    catatan TEXT,
    dilakukan_oleh UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_riwayat_nilai_akhir_rekap
    ON riwayat_nilai_akhir(nilai_akhir_kuliah_id, created_at DESC);
