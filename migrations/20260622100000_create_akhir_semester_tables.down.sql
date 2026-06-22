DROP TABLE IF EXISTS feeder_sync_outbox;
DROP TYPE IF EXISTS "StatusSinkronisasiFeeder";
DROP TABLE IF EXISTS koreksi_nilai;
DROP TYPE IF EXISTS "StatusKoreksiNilai";
ALTER TABLE tahun_akademik
    DROP COLUMN IF EXISTS ditutup_pada,
    DROP COLUMN IF EXISTS ditutup_oleh,
    DROP COLUMN IF EXISTS status_penutupan;
