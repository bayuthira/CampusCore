-- =========================================================================
-- 1. ALTER TABEL JADWAL KULIAH (KELAS KULIAH)
-- =========================================================================
ALTER TABLE jadwal_kuliah ADD COLUMN id_kelas_kuliah_feeder UUID UNIQUE;
-- Feeder mewajibkan nama kelas spesifik, kita tambahkan jika butuh format beda dari kolom 'kelas' saat ini
ALTER TABLE jadwal_kuliah ADD COLUMN nama_kelas_kuliah VARCHAR(50);
-- Kita migrasikan sementara data dari kolom kelas yang sudah ada
UPDATE jadwal_kuliah SET nama_kelas_kuliah = kelas;

-- =========================================================================
-- 2. ALTER TABEL ENROLLMENTS (KRS & NILAI)
-- =========================================================================
ALTER TABLE enrollments ADD COLUMN id_peserta_kelas_feeder UUID UNIQUE;
ALTER TABLE enrollments ADD COLUMN id_nilai_feeder UUID UNIQUE;
ALTER TABLE enrollments ADD COLUMN nilai_angka NUMERIC(5,2); -- Contoh: 85.50
ALTER TABLE enrollments ADD COLUMN nilai_indeks NUMERIC(3,2); -- Contoh: 3.50 (A-)

-- =========================================================================
-- 3. ALTER TABEL JADWAL DOSEN PENGAMPU (AKTIVITAS MENGAJAR)
-- =========================================================================
ALTER TABLE jadwal_dosen_pengampu ADD COLUMN id_aktivitas_mengajar_feeder UUID UNIQUE;
ALTER TABLE jadwal_dosen_pengampu ADD COLUMN sks_substansi_total INTEGER DEFAULT 0;
ALTER TABLE jadwal_dosen_pengampu ADD COLUMN rencana_tatap_muka INTEGER DEFAULT 16;
ALTER TABLE jadwal_dosen_pengampu ADD COLUMN realisasi_tatap_muka INTEGER DEFAULT 0;

-- =========================================================================
-- 4. CREATE TABEL BARU: AKTIVITAS KULIAH MAHASISWA (AKM)
-- =========================================================================
CREATE TABLE aktivitas_kuliah_mahasiswa (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registrasi_id UUID NOT NULL REFERENCES registrasi_mahasiswa(id) ON DELETE CASCADE,
    tahun_akademik_id UUID NOT NULL REFERENCES tahun_akademik(id) ON DELETE CASCADE,
    id_aktivitas_kuliah_feeder UUID UNIQUE,
    status_mahasiswa VARCHAR(50) NOT NULL DEFAULT 'Aktif', -- Aktif, Cuti, Lulus, Mutasi, dll
    ips NUMERIC(3,2) DEFAULT 0.00,
    ipk NUMERIC(3,2) DEFAULT 0.00,
    sks_semester INTEGER DEFAULT 0,
    sks_total INTEGER DEFAULT 0,
    biaya_kuliah_smt NUMERIC(15,2) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    UNIQUE(registrasi_id, tahun_akademik_id) -- Mahasiswa hanya punya 1 AKM per semester
);

-- =========================================================================
-- 5. CREATE TABEL BARU: SKALA NILAI (REFERENSI NILAI FEEDER)
-- =========================================================================
CREATE TABLE skala_nilai (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prodi_id UUID NOT NULL REFERENCES prodi(id) ON DELETE CASCADE,
    id_bobot_nilai_feeder UUID UNIQUE,
    nilai_huruf VARCHAR(5) NOT NULL, -- A, B+, C, dll
    nilai_indeks NUMERIC(3,2) NOT NULL, -- 4.00, 3.50, 2.00, dll
    bobot_minimum NUMERIC(5,2) NOT NULL, -- Batas bawah, misal 80.00
    bobot_maksimum NUMERIC(5,2) NOT NULL, -- Batas atas, misal 100.00
    tanggal_mulai_efektif DATE NOT NULL,
    tanggal_akhir_efektif DATE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);