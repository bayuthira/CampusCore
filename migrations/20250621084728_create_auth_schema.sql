-- migrations/YYYYMMDDHHMMSS_create_auth_schema.sql

-- Tabel 1: Pengguna Sistem (Users)
-- Tabel ini menyimpan informasi login dasar untuk SEMUA orang yang bisa masuk ke sistem.
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Bisa NIDN untuk dosen, NIM untuk mahasiswa, atau ID Pegawai untuk staf
    username VARCHAR(50) UNIQUE NOT NULL,
    
    -- SANGAT PENTING: Jangan pernah simpan password asli. Simpan HASH-nya.
    -- Kita akan menggunakan library seperti `bcrypt` atau `argon2` di Rust nanti.
    password_hash VARCHAR(255) NOT NULL,
    
    full_name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT true, -- Untuk menonaktifkan user tanpa menghapusnya
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tabel 2: Peran (Roles)
-- Daftar semua peran yang mungkin ada di dalam sistem.
CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL, -- Contoh: 'SUPER_ADMIN', 'DOSEN', 'MAHASISWA', 'KAPRODI'
    description TEXT
);

-- Tabel 3: Izin (Permissions)
-- Daftar semua aksi spesifik yang bisa dilakukan.
-- Pola 'resource:action' adalah praktik yang baik.
CREATE TABLE permissions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL, -- Contoh: 'dosen:create', 'dosen:read', 'nilai:update', 'krs:read_own'
    description TEXT
);

-- Tabel 4: Tabel Pivot User-Roles (Many-to-Many)
-- Tabel ini adalah jantung dari sistem multi-role.
-- Menghubungkan seorang user dengan satu atau lebih role.
CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id INT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    
    -- KOLOM AJAIB: untuk konteks yang dinamis (Attribute-Based)
    -- Contoh: Seorang dosen menjadi Kaprodi Informatika.
    -- user_id = (ID dosen), role_id = (ID Kaprodi), context = {"prodi_id": "uuid-prodi-informatika"}
    context JSONB,
    
    -- Setiap user hanya bisa punya satu role yang sama (misal: tidak bisa jadi DOSEN dua kali)
    PRIMARY KEY (user_id, role_id)
);

-- Tabel 5: Tabel Pivot Role-Permissions (Many-to-Many)
-- Tabel ini mendefinisikan apa saja yang bisa dilakukan oleh sebuah role.
CREATE TABLE role_permissions (
    role_id INT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    
    PRIMARY KEY (role_id, permission_id)
);

-- Membuat beberapa index untuk mempercepat query
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_role_permissions_role_id ON role_permissions(role_id);