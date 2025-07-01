-- migrations/YYYYMMDDHHMMSS_add_dosen_pa_to_mahasiswa.sql
ALTER TABLE mahasiswa
ADD COLUMN dosen_pa_id UUID REFERENCES dosen(id) ON DELETE SET NULL;

CREATE INDEX idx_mahasiswa_dosen_pa_id ON mahasiswa(dosen_pa_id);-- Add migration script here
