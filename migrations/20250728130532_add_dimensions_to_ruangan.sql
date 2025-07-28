-- migrations/YYYY..._add_dimensions_to_ruangan.sql

ALTER TABLE ruangan
ADD COLUMN panjang NUMERIC(5, 2) NOT NULL DEFAULT 0, -- Total 5 digit, 2 di belakang koma (misal: 10.50 meter)
ADD COLUMN lebar NUMERIC(5, 2) NOT NULL DEFAULT 0;   -- Sama seperti panjang-- Add migration script here
