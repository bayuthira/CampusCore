-- Hapus Tabel Baru
DROP TABLE IF EXISTS skala_nilai;
DROP TABLE IF EXISTS aktivitas_kuliah_mahasiswa;

-- Rollback Tabel Jadwal Dosen Pengampu
ALTER TABLE jadwal_dosen_pengampu DROP COLUMN id_aktivitas_mengajar_feeder;
ALTER TABLE jadwal_dosen_pengampu DROP COLUMN sks_substansi_total;
ALTER TABLE jadwal_dosen_pengampu DROP COLUMN rencana_tatap_muka;
ALTER TABLE jadwal_dosen_pengampu DROP COLUMN realisasi_tatap_muka;

-- Rollback Tabel Enrollments
ALTER TABLE enrollments DROP COLUMN id_peserta_kelas_feeder;
ALTER TABLE enrollments DROP COLUMN id_nilai_feeder;
ALTER TABLE enrollments DROP COLUMN nilai_angka;
ALTER TABLE enrollments DROP COLUMN nilai_indeks;

-- Rollback Tabel Jadwal Kuliah
ALTER TABLE jadwal_kuliah DROP COLUMN id_kelas_kuliah_feeder;
ALTER TABLE jadwal_kuliah DROP COLUMN nama_kelas_kuliah;