-- migrations/YYYYMMDDHHMMSS_create_mahasiswa_table.sql
CREATE TABLE mahasiswa (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    nim VARCHAR(20) NOT NULL UNIQUE,
    nama_mahasiswa VARCHAR(255) NOT NULL,
    angkatan INT NOT NULL, -- Tahun masuk
    email VARCHAR(255) UNIQUE,

    -- Foreign Key ke tabel Users. Ini mengikat profil mahasiswa ke akun loginnya.
    -- UNIQUE memastikan satu user hanya bisa punya satu profil mahasiswa.
    -- ON DELETE SET NULL berarti jika akun user dihapus, profil mahasiswa tetap ada tapi tidak lagi terhubung (user_id menjadi NULL).
    user_id UUID UNIQUE REFERENCES users(id) ON DELETE SET NULL,

    -- Foreign Key ke tabel Prodi
    prodi_id UUID NOT NULL REFERENCES prodi(id) ON DELETE RESTRICT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_mahasiswa_nim ON mahasiswa(nim);
CREATE INDEX idx_mahasiswa_prodi_id ON mahasiswa(prodi_id);
CREATE INDEX idx_mahasiswa_user_id ON mahasiswa(user_id);