CREATE TYPE "JenisAsesmenKuliah" AS ENUM ('Kuis', 'Tugas', 'UTS', 'UAS', 'Praktik');
CREATE TYPE "ModeAsesmenKuliah" AS ENUM ('Manual', 'Online');
CREATE TYPE "StatusAsesmenKuliah" AS ENUM (
    'Draft', 'Diajukan', 'PerluRevisi', 'Disetujui', 'SiapDilaksanakan',
    'Berlangsung', 'Selesai', 'Dinilai', 'Dikunci', 'Dibatalkan'
);
CREATE TYPE "JenisDokumenAsesmen" AS ENUM ('Soal', 'Lampiran', 'KunciJawaban');

CREATE TABLE asesmen_kuliah (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jadwal_kuliah_id UUID NOT NULL REFERENCES jadwal_kuliah(id) ON DELETE CASCADE,
    pertemuan_kuliah_id UUID UNIQUE REFERENCES pertemuan_kuliah(id) ON DELETE SET NULL,
    jenis "JenisAsesmenKuliah" NOT NULL,
    judul VARCHAR(255) NOT NULL,
    mode "ModeAsesmenKuliah" NOT NULL,
    bobot NUMERIC(5,2) NOT NULL CHECK (bobot >= 0 AND bobot <= 100),
    durasi_menit INTEGER NOT NULL CHECK (durasi_menit > 0),
    mulai_terjadwal TIMESTAMPTZ NOT NULL,
    selesai_terjadwal TIMESTAMPTZ NOT NULL,
    online_url TEXT,
    instruksi TEXT,
    sifat_ujian VARCHAR(50),
    hitung_sebagai_pertemuan BOOLEAN NOT NULL DEFAULT true,
    status "StatusAsesmenKuliah" NOT NULL DEFAULT 'Draft',
    dibuat_oleh UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    diajukan_pada TIMESTAMPTZ,
    disetujui_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    disetujui_pada TIMESTAMPTZ,
    catatan_review TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (selesai_terjadwal > mulai_terjadwal),
    CHECK (mode <> 'Online' OR online_url IS NOT NULL)
);

CREATE TABLE dokumen_asesmen (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asesmen_id UUID NOT NULL REFERENCES asesmen_kuliah(id) ON DELETE CASCADE,
    jenis "JenisDokumenAsesmen" NOT NULL,
    versi INTEGER NOT NULL CHECK (versi > 0),
    nama_file_asli VARCHAR(255) NOT NULL,
    path_file TEXT NOT NULL,
    mime_type VARCHAR(150) NOT NULL,
    ukuran_bytes BIGINT NOT NULL,
    diunggah_oleh UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (asesmen_id, jenis, versi)
);

CREATE TABLE audit_dokumen_asesmen (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dokumen_id UUID NOT NULL REFERENCES dokumen_asesmen(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    aksi VARCHAR(30) NOT NULL DEFAULT 'Download',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE review_asesmen (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asesmen_id UUID NOT NULL REFERENCES asesmen_kuliah(id) ON DELETE CASCADE,
    aksi VARCHAR(30) NOT NULL CHECK (aksi IN ('Disetujui', 'PerluRevisi')),
    catatan TEXT,
    reviewer_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE penggandaan_asesmen (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asesmen_id UUID NOT NULL UNIQUE REFERENCES asesmen_kuliah(id) ON DELETE CASCADE,
    jumlah_utama INTEGER NOT NULL CHECK (jumlah_utama >= 0),
    jumlah_cadangan INTEGER NOT NULL DEFAULT 0 CHECK (jumlah_cadangan >= 0),
    status VARCHAR(30) NOT NULL DEFAULT 'Diajukan'
        CHECK (status IN ('Diajukan', 'Diproses', 'Selesai', 'Diserahkan')),
    catatan TEXT,
    diproses_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    diserahkan_pada TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE pelaksanaan_asesmen (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asesmen_id UUID NOT NULL UNIQUE REFERENCES asesmen_kuliah(id) ON DELETE CASCADE,
    pengawas_user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    mulai_aktual TIMESTAMPTZ NOT NULL DEFAULT now(),
    selesai_aktual TIMESTAMPTZ,
    versi_soal VARCHAR(50),
    jumlah_lembar_diterima INTEGER CHECK (jumlah_lembar_diterima >= 0),
    bap TEXT,
    insiden TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE sesi_presensi_asesmen (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asesmen_id UUID NOT NULL REFERENCES asesmen_kuliah(id) ON DELETE CASCADE,
    kode VARCHAR(8) NOT NULL UNIQUE,
    berlaku_sampai TIMESTAMPTZ NOT NULL,
    aktif BOOLEAN NOT NULL DEFAULT true,
    dibuat_oleh UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE presensi_asesmen (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asesmen_id UUID NOT NULL REFERENCES asesmen_kuliah(id) ON DELETE CASCADE,
    enrollment_id UUID NOT NULL REFERENCES enrollments(id) ON DELETE CASCADE,
    status "StatusPresensiMahasiswa" NOT NULL DEFAULT 'Hadir',
    check_in_at TIMESTAMPTZ,
    sumber "SumberPresensiKuliah" NOT NULL,
    catatan TEXT,
    dicatat_oleh UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (asesmen_id, enrollment_id)
);

CREATE TABLE nilai_asesmen (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asesmen_id UUID NOT NULL REFERENCES asesmen_kuliah(id) ON DELETE CASCADE,
    enrollment_id UUID NOT NULL REFERENCES enrollments(id) ON DELETE CASCADE,
    attempt INTEGER NOT NULL DEFAULT 1 CHECK (attempt > 0),
    nilai NUMERIC(5,2) NOT NULL CHECK (nilai >= 0 AND nilai <= 100),
    umpan_balik TEXT,
    dinilai_oleh UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (asesmen_id, enrollment_id, attempt)
);

CREATE INDEX idx_asesmen_jadwal ON asesmen_kuliah(jadwal_kuliah_id);
CREATE INDEX idx_asesmen_status ON asesmen_kuliah(status);
CREATE INDEX idx_dokumen_asesmen ON dokumen_asesmen(asesmen_id);
CREATE INDEX idx_audit_dokumen_asesmen ON audit_dokumen_asesmen(dokumen_id, created_at);
CREATE INDEX idx_presensi_asesmen ON presensi_asesmen(asesmen_id);
CREATE INDEX idx_nilai_asesmen ON nilai_asesmen(asesmen_id, enrollment_id);
