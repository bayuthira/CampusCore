ALTER TABLE log_absensi 
ADD COLUMN foto_absensi_path VARCHAR(255), -- Path file foto selfie saat absen
ADD COLUMN face_confidence_score REAL,     -- Nilai kemiripan dari Azure (misal: 0.98)
ADD COLUMN is_face_verified BOOLEAN DEFAULT FALSE; -- True jika wajah cocok