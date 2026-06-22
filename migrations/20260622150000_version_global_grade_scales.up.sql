ALTER TABLE skala_nilai ALTER COLUMN prodi_id DROP NOT NULL;
ALTER TABLE skala_nilai
    ADD COLUMN is_locked BOOLEAN NOT NULL DEFAULT false;

COMMENT ON COLUMN skala_nilai.prodi_id IS
    'NULL berarti skala global institusi; UUID berarti override untuk Prodi.';

CREATE INDEX idx_skala_nilai_scope_effective
    ON skala_nilai(prodi_id, tanggal_mulai_efektif, tanggal_akhir_efektif);

CREATE TABLE skala_nilai_feeder_map (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    skala_nilai_id UUID NOT NULL REFERENCES skala_nilai(id) ON DELETE RESTRICT,
    prodi_id UUID NOT NULL REFERENCES prodi(id) ON DELETE CASCADE,
    id_bobot_nilai_feeder UUID UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(skala_nilai_id, prodi_id)
);

INSERT INTO skala_nilai_feeder_map (
    skala_nilai_id, prodi_id, id_bobot_nilai_feeder
)
SELECT id, prodi_id, id_bobot_nilai_feeder
FROM skala_nilai
WHERE prodi_id IS NOT NULL AND id_bobot_nilai_feeder IS NOT NULL
ON CONFLICT (skala_nilai_id, prodi_id) DO NOTHING;
