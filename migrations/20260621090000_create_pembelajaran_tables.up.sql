CREATE TYPE "StatusPertemuanKuliah" AS ENUM (
    'Dijadwalkan', 'Dibuka', 'Ditutup', 'Dibatalkan'
);

CREATE TYPE "StatusPresensiDosenKuliah" AS ENUM (
    'Hadir', 'Pengganti', 'Izin', 'TidakHadir'
);

CREATE TYPE "StatusPresensiMahasiswa" AS ENUM (
    'Hadir', 'Terlambat', 'Izin', 'Sakit', 'Alpa'
);

CREATE TYPE "SumberPresensiKuliah" AS ENUM (
    'KodeDinamis', 'ManualDosen', 'Admin', 'Sistem'
);

CREATE TABLE pertemuan_kuliah (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jadwal_kuliah_id UUID NOT NULL REFERENCES jadwal_kuliah(id) ON DELETE CASCADE,
    rps_mingguan_id UUID REFERENCES mata_kuliah_rps_mingguan(id) ON DELETE SET NULL,
    pertemuan_ke INTEGER NOT NULL CHECK (pertemuan_ke BETWEEN 1 AND 32),
    tanggal DATE NOT NULL,
    topik_rencana TEXT,
    topik_realisasi TEXT,
    metode_pembelajaran TEXT,
    bap TEXT,
    status "StatusPertemuanKuliah" NOT NULL DEFAULT 'Dijadwalkan',
    dibuka_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    dibuka_pada TIMESTAMPTZ,
    ditutup_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    ditutup_pada TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (jadwal_kuliah_id, pertemuan_ke)
);

CREATE TABLE presensi_dosen_kuliah (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pertemuan_id UUID NOT NULL REFERENCES pertemuan_kuliah(id) ON DELETE CASCADE,
    dosen_id UUID NOT NULL REFERENCES dosen(id) ON DELETE RESTRICT,
    status "StatusPresensiDosenKuliah" NOT NULL DEFAULT 'Hadir',
    check_in_at TIMESTAMPTZ,
    check_out_at TIMESTAMPTZ,
    sumber "SumberPresensiKuliah" NOT NULL DEFAULT 'Sistem',
    catatan TEXT,
    dicatat_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (pertemuan_id, dosen_id)
);

CREATE TABLE sesi_presensi_mahasiswa (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pertemuan_id UUID NOT NULL REFERENCES pertemuan_kuliah(id) ON DELETE CASCADE,
    kode VARCHAR(8) NOT NULL UNIQUE,
    berlaku_sampai TIMESTAMPTZ NOT NULL,
    aktif BOOLEAN NOT NULL DEFAULT true,
    dibuat_oleh UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE presensi_mahasiswa_kuliah (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pertemuan_id UUID NOT NULL REFERENCES pertemuan_kuliah(id) ON DELETE CASCADE,
    enrollment_id UUID NOT NULL REFERENCES enrollments(id) ON DELETE CASCADE,
    status "StatusPresensiMahasiswa" NOT NULL DEFAULT 'Hadir',
    check_in_at TIMESTAMPTZ,
    sumber "SumberPresensiKuliah" NOT NULL,
    catatan TEXT,
    dicatat_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (pertemuan_id, enrollment_id)
);

CREATE INDEX idx_pertemuan_jadwal ON pertemuan_kuliah(jadwal_kuliah_id);
CREATE INDEX idx_presensi_dosen_pertemuan ON presensi_dosen_kuliah(pertemuan_id);
CREATE INDEX idx_presensi_mahasiswa_pertemuan ON presensi_mahasiswa_kuliah(pertemuan_id);
CREATE INDEX idx_presensi_mahasiswa_enrollment ON presensi_mahasiswa_kuliah(enrollment_id);

