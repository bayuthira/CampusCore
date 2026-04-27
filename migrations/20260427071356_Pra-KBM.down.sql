-- =========================================================================
-- 1. HAPUS TABEL RENCANA PENILAIAN
-- =========================================================================
DROP TABLE IF EXISTS jadwal_rencana_penilaian;

-- =========================================================================
-- 2. KEMBALIKAN TABEL MATA KULIAH (HAPUS KOLOM RPS)
-- =========================================================================
ALTER TABLE mata_kuliah DROP COLUMN file_rps_path;