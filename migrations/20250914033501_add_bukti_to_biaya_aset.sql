-- migrations/YYYY..._add_bukti_to_biaya_aset.sql
ALTER TABLE biaya_aset
ADD COLUMN bukti_url VARCHAR(255); -- Path ke file yang di-upload-- Add migration script here
