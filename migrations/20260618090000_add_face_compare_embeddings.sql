ALTER TABLE pegawai
ADD COLUMN face_reference_embedding JSONB;

ALTER TABLE log_absensi
ADD COLUMN face_absensi_embedding JSONB;
