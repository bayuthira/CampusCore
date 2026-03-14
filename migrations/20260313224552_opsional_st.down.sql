-- Mengisi data yang kosong dengan strip '-' sebelum mengembalikan aturan wajib isi
UPDATE surat_tugas_master SET nomor_surat = '-' WHERE nomor_surat IS NULL;
ALTER TABLE surat_tugas_master ALTER COLUMN nomor_surat SET NOT NULL;