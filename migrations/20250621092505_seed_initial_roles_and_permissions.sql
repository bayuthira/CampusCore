-- migrations/YYYYMMDDHHMMSS_seed_initial_roles_and_permissions.sql
-- (Ganti isi file migrasi seeding yang LAMA dengan ini)

-- Pertama, masukkan data ke tabel 'roles' dan 'permissions'.
-- Biarkan database yang meng-generate UUID-nya secara otomatis.
INSERT INTO roles (name, description) VALUES
('SUPER_ADMIN', 'Memiliki akses ke semua fitur tanpa batasan.'),
('DOSEN', 'Peran standar untuk dosen.'),
('MAHASISWA', 'Peran standar untuk mahasiswa.'),
('KAPRODI', 'Kepala Program Studi, memiliki hak akses spesifik prodi.'),
('STAF_AKADEMIK', 'Staf di bagian akademik.');

INSERT INTO permissions (name, description) VALUES
('prodi:create', 'Izin untuk membuat program studi baru.'),
('prodi:read', 'Izin untuk melihat data program studi.'),
('prodi:update', 'Izin untuk mengubah data program studi.'),
('prodi:delete', 'Izin untuk menghapus program studi.'),
('dosen:create', 'Izin untuk membuat data dosen baru.'),
('dosen:read', 'Izin untuk melihat data dosen.'),
('dosen:update', 'Izin untuk mengubah data dosen.'),
('dosen:delete', 'Izin untuk menghapus data dosen.'),
('krs:read_own', 'Izin untuk mahasiswa melihat KRS miliknya sendiri.');


-- Kedua, isi tabel pivot 'role_permissions' dengan MENGGUNAKAN SUBQUERY
-- untuk mengambil UUID yang baru saja dibuat berdasarkan nama.
INSERT INTO role_permissions (role_id, permission_id) VALUES
-- SUPER_ADMIN bisa segalanya
((SELECT id FROM roles WHERE name = 'SUPER_ADMIN'), (SELECT id FROM permissions WHERE name = 'prodi:create')),
((SELECT id FROM roles WHERE name = 'SUPER_ADMIN'), (SELECT id FROM permissions WHERE name = 'prodi:read')),
((SELECT id FROM roles WHERE name = 'SUPER_ADMIN'), (SELECT id FROM permissions WHERE name = 'prodi:update')),
((SELECT id FROM roles WHERE name = 'SUPER_ADMIN'), (SELECT id FROM permissions WHERE name = 'prodi:delete')),
((SELECT id FROM roles WHERE name = 'SUPER_ADMIN'), (SELECT id FROM permissions WHERE name = 'dosen:create')),
((SELECT id FROM roles WHERE name = 'SUPER_ADMIN'), (SELECT id FROM permissions WHERE name = 'dosen:read')),
((SELECT id FROM roles WHERE name = 'SUPER_ADMIN'), (SELECT id FROM permissions WHERE name = 'dosen:update')),
((SELECT id FROM roles WHERE name = 'SUPER_ADMIN'), (SELECT id FROM permissions WHERE name = 'dosen:delete')),
((SELECT id FROM roles WHERE name = 'SUPER_ADMIN'), (SELECT id FROM permissions WHERE name = 'krs:read_own')),

-- DOSEN
((SELECT id FROM roles WHERE name = 'DOSEN'), (SELECT id FROM permissions WHERE name = 'prodi:read')),
((SELECT id FROM roles WHERE name = 'DOSEN'), (SELECT id FROM permissions WHERE name = 'dosen:read')),

-- MAHASISWA
((SELECT id FROM roles WHERE name = 'MAHASISWA'), (SELECT id FROM permissions WHERE name = 'krs:read_own'));

-- (Anda bisa menambahkan izin untuk KAPRODI dan STAF_AKADEMIK dengan pola yang sama)