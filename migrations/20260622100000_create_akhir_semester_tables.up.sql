ALTER TABLE tahun_akademik
    ADD COLUMN status_penutupan VARCHAR(20) NOT NULL DEFAULT 'Terbuka'
        CHECK (status_penutupan IN ('Terbuka', 'Ditutup')),
    ADD COLUMN ditutup_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN ditutup_pada TIMESTAMPTZ;

CREATE TYPE "StatusKoreksiNilai" AS ENUM (
    'Diajukan', 'Disetujui', 'Ditolak', 'Diterapkan'
);

CREATE TABLE koreksi_nilai (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    enrollment_id UUID NOT NULL REFERENCES enrollments(id) ON DELETE CASCADE,
    nilai_angka_lama NUMERIC(5,2),
    nilai_huruf_lama VARCHAR(5),
    nilai_indeks_lama NUMERIC(3,2),
    nilai_angka_baru NUMERIC(5,2) NOT NULL CHECK (nilai_angka_baru BETWEEN 0 AND 100),
    nilai_huruf_baru VARCHAR(5) NOT NULL,
    nilai_indeks_baru NUMERIC(3,2) NOT NULL CHECK (nilai_indeks_baru BETWEEN 0 AND 4),
    alasan TEXT NOT NULL,
    status "StatusKoreksiNilai" NOT NULL DEFAULT 'Diajukan',
    diajukan_oleh UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    ditinjau_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    ditinjau_pada TIMESTAMPTZ,
    diterapkan_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    diterapkan_pada TIMESTAMPTZ,
    catatan_review TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uq_koreksi_nilai_aktif
    ON koreksi_nilai(enrollment_id)
    WHERE status IN ('Diajukan', 'Disetujui');

CREATE TYPE "StatusSinkronisasiFeeder" AS ENUM (
    'Menunggu', 'Diproses', 'Berhasil', 'Gagal'
);

CREATE TABLE feeder_sync_outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(40) NOT NULL,
    entity_id UUID NOT NULL,
    operation VARCHAR(20) NOT NULL DEFAULT 'UPSERT',
    payload JSONB NOT NULL,
    status "StatusSinkronisasiFeeder" NOT NULL DEFAULT 'Menunggu',
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    next_attempt_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(entity_type, entity_id, operation)
);

CREATE INDEX idx_feeder_outbox_pending
    ON feeder_sync_outbox(status, next_attempt_at, created_at);
