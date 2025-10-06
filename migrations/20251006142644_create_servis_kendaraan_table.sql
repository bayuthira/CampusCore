-- migrations/YYYY..._create_servis_kendaraan_table.sql
CREATE TABLE servis_kendaraan (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kendaraan_id UUID NOT NULL REFERENCES kendaraan(id) ON DELETE CASCADE,

    tanggal_servis DATE NOT NULL,
    odometer_saat_servis INT NOT NULL,
    deskripsi TEXT NOT NULL,
    biaya NUMERIC(15, 2) NOT NULL,

    -- Siapa user yang mencatat servis ini
    user_pencatat_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_servis_kendaraan_kendaraan_id ON servis_kendaraan(kendaraan_id);