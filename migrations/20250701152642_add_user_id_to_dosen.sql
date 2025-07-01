-- migrations/YYYYMMDDHHMMSS_add_user_id_to_dosen.sql
ALTER TABLE dosen
ADD COLUMN user_id UUID UNIQUE REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX idx_dosen_user_id ON dosen(user_id);-- Add migration script here
