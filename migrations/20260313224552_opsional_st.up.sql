-- Membuat kolom nomor_surat menjadi opsional (boleh NULL)
ALTER TABLE surat_tugas_master ALTER COLUMN nomor_surat DROP NOT NULL;