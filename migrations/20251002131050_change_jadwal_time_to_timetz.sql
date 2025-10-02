-- migrations/YYYY..._change_jadwal_time_to_timetz.sql
ALTER TABLE jadwal_kuliah
ALTER COLUMN jam_mulai TYPE TIME WITH TIME ZONE,
ALTER COLUMN jam_selesai TYPE TIME WITH TIME ZONE;