DROP TABLE IF EXISTS skala_nilai_feeder_map;
DROP INDEX IF EXISTS idx_skala_nilai_scope_effective;
ALTER TABLE skala_nilai DROP COLUMN IF EXISTS is_locked;
DELETE FROM skala_nilai WHERE prodi_id IS NULL;
ALTER TABLE skala_nilai ALTER COLUMN prodi_id SET NOT NULL;
