-- migrations/YYYY..._update_pangkat_golongan_enum.sql

-- 1. Ubah kolom yang menggunakan ENUM menjadi TEXT sementara
ALTER TABLE riwayat_jad ALTER COLUMN pangkat_golongan TYPE TEXT;

-- 2. Hapus ENUM yang lama
DROP TYPE "PangkatGolongan";

-- 3. Buat ENUM baru dengan nama yang deskriptif
CREATE TYPE "PangkatGolongan" AS ENUM (
    'Penata Muda / III.a',
    'Penata Muda Tk.I / III.b',
    'Penata / III.c',
    'Penata Tk. I / III.d',
    'Pembina / IV.a',
    'Pembina Tk. I / IV.b',
    'Pembina Utama Muda / IV.c',
    'Pembina Utama Madya / IV.d',
    'Pembina Utama / IV.e'
);

-- 4. Ubah kembali kolom TEXT ke ENUM yang baru
-- Karena tabelnya kosong, kita bisa langsung cast
ALTER TABLE riwayat_jad 
ALTER COLUMN pangkat_golongan TYPE "PangkatGolongan"
USING (pangkat_golongan::"PangkatGolongan");