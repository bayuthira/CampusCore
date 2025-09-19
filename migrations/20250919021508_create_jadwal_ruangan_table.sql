-- migrations/YYYY..._create_jadwal_ruangan_table.sql

CREATE TABLE jadwal_ruangan (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    ruangan_id UUID NOT NULL REFERENCES ruangan(id) ON DELETE CASCADE,

    judul_kegiatan VARCHAR(255) NOT NULL, -- Contoh: 'Kuliah IF211', 'Rapat Dosen Bulanan'
    deskripsi TEXT,

    -- Waktu mulai dan selesai yang spesifik untuk instance ini
    waktu_mulai TIMESTAMPTZ NOT NULL,
    waktu_selesai TIMESTAMPTZ NOT NULL,

    -- ID untuk mengelompokkan semua instance dari satu jadwal berulang yang sama
    recurring_event_id UUID,

    -- Siapa yang membuat jadwal ini
    user_pembuat_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index untuk mempercepat pencarian jadwal di satu ruangan pada rentang waktu tertentu
CREATE INDEX idx_jadwal_ruangan_waktu ON jadwal_ruangan (ruangan_id, waktu_mulai, waktu_selesai);
