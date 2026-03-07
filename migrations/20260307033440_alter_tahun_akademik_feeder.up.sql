-- Menambahkan ID Semester baku dari PDDIKTI Neo Feeder
-- Format: 5 Digit (Contoh: '20231' untuk Ganjil 2023, '20232' untuk Genap 2023)
-- Kita buat UNIQUE agar tidak ada duplikasi ID Semester.
ALTER TABLE tahun_akademik 
ADD COLUMN id_semester_feeder VARCHAR(5) UNIQUE;