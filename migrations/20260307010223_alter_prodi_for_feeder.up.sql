-- Menambahkan ID unik dari Neo Feeder PDDIKTI
-- Dibuat UNIQUE agar tidak ada prodi ganda saat sinkronisasi
ALTER TABLE prodi 
ADD COLUMN id_prodi_feeder UUID UNIQUE;

-- Menambahkan jenjang pendidikan (S1, D3, Profesi, S2, dll)
ALTER TABLE prodi 
ADD COLUMN jenjang VARCHAR(10);

-- Menambahkan status prodi (Aktif, Pembinaan, Tutup)
-- Diberi DEFAULT 'Aktif' agar data prodi yang sudah ada di database saat ini tidak bernilai NULL
ALTER TABLE prodi 
ADD COLUMN status_prodi VARCHAR(20) DEFAULT 'Aktif';