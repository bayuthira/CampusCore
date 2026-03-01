-- Menambahkan role KARYAWAN ke tabel roles
-- Menggunakan ON CONFLICT DO NOTHING untuk mencegah error jika role sudah ada
INSERT INTO roles (name) VALUES ('KARYAWAN') ON CONFLICT (name) DO NOTHING;