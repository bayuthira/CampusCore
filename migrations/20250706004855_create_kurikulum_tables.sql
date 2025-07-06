-- migrations/YYYYMMDDHHMMSS_create_kurikulum_tables.sql

CREATE TABLE kurikulum (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nama VARCHAR(255) NOT NULL,
    tahun_mulai SMALLINT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT false,
    prodi_id UUID NOT NULL REFERENCES prodi(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (prodi_id, nama)
);

CREATE TABLE kurikulum_matakuliah (
    kurikulum_id UUID NOT NULL REFERENCES kurikulum(id) ON DELETE CASCADE,
    matakuliah_id UUID NOT NULL REFERENCES mata_kuliah(id) ON DELETE CASCADE,
    PRIMARY KEY (kurikulum_id, matakuliah_id)
);