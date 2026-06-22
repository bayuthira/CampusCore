# Campus Core - Sistem Informasi Kampus Terpadu (API Documentation)

Backend API untuk Sistem Informasi Kampus Terpadu, dibangun dengan **Rust + Axum + PostgreSQL**.

**Base URL:** `http://localhost:8080/api`  
**Production:** `https://satria.respati-tasikmalaya.ac.id/api`

---

## Daftar Isi

1. [Autentikasi & Otorisasi](#1-autentikasi--otorisasi)
2. [Format Respons Error](#2-format-respons-error)
3. [Roles (Peran Pengguna)](#3-roles-peran-pengguna)
4. [API Endpoints](#4-api-endpoints)
   - [Auth](#41-auth)
   - [User Management](#42-user-management)
   - [Prodi](#43-prodi)
   - [Dosen](#44-dosen)
   - [Mahasiswa](#45-mahasiswa)
   - [Mata Kuliah & RPS](#46-mata-kuliah--rps)
   - [Tahun Akademik](#47-tahun-akademik)
   - [Kurikulum](#48-kurikulum)
   - [KRS (Kartu Rencana Studi)](#49-krs-kartu-rencana-studi)
   - [Dosen PA (Pembimbing Akademik)](#410-dosen-pa)
   - [Akademik - Jadwal Kuliah](#411-akademik---jadwal-kuliah)
   - [Akademik - Rencana Penilaian](#412-akademik---rencana-penilaian)
   - [Aset - Jenis Aset](#413-aset---jenis-aset)
   - [Aset - Ruangan](#414-aset---ruangan)
   - [Aset - Item](#415-aset---item)
   - [Aset - Jadwal Ruangan](#416-aset---jadwal-ruangan)
   - [Aset - Biaya](#417-aset---biaya)
   - [Aset - Habis Pakai (Konsumsi)](#418-aset---habis-pakai-konsumsi)
   - [Fleet - Kendaraan](#419-fleet---kendaraan)
   - [Fleet - Booking](#420-fleet---booking)
   - [Fleet - Servis](#421-fleet---servis)
   - [SDM - Pegawai](#422-sdm---pegawai)
   - [SDM - Absensi](#423-sdm---absensi)
   - [SDM - Cuti](#424-sdm---cuti)
   - [SDM - Ijin](#425-sdm---ijin)
   - [SDM - Surat Tugas](#426-sdm---surat-tugas)
   - [SDM - Unit Kerja](#427-sdm---unit-kerja)
   - [SDM - Penempatan](#428-sdm---penempatan)
   - [SDM - Dokumen](#429-sdm---dokumen)
   - [SDM - Riwayat](#430-sdm---riwayat)
   - [Lookup](#431-lookup)
   - [Files](#432-files)
   - [Rombel (Rombongan Belajar)](#433-rombel)
   - [Ujian, Asesmen, dan Nilai Akhir](#434-ujian-asesmen-dan-nilai-akhir)
5. [Alur Penggunaan Sistem](#5-alur-penggunaan-sistem)

---


## 1. Autentikasi & Otorisasi

Sistem menggunakan **JWT (JSON Web Token)** dengan skema **Bearer Token**.

### Alur Login:
1. Frontend mengirim `POST /api/auth/login` dengan `username` dan `password`
2. Backend memvalidasi kredensial dan mengembalikan `token` + data user
3. Frontend menyimpan token (localStorage/cookie)
4. Untuk setiap request ke endpoint protected, sertakan header:

```
Authorization: Bearer <token_jwt>
```

### Token Claims (Payload JWT):
```json
{
  "sub": "uuid-user-id",
  "roles": ["SUPER_ADMIN", "DOSEN"],
  "iat": 1700000000,
  "exp": 1700086400
}
```

---

## 2. Format Respons Error

Semua error dikembalikan dalam format JSON konsisten:

```json
{
  "error": "Pesan error yang bisa ditampilkan ke user"
}
```

### HTTP Status Codes:
| Code | Keterangan |
|------|-----------|
| `200` | OK - Berhasil |
| `201` | Created - Data berhasil dibuat |
| `204` | No Content - Berhasil hapus |
| `400` | Bad Request - Input tidak valid |
| `401` | Unauthorized - Token tidak ada/expired |
| `403` | Forbidden - Tidak punya izin |
| `409` | Conflict - Data duplikat |
| `500` | Internal Server Error |

---

## 3. Roles (Peran Pengguna)

| Role | Deskripsi |
|------|-----------|
| `SUPER_ADMIN` | Akses penuh ke seluruh sistem |
| `STAF_AKADEMIK` | Kelola data akademik (mahasiswa, jadwal) |
| `STAF_BAUM` | Kelola aset & fleet management |
| `STAF_BASDM` | Kelola SDM & kepegawaian |
| `KAPRODI` | Ketua Program Studi |
| `DOSEN` | Akses fitur dosen (KRS approval, nilai) |
| `MAHASISWA` | Akses fitur mahasiswa (KRS, jadwal) |
| `KARYAWAN` | Akses fitur pegawai (absensi, cuti, ijin) |

---


## 4. API Endpoints

### 4.1 Auth

#### `POST /api/auth/login` *(Public - Tidak perlu token)*

Login dan mendapatkan JWT token.

**Request Body:**
```json
{
  "username": "admin",
  "password": "password123"
}
```

**Response (200 OK):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "admin",
    "full_name": "Administrator",
    "roles": ["SUPER_ADMIN"]
  }
}
```

**Error (401):**
```json
{
  "error": "Username atau password salah"
}
```

---

### 4.2 User Management

> **Roles:** `SUPER_ADMIN`

#### `GET /api/users`

Mendapatkan semua user beserta perannya.

**Response (200):**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "full_name": "Administrator",
    "username": "admin",
    "email": "admin@kampus.ac.id",
    "is_active": true,
    "created_at": "2025-06-20T10:00:00+07:00",
    "roles": ["SUPER_ADMIN"]
  }
]
```


#### `POST /api/users`

Membuat user baru.

**Request Body:**
```json
{
  "username": "dosen01",
  "full_name": "Dr. Budi Santoso",
  "email": "budi@kampus.ac.id",
  "password": "password123",
  "role_ids": ["uuid-role-dosen"]
}
```

**Response (201):**
```json
{
  "id": "uuid-new-user",
  "full_name": "Dr. Budi Santoso",
  "username": "dosen01",
  "email": "budi@kampus.ac.id",
  "is_active": true,
  "created_at": "2025-06-21T10:00:00+07:00",
  "roles": ["DOSEN"]
}
```

#### `GET /api/users/{id}`

Mendapatkan detail satu user.

#### `PUT /api/users/{id}`

Update data user.

**Request Body:**
```json
{
  "full_name": "Dr. Budi Santoso, M.Kom",
  "email": "budi.s@kampus.ac.id",
  "is_active": true,
  "role_ids": ["uuid-role-dosen", "uuid-role-kaprodi"]
}
```

#### `DELETE /api/users/{id}`

Hapus user. **Response:** `204 No Content`

#### `PUT /api/users/{id}/reset-password`

Reset password user.

**Request Body:**
```json
{
  "new_password": "newpassword123"
}
```

**Response (200):**
```json
{
  "message": "Password user berhasil direset."
}
```

#### `POST /api/users/assign-role`

Memberikan peran ke user.

**Request Body:**
```json
{
  "user_id": "uuid-user",
  "role_id": "uuid-role"
}
```

#### `DELETE /api/users/revoke-role`

Mencabut peran dari user.

**Request Body:**
```json
{
  "user_id": "uuid-user",
  "role_id": "uuid-role"
}
```

#### `GET /api/roles`

Mendapatkan daftar semua peran.

**Response (200):**
```json
[
  { "id": "uuid-role", "name": "SUPER_ADMIN" },
  { "id": "uuid-role", "name": "DOSEN" },
  { "id": "uuid-role", "name": "MAHASISWA" }
]
```

---


### 4.3 Prodi

#### `GET /api/prodi` *(Semua user login)*

Mendapatkan semua program studi.

**Response (200):**
```json
[
  {
    "id": "uuid-prodi",
    "kode_prodi": "S1TI",
    "nama_prodi": "Teknik Informatika",
    "id_prodi_feeder": null,
    "jenjang": "S1",
    "status_prodi": "Aktif",
    "created_at": "2025-06-20T10:00:00+07:00",
    "updated_at": "2025-06-20T10:00:00+07:00"
  }
]
```

#### `GET /api/prodi/{id}` *(Semua user login)*

#### `POST /api/prodi` *(SUPER_ADMIN)*

**Request Body:**
```json
{
  "kode_prodi": "S1TI",
  "nama_prodi": "Teknik Informatika",
  "id_prodi_feeder": null,
  "jenjang": "S1",
  "status_prodi": "Aktif"
}
```

#### `PUT /api/prodi/{id}` *(SUPER_ADMIN)*

**Request Body (partial update):**
```json
{
  "nama_prodi": "Teknik Informatika (Updated)",
  "jenjang": "S1"
}
```

#### `DELETE /api/prodi/{id}` *(SUPER_ADMIN)*

**Response:** `204 No Content`

---

### 4.4 Dosen

#### `GET /api/dosen` *(Semua user login)*

**Response (200):**
```json
[
  {
    "id": "uuid-dosen",
    "nidn": "0412098901",
    "nama_dosen": "Dr. Budi Santoso, M.Kom",
    "email": "budi@kampus.ac.id",
    "prodi_id": "uuid-prodi",
    "nama_prodi": "Teknik Informatika",
    "pegawai_id": "uuid-pegawai",
    "id_penugasan_feeder": null,
    "ikatan_kerja": "Tetap"
  }
]
```

#### `GET /api/dosen/{id}` *(Semua user login)*

#### `POST /api/dosen` *(SUPER_ADMIN)*

**Request Body:**
```json
{
  "nidn": "0412098901",
  "pegawai_id": "uuid-pegawai",
  "prodi_id": "uuid-prodi",
  "id_penugasan_feeder": null,
  "ikatan_kerja": "Tetap"
}
```

#### `PUT /api/dosen/{id}` *(SUPER_ADMIN)*

**Request Body:**
```json
{
  "nidn": "0412098902",
  "prodi_id": "uuid-prodi-baru"
}
```

#### `DELETE /api/dosen/{id}` *(SUPER_ADMIN)*

---


### 4.5 Mahasiswa

#### `GET /api/mahasiswa` *(SUPER_ADMIN, STAF_AKADEMIK, DOSEN)*

**Response (200):**
```json
[
  {
    "id": "uuid-mahasiswa",
    "registrasi_id": "uuid-registrasi",
    "nik": "3302501010011",
    "nama_mahasiswa": "Ahmad Fauzi",
    "email": "ahmad@student.kampus.ac.id",
    "nim": "250101001",
    "angkatan": 2025,
    "prodi_id": "uuid-prodi",
    "nama_prodi": "Teknik Informatika",
    "status_mahasiswa": "Aktif",
    "dosen_pa_id": "uuid-dosen",
    "nama_dosen_pa": "Dr. Budi Santoso",
    "user_id": "uuid-user",
    "username": "250101001"
  }
]
```

#### `GET /api/mahasiswa/{id}` *(SUPER_ADMIN, STAF_AKADEMIK)*

#### `POST /api/mahasiswa` *(SUPER_ADMIN, STAF_AKADEMIK)*

Membuat mahasiswa baru (otomatis membuat akun user).

**Request Body:**
```json
{
  "nik": "3302501010011",
  "nama_mahasiswa": "Ahmad Fauzi",
  "email": "ahmad@student.kampus.ac.id",
  "tempat_lahir": "Tasikmalaya",
  "tanggal_lahir": "2003-05-15",
  "nama_ibu_kandung": "Siti Nurhasanah",
  "nim": "250101001",
  "angkatan": 2025,
  "prodi_id": "uuid-prodi",
  "periode_masuk": "20251",
  "password": "password123"
}
```

#### `PUT /api/mahasiswa/{id}` *(SUPER_ADMIN, STAF_AKADEMIK)*

**Request Body (partial update):**
```json
{
  "nama_mahasiswa": "Ahmad Fauzi Updated",
  "email": "ahmad.new@student.kampus.ac.id",
  "status_mahasiswa": "Cuti"
}
```

> **Catatan:** Hanya `SUPER_ADMIN` yang bisa mengubah `nim`.

#### `DELETE /api/mahasiswa/{id}` *(SUPER_ADMIN, STAF_AKADEMIK)*

#### `GET /api/mahasiswa/template-csv`

Download template CSV untuk import mahasiswa. **Response:** file CSV.

#### `POST /api/mahasiswa/import-csv` *(SUPER_ADMIN, STAF_AKADEMIK)*

Import mahasiswa dari file CSV.

**Request:** `multipart/form-data` dengan field `file` (CSV).

**Format CSV (delimiter `;`):**
```
nik;nim;nama_mahasiswa;email;angkatan;kode_prodi
330250101001105;250101001;Budi Darmawan;budi.d@student.kampus.ac.id;2025;S1TI
```

**Response (200):**
```json
{
  "status": "SUKSES",
  "total_baris_dipindai": 10,
  "baris_berhasil_disimpan": 10,
  "detail_error": []
}
```

---


### 4.6 Mata Kuliah & RPS

#### `GET /api/matakuliah` *(Semua user login)*

**Response (200):**
```json
[
  {
    "id": "uuid-mk",
    "kode_mk": "IF101",
    "nama_mk": "Algoritma dan Pemrograman",
    "sks": 4,
    "semester_target": 1,
    "prodi_id": "uuid-prodi",
    "nama_prodi": "Teknik Informatika",
    "id_matkul_feeder": null,
    "sks_tatap_muka": 2,
    "sks_praktek": 2,
    "sks_praktek_lapangan": 0,
    "sks_simulasi": 0,
    "jenis_mk": "Wajib",
    "file_rps_path": null,
    "status_verifikasi_rps": "Belum Upload",
    "catatan_verifikasi_rps": null
  }
]
```

#### `GET /api/matakuliah/{id}` *(Semua user login)*

#### `POST /api/matakuliah` *(SUPER_ADMIN, KAPRODI)*

**Request Body:**
```json
{
  "kode_mk": "IF101",
  "nama_mk": "Algoritma dan Pemrograman",
  "semester_target": 1,
  "prodi_id": "uuid-prodi",
  "sks_tatap_muka": 2,
  "sks_praktek": 2,
  "sks_praktek_lapangan": 0,
  "sks_simulasi": 0,
  "jenis_mk": "Wajib"
}
```

#### `PUT /api/matakuliah/{id}` *(SUPER_ADMIN, KAPRODI)*

> **Catatan:** Hanya `SUPER_ADMIN` yang bisa mengubah `kode_mk`.

#### `DELETE /api/matakuliah/{id}` *(SUPER_ADMIN, KAPRODI)*

#### `POST /api/matakuliah/{id}/upload-rps` *(SUPER_ADMIN, KAPRODI, DOSEN)*

Upload file RPS (PDF/DOC). **Request:** `multipart/form-data` dengan field `file`.

**Response (200):**
```json
{
  "message": "File RPS berhasil diunggah. Menunggu verifikasi Kaprodi."
}
```

#### `PUT /api/matakuliah/{id}/verifikasi-rps` *(SUPER_ADMIN, KAPRODI)*

**Request Body:**
```json
{
  "status_verifikasi": "Disetujui",
  "catatan": "RPS sudah lengkap dan sesuai"
}
```

Status: `"Disetujui"` | `"Ditolak"`

#### RPS Header

##### `GET /api/matakuliah/{id}/rps-header` *(SUPER_ADMIN, KAPRODI, DOSEN)*

**Response (200):**
```json
{
  "mata_kuliah_id": "uuid-mk",
  "deskripsi_singkat": "Pengenalan konsep dasar algoritma...",
  "capaian_pembelajaran": "Mahasiswa mampu...",
  "pustaka_utama": "Cormen et al. Introduction to Algorithms",
  "pustaka_pendukung": "Sedgewick, Algorithms",
  "matakuliah_syarat": null,
  "created_at": "2025-07-01T10:00:00+07:00",
  "updated_at": "2025-07-01T10:00:00+07:00"
}
```

##### `PUT /api/matakuliah/{id}/rps-header` *(SUPER_ADMIN, KAPRODI, DOSEN)*

**Request Body:**
```json
{
  "deskripsi_singkat": "Pengenalan konsep dasar algoritma...",
  "capaian_pembelajaran": "Mahasiswa mampu...",
  "pustaka_utama": "Cormen et al.",
  "pustaka_pendukung": "Sedgewick",
  "matakuliah_syarat": null
}
```

#### RPS Mingguan

##### `GET /api/matakuliah/{id}/rps-mingguan` *(SUPER_ADMIN, KAPRODI, DOSEN)*

**Response (200):**
```json
[
  {
    "id": "uuid",
    "mata_kuliah_id": "uuid-mk",
    "minggu_ke": 1,
    "kemampuan_akhir_diharapkan": "Memahami konsep dasar...",
    "bahan_kajian": "Pengenalan Algoritma",
    "metode_pembelajaran": "Ceramah + Diskusi",
    "waktu_belajar": "3 x 50 menit",
    "kriteria_penilaian": "Quiz & Tugas",
    "bobot_penilaian": "5.00"
  }
]
```

##### `POST /api/matakuliah/{id}/rps-mingguan` *(SUPER_ADMIN, KAPRODI, DOSEN)*

**Request Body:**
```json
{
  "minggu_ke": 1,
  "kemampuan_akhir_diharapkan": "Memahami konsep dasar...",
  "bahan_kajian": "Pengenalan Algoritma",
  "metode_pembelajaran": "Ceramah + Diskusi",
  "waktu_belajar": "3 x 50 menit",
  "kriteria_penilaian": "Quiz & Tugas",
  "bobot_penilaian": "5.00"
}
```

##### `DELETE /api/matakuliah/rps-mingguan/{id_mingguan}` *(SUPER_ADMIN, KAPRODI, DOSEN)*

#### `GET /api/matakuliah/{id}/rps/print` *(Semua user login)*

Mendapatkan halaman HTML cetak RPS (untuk print/PDF).

---


### 4.7 Tahun Akademik

#### `GET /api/tahun-akademik` *(SUPER_ADMIN, STAF_AKADEMIK, STAF_BAUM, DOSEN, MAHASISWA)*

**Response (200):**
```json
[
  {
    "id": "uuid-ta",
    "nama": "2025/2026 Ganjil",
    "tanggal_mulai": "2025-09-01",
    "tanggal_selesai": "2026-01-31",
    "krs_mulai": "2025-08-15",
    "krs_selesai": "2025-08-30",
    "is_active": true,
    "id_semester_feeder": "20251",
    "created_at": "2025-06-20T10:00:00+07:00",
    "updated_at": "2025-06-20T10:00:00+07:00"
  }
]
```

#### `GET /api/tahun-akademik/{id}`

#### `POST /api/tahun-akademik` *(SUPER_ADMIN)*

**Request Body:**
```json
{
  "nama": "2025/2026 Ganjil",
  "tanggal_mulai": "2025-09-01",
  "tanggal_selesai": "2026-01-31",
  "krs_mulai": "2025-08-15",
  "krs_selesai": "2025-08-30",
  "is_active": true,
  "id_semester_feeder": "20251"
}
```

> **Format tanggal:** `YYYY-MM-DD`

#### `PUT /api/tahun-akademik/{id}` *(SUPER_ADMIN)*

#### `DELETE /api/tahun-akademik/{id}` *(SUPER_ADMIN)*

---

### 4.8 Kurikulum

#### `GET /api/kurikulum` *(Semua user login)*

**Response (200):**
```json
[
  {
    "id": "uuid-kurikulum",
    "nama": "Kurikulum Inti 2025",
    "tahun_mulai": 2025,
    "is_active": true,
    "prodi_id": "uuid-prodi",
    "nama_prodi": "Teknik Informatika",
    "id_kurikulum_feeder": null,
    "sks_lulus": 144,
    "sks_wajib": 120,
    "sks_pilihan": 24,
    "id_semester_mulai": "20251",
    "created_at": "2025-07-06T10:00:00+07:00",
    "updated_at": "2025-07-06T10:00:00+07:00"
  }
]
```

#### `GET /api/kurikulum/{id}` *(Semua user login)*

#### `POST /api/kurikulum` *(SUPER_ADMIN, KAPRODI)*

**Request Body:**
```json
{
  "nama": "Kurikulum Inti 2025",
  "tahun_mulai": 2025,
  "is_active": true,
  "prodi_id": "uuid-prodi",
  "sks_lulus": 144,
  "sks_wajib": 120,
  "sks_pilihan": 24,
  "id_semester_mulai": "20251"
}
```

#### `PUT /api/kurikulum/{id}` *(SUPER_ADMIN, KAPRODI)*

#### `DELETE /api/kurikulum/{id}` *(SUPER_ADMIN, KAPRODI)*

#### `GET /api/kurikulum/{id}/matakuliah` *(Semua user login)*

Mendapatkan daftar mata kuliah dalam kurikulum.

#### `POST /api/kurikulum/{id}/matakuliah` *(SUPER_ADMIN, KAPRODI)*

**Request Body:**
```json
{
  "matakuliah_id": "uuid-mk"
}
```

**Response:** `201 Created`

#### `DELETE /api/kurikulum/{id}/matakuliah/{mk_id}` *(SUPER_ADMIN, KAPRODI)*

#### `GET /api/kurikulum/mapping/template` *(SUPER_ADMIN, KAPRODI)*

Download template CSV mapping kurikulum-matakuliah.

#### `POST /api/kurikulum/mapping/import` *(SUPER_ADMIN, KAPRODI)*

Import mapping via CSV. **Request:** `multipart/form-data` field `file`.

**Format CSV (delimiter `;`):**
```
nama_kurikulum;kode_mk
Kurikulum Inti 2025;IF101
Kurikulum Inti 2025;IF102
```

**Response (200):**
```json
{
  "message": "Proses import selesai.",
  "success_count": 5,
  "failed_count": 1,
  "errors": ["Kode MK 'XX999' tidak ditemukan di database."]
}
```

---


### 4.9 KRS (Kartu Rencana Studi)

#### `GET /api/krs/jadwal-available?tahun_akademik_id={uuid}` *(MAHASISWA)*

Mendapatkan jadwal kuliah yang tersedia untuk pengambilan KRS.

**Response (200):**
```json
[
  {
    "jadwal_id": "uuid-jadwal",
    "matakuliah_id": "uuid-mk",
    "kode_mk": "IF101",
    "nama_mk": "Algoritma dan Pemrograman",
    "sks": 4,
    "semester_target": 1,
    "kelas": "A",
    "nama_kelas_kuliah": "IF101-A",
    "hari": "Senin",
    "jam_mulai": "08:00+07:00",
    "jam_selesai": "10:30+07:00",
    "dosen_pengampu": "Dr. Budi Santoso, Ir. Citra",
    "is_paket": true
  }
]
```

#### `POST /api/krs/enrollments` *(MAHASISWA)*

Mengambil mata kuliah (KRS).

**Request Body:**
```json
{
  "tahun_akademik_id": "uuid-ta",
  "jadwal_kuliah_ids": ["uuid-jadwal-1", "uuid-jadwal-2"]
}
```

**Response (201):**
```json
{
  "message": "Mata Kuliah berhasil masuk krs"
}
```

#### `GET /api/krs/my-enrollments?tahun_akademik_id={uuid}` *(MAHASISWA)*

Melihat KRS yang sudah diambil.

**Response (200):**
```json
[
  {
    "id": "uuid-enrollment",
    "registrasi_id": "uuid-registrasi",
    "tahun_akademik": "2025/2026 Ganjil",
    "kode_mk": "IF101",
    "nama_mk": "Algoritma dan Pemrograman",
    "sks": 4,
    "status_approval": "MenungguPersetujuan",
    "nilai_huruf": null,
    "id_peserta_kelas_feeder": null,
    "id_nilai_feeder": null,
    "nilai_angka": null,
    "nilai_indeks": null
  }
]
```

> **Status Approval:** `MenungguPersetujuan` | `Disetujui` | `Ditolak` | `Selesai` | `Mengulang`

#### `DELETE /api/krs/enrollments/{id}` *(MAHASISWA, SUPER_ADMIN)*

Membatalkan pengambilan mata kuliah.

#### `PUT /api/krs/enrollments/{id}/status` *(DOSEN, SUPER_ADMIN)*

Menyetujui/menolak KRS mahasiswa (oleh Dosen PA).

**Request Body:**
```json
{
  "status_approval": "Disetujui"
}
```

#### `PUT /api/krs/enrollments/{id}/nilai` *(DOSEN, SUPER_ADMIN)*

Input nilai mahasiswa.

**Request Body:**
```json
{
  "nilai_angka": "85.50",
  "nilai_indeks": "3.50",
  "nilai_huruf": "A",
  "id_nilai_feeder": null
}
```

---

### 4.10 Dosen PA

#### `GET /api/dosen-pa/my-advisees` *(DOSEN)*

Mendapatkan daftar mahasiswa bimbingan.

**Response (200):**
```json
[
  {
    "id": "uuid-mahasiswa",
    "nim": "250101001",
    "nama_mahasiswa": "Ahmad Fauzi",
    "angkatan": 2025,
    "email": "ahmad@student.kampus.ac.id",
    "nama_prodi": "Teknik Informatika"
  }
]
```

#### `GET /api/dosen-pa/advisee-krs/{mahasiswa_id}?tahun_akademik_id={uuid}` *(DOSEN, SUPER_ADMIN)*

Melihat KRS mahasiswa bimbingan.

#### `PUT /api/dosen-pa/batch-assign` *(SUPER_ADMIN, KAPRODI, STAF_AKADEMIK)*

Assign Dosen PA secara batch per rombel.

**Request Body:**
```json
{
  "prodi_id": "uuid-prodi",
  "angkatan": 2025,
  "kode_rombel": "A",
  "dosen_pa_id": "uuid-dosen"
}
```

**Response (200):**
```json
{
  "message": "Berhasil menetapkan Dosen PA untuk 30 mahasiswa."
}
```

#### `PUT /api/dosen-pa/single-assign` *(SUPER_ADMIN, KAPRODI, STAF_AKADEMIK)*

Assign Dosen PA untuk satu mahasiswa.

**Request Body:**
```json
{
  "registrasi_id": "uuid-registrasi",
  "dosen_pa_id": "uuid-dosen"
}
```

---


### 4.11 Akademik - Jadwal Kuliah

#### `GET /api/akademik/jadwal-kuliah?tahun_akademik_id={uuid}&prodi_id={uuid}` *(SUPER_ADMIN, STAF_AKADEMIK, STAF_BAUM, DOSEN)*

**Response (200):**
```json
[
  {
    "id": "uuid-jadwal",
    "kelas": "A",
    "id_kelas_kuliah_feeder": null,
    "nama_kelas_kuliah": "IF101-A",
    "hari": "Senin",
    "jam_mulai": "08:00+07:00",
    "jam_selesai": "10:30+07:00",
    "matakuliah_id": "uuid-mk",
    "nama_mk": "Algoritma dan Pemrograman",
    "kode_mk": "IF101",
    "sks": 4,
    "prodi_id": "uuid-prodi",
    "nama_prodi": "Teknik Informatika",
    "tahun_akademik_id": "uuid-ta",
    "nama_tahun_akademik": "2025/2026 Ganjil",
    "dosen_pengampu": [
      {
        "dosen_id": "uuid-dosen",
        "nama_dosen": "Dr. Budi Santoso",
        "peran": "Koordinator",
        "id_aktivitas_mengajar_feeder": null,
        "sks_substansi_total": "4.00",
        "rencana_tatap_muka": 16,
        "realisasi_tatap_muka": null
      }
    ],
    "ruangan_id": "uuid-ruangan",
    "nama_ruangan": "Lab Komputer 1"
  }
]
```

#### `POST /api/akademik/jadwal-kuliah` *(SUPER_ADMIN, STAF_AKADEMIK)*

**Request Body:**
```json
{
  "matakuliah_id": "uuid-mk",
  "tahun_akademik_id": "uuid-ta",
  "hari": "Senin",
  "jam_mulai": "08:00+07:00",
  "jam_selesai": "10:30+07:00",
  "kelas": "A",
  "nama_kelas_kuliah": "IF101-A",
  "dosen_pengampu": [
    {
      "dosen_id": "uuid-dosen",
      "peran": "Koordinator",
      "sks_substansi_total": "4.00",
      "rencana_tatap_muka": 16
    }
  ]
}
```

> **Hari:** `Senin` | `Selasa` | `Rabu` | `Kamis` | `Jumat` | `Sabtu` | `Minggu`  
> **Peran:** `Koordinator` | `Anggota`  
> **Format jam:** `HH:MM+07:00` (TIMETZ) atau `HH:MM` (otomatis WIB)

#### `PUT /api/akademik/jadwal-kuliah/{id}` *(SUPER_ADMIN, STAF_AKADEMIK)*

#### `DELETE /api/akademik/jadwal-kuliah/{id}` *(SUPER_ADMIN, STAF_AKADEMIK)*

#### `GET /api/akademik/jadwal-kuliah/template-csv`

Download template CSV import jadwal.

#### `POST /api/akademik/jadwal-kuliah/import-csv` *(SUPER_ADMIN, STAF_AKADEMIK)*

Import jadwal dari CSV. **Request:** `multipart/form-data` field `file`.

**Format CSV (delimiter `;`):**
```
Hari;Jam;Kode MK;Dosen Pengampu;Kelas;Ruangan;tahun akademik
Senin;08:00-10:30;IF101;Dr. Budi Santoso;A;Lab Komputer 1;2025/2026 Ganjil
```

#### `POST /api/akademik/plot-jadwal-ruangan` *(SUPER_ADMIN, STAF_BAUM)*

Plot jadwal kuliah ke ruangan.

**Request Body:**
```json
{
  "jadwal_kuliah_id": "uuid-jadwal",
  "ruangan_id": "uuid-ruangan"
}
```

#### `DELETE /api/akademik/plot-jadwal-ruangan/{id}` *(SUPER_ADMIN, STAF_BAUM)*

---

### 4.12 Akademik - Rencana Penilaian

> **Roles:** SUPER_ADMIN, KAPRODI, DOSEN

#### `GET /api/akademik/jadwal-kuliah/{jadwal_kuliah_id}/rencana-penilaian`

**Response (200):**
```json
{
  "id": "uuid",
  "jadwal_kuliah_id": "uuid-jadwal",
  "file_kontrak_path": null,
  "bobot_kehadiran": "10.00",
  "bobot_tugas": "20.00",
  "bobot_uts": "25.00",
  "bobot_uas": "30.00",
  "bobot_praktek": "15.00",
  "catatan_rencana_praktikum": "Lab setiap Rabu",
  "file_praktikum_path": null,
  "created_at": "2025-09-01T10:00:00+07:00",
  "updated_at": "2025-09-01T10:00:00+07:00"
}
```

#### `PUT /api/akademik/jadwal-kuliah/{jadwal_kuliah_id}/rencana-penilaian`

**Request Body:**
```json
{
  "bobot_kehadiran": "10.00",
  "bobot_tugas": "20.00",
  "bobot_uts": "25.00",
  "bobot_uas": "30.00",
  "bobot_praktek": "15.00",
  "catatan_rencana_praktikum": "Lab setiap Rabu"
}
```

#### `POST /api/akademik/jadwal-kuliah/{jadwal_kuliah_id}/rencana-penilaian/upload/{jenis_file}`

Upload file kontrak/praktikum. `{jenis_file}`: `kontrak` | `praktikum`

**Request:** `multipart/form-data` field `file`.

---


### 4.13 Aset - Jenis Aset

> **Roles:** SUPER_ADMIN, STAF_BAUM

#### `GET /api/aset/jenis`

**Response (200):**
```json
[
  {
    "id": "uuid",
    "nama_jenis": "Komputer",
    "deskripsi": "Perangkat komputer desktop/laptop",
    "kelompok": "Sarana",
    "created_at": "2025-07-14T10:00:00+07:00",
    "updated_at": "2025-07-14T10:00:00+07:00"
  }
]
```

> **Kelompok:** `"Sarana"` | `"Prasarana"`

#### `POST /api/aset/jenis`

**Request Body:**
```json
{
  "nama_jenis": "Komputer",
  "deskripsi": "Perangkat komputer desktop/laptop",
  "kelompok": "Sarana"
}
```

#### `GET /api/aset/jenis/{id}`
#### `PUT /api/aset/jenis/{id}`
#### `DELETE /api/aset/jenis/{id}`

---

### 4.14 Aset - Ruangan

> **Roles:** SUPER_ADMIN, STAF_BAUM

#### `GET /api/aset/ruangan`

**Response (200):**
```json
[
  {
    "id": "uuid",
    "kode_ruangan": "R101",
    "nama_ruangan": "Lab Komputer 1",
    "kapasitas": 40,
    "panjang": "12.00",
    "lebar": "8.00",
    "created_at": "2025-07-11T10:00:00+07:00",
    "updated_at": "2025-07-11T10:00:00+07:00"
  }
]
```

#### `POST /api/aset/ruangan`

**Request Body:**
```json
{
  "kode_ruangan": "R101",
  "nama_ruangan": "Lab Komputer 1",
  "kapasitas": 40,
  "panjang": "12.00",
  "lebar": "8.00"
}
```

#### `GET /api/aset/ruangan/{id}`
#### `PUT /api/aset/ruangan/{id}`
#### `DELETE /api/aset/ruangan/{id}`

---

### 4.15 Aset - Item

> **Roles:** SUPER_ADMIN, STAF_BAUM

#### `GET /api/aset/item?ruangan_id={uuid}` (filter opsional)

**Response (200):**
```json
[
  {
    "id": "uuid",
    "nama_aset": "Laptop ASUS X515",
    "kode_aset": "AST-2025-001",
    "deskripsi": "Laptop untuk lab",
    "tanggal_pembelian": "2025-03-15",
    "kondisi": "Baik",
    "jenis_aset_id": "uuid-jenis",
    "nama_jenis": "Komputer",
    "ruangan_id": "uuid-ruangan",
    "nama_ruangan": "Lab Komputer 1",
    "kode_ruangan": "R101",
    "created_at": "2025-07-14T10:00:00+07:00",
    "updated_at": "2025-07-14T10:00:00+07:00",
    "peminjaman_id": null,
    "nama_peminjam": null,
    "estimasi_tanggal_kembali": null
  }
]
```

> **Kondisi:** `"Baik"` | `"Rusak Ringan"` | `"Rusak Berat"` | `"Dalam Perbaikan"` | `"Dihapuskan"`

#### `POST /api/aset/item`

**Request Body:**
```json
{
  "nama_aset": "Laptop ASUS X515",
  "kode_aset": "AST-2025-001",
  "deskripsi": "Laptop untuk lab",
  "tanggal_pembelian": "2025-03-15",
  "kondisi": "Baik",
  "jenis_aset_id": "uuid-jenis",
  "ruangan_id": "uuid-ruangan"
}
```

#### `GET /api/aset/item/{id}`
#### `PUT /api/aset/item/{id}`
#### `DELETE /api/aset/item/{id}`

#### `GET /api/aset/item/{id}/histori`

Mendapatkan riwayat histori aset.

#### `POST /api/aset/item/{id}/histori`

**Request Body:**
```json
{
  "status": "Dipindahkan",
  "catatan": "Dipindahkan ke ruang dosen",
  "ke_ruangan_id": "uuid-ruangan-baru"
}
```

> **Status Histori:** `"Ditempatkan"` | `"Dipindahkan"` | `"Dipinjam"` | `"Dikembalikan"` | `"Dalam Perbaikan"` | `"Perbaikan Selesai"` | `"Dihapuskan"`

#### `POST /api/aset/item/{id}/pindahkan`

**Request Body:**
```json
{
  "ke_ruangan_id": "uuid-ruangan-baru",
  "catatan": "Dipindahkan untuk keperluan lab baru"
}
```

#### `POST /api/aset/item/{id}/update-kondisi`

**Request Body:**
```json
{
  "kondisi": "Rusak Ringan",
  "catatan": "Keyboard rusak"
}
```

#### `POST /api/aset/item/{id}/pinjam`

**Request Body:**
```json
{
  "user_peminjam_id": "uuid-user",
  "estimasi_tanggal_kembali": "2025-08-01T17:00:00+07:00",
  "catatan": "Untuk kegiatan seminar"
}
```

#### `POST /api/peminjaman/{id}/kembalikan`

**Request Body:**
```json
{
  "catatan": "Dikembalikan dalam kondisi baik"
}
```

#### `GET /api/aset/summary-kondisi`

**Response (200):**
```json
{
  "baik": 150,
  "rusak_ringan": 10,
  "rusak_berat": 3,
  "dalam_perbaikan": 5,
  "dihapuskan": 2
}
```

#### `GET /api/aset/item/{id}/summary-aktivitas`

**Response (200):**
```json
{
  "ditempatkan": 1,
  "dipindahkan": 3,
  "dipinjam": 5,
  "dikembalikan": 4,
  "dalam_perbaikan": 1,
  "perbaikan_selesai": 1,
  "dihapuskan": 0
}
```

---


### 4.16 Aset - Jadwal Ruangan

> **Roles:** SUPER_ADMIN, STAF_BAUM

#### `GET /api/aset/ruangan/{id}/jadwal?start={rfc3339}&end={rfc3339}`

Query: `start` dan `end` dalam format RFC3339 (`2025-09-01T00:00:00+07:00`)

**Response (200):**
```json
[
  {
    "id": "uuid",
    "ruangan_id": "uuid-ruangan",
    "judul_kegiatan": "Rapat Jurusan",
    "deskripsi": "Rapat koordinasi semester",
    "waktu_mulai": "2025-09-15T09:00:00+07:00",
    "waktu_selesai": "2025-09-15T11:00:00+07:00",
    "recurring_event_id": null,
    "jadwal_kuliah_id": null,
    "user_pembuat_id": "uuid-user",
    "nama_pembuat": "Admin"
  }
]
```

#### `POST /api/aset/ruangan/jadwal`

**Request Body (single event):**
```json
{
  "ruangan_id": "uuid-ruangan",
  "judul_kegiatan": "Rapat Jurusan",
  "deskripsi": "Rapat koordinasi",
  "waktu_mulai": "2025-09-15T09:00:00+07:00",
  "waktu_selesai": "2025-09-15T11:00:00+07:00"
}
```

**Request Body (recurring event):**
```json
{
  "ruangan_id": "uuid-ruangan",
  "judul_kegiatan": "Kuliah Umum",
  "waktu_mulai": "2025-09-01T08:00:00+07:00",
  "waktu_selesai": "2025-09-01T10:00:00+07:00",
  "tipe_perulangan": "Mingguan",
  "tanggal_akhir_perulangan": "2025-12-31"
}
```

> **Tipe Perulangan:** `"Mingguan"` | `"Harian"`

#### `DELETE /api/aset/ruangan/jadwal/{id}`
#### `DELETE /api/aset/ruangan/jadwal/recurring/{id}` (hapus semua jadwal berulang)

---

### 4.17 Aset - Biaya

> **Roles:** SUPER_ADMIN, STAF_BAUM

#### `GET /api/aset/item/{aset_id}/biaya`

**Response (200):**
```json
[
  {
    "id": "uuid",
    "aset_id": "uuid-aset",
    "tipe_biaya": "Pembelian",
    "deskripsi": "Pembelian awal laptop",
    "jumlah": "8500000.00",
    "tanggal_transaksi": "2025-03-15",
    "vendor": "Toko Komputer ABC",
    "user_pencatat_id": "uuid-user",
    "nama_pencatat": "Admin",
    "bukti_url": "uploads/biaya/uuid.pdf",
    "created_at": "2025-09-13T10:00:00+07:00",
    "updated_at": "2025-09-13T10:00:00+07:00"
  }
]
```

> **Tipe Biaya:** `"Pembelian"` | `"Perawatan"` | `"Perbaikan"` | `"Upgrade"` | `"Lain-lain"`

#### `POST /api/aset/item/{aset_id}/biaya`

**Request Body:**
```json
{
  "aset_id": "uuid-aset",
  "tipe_biaya": "Pembelian",
  "deskripsi": "Pembelian awal laptop",
  "jumlah": "8500000.00",
  "tanggal_transaksi": "2025-03-15",
  "vendor": "Toko Komputer ABC"
}
```

#### `PUT /api/aset/biaya/{id}`
#### `DELETE /api/aset/biaya/{id}`

#### `POST /api/aset/biaya/{id}/update-bukti`

Upload bukti pembayaran. **Request:** `multipart/form-data` field `file`.

#### `DELETE /api/aset/biaya/{id}/hapus-bukti`

#### `GET /api/aset/biaya/summary?start_date=2025-01-01&end_date=2025-12-31`

**Response (200):**
```json
[
  { "tipe_biaya": "Pembelian", "total": "50000000.00" },
  { "tipe_biaya": "Perawatan", "total": "5000000.00" }
]
```

---


### 4.18 Aset - Habis Pakai (Konsumsi)

> **Roles:** SUPER_ADMIN, STAF_BAUM

#### `GET /api/aset/konsumsi`

**Response (200):**
```json
[
  {
    "id": "uuid",
    "nama_barang": "Kertas HVS A4",
    "deskripsi": "Kertas untuk printer",
    "satuan": "Rim",
    "stok": 50,
    "batas_minimum_stok": 10,
    "created_at": "2025-07-18T10:00:00+07:00",
    "updated_at": "2025-07-18T10:00:00+07:00"
  }
]
```

#### `POST /api/aset/konsumsi`

**Request Body:**
```json
{
  "nama_barang": "Kertas HVS A4",
  "deskripsi": "Kertas untuk printer",
  "satuan": "Rim",
  "batas_minimum_stok": 10
}
```

#### `GET /api/aset/konsumsi/{id}`
#### `PUT /api/aset/konsumsi/{id}`
#### `DELETE /api/aset/konsumsi/{id}`

#### `POST /api/aset/konsumsi/{id}/tambah-stok`

**Request Body:**
```json
{
  "jumlah": 20,
  "catatan": "Pembelian bulanan"
}
```

#### `POST /api/aset/konsumsi/{id}/ambil-stok`

**Request Body:**
```json
{
  "jumlah": 5,
  "catatan": "Untuk bagian akademik"
}
```

#### `POST /api/aset/konsumsi/{id}/stok-opname`

**Request Body:**
```json
{
  "stok_fisik": 45,
  "catatan": "Selisih 2 rim (mungkin tercecer)"
}
```

#### `GET /api/aset/konsumsi/{id}/histori`

**Response (200):**
```json
[
  {
    "id": "uuid",
    "tipe_transaksi": "Pembelian",
    "jumlah": 20,
    "saldo_sebelum": 30,
    "saldo_setelah": 50,
    "catatan": "Pembelian bulanan",
    "tanggal_transaksi": "2025-07-20T10:00:00+07:00",
    "user_aksi_id": "uuid-user",
    "nama_user_aksi": "Admin"
  }
]
```

> **Tipe Transaksi:** `"Pembelian"` | `"Pengambilan"` | `"Stok Opname"`

#### `GET /api/aset/konsumsi/low-stock`

Mendapatkan barang yang stoknya di bawah batas minimum.

**Response (200):**
```json
[
  {
    "id": "uuid",
    "nama_barang": "Tinta Printer",
    "stok": 2,
    "batas_minimum_stok": 5
  }
]
```

---


### 4.19 Fleet - Kendaraan

#### `GET /api/fleet/kendaraan` *(Semua user login)*

**Response (200):**
```json
[
  {
    "id": "uuid",
    "jenis": "Mobil",
    "nama": "Toyota Avanza Putih",
    "nomor_polisi": "D 1234 ABC",
    "merk": "Toyota",
    "model": "Avanza",
    "tahun": 2022,
    "status": "Tersedia",
    "created_at": "2025-10-04T10:00:00+07:00",
    "updated_at": "2025-10-04T10:00:00+07:00"
  }
]
```

> **Jenis:** `"Mobil"` | `"Motor"` | `"Bus"`  
> **Status:** `"Tersedia"` | `"Digunakan"` | `"Perawatan"`

#### `GET /api/fleet/kendaraan/{id}` *(Semua user login)*

#### `POST /api/fleet/kendaraan` *(SUPER_ADMIN, STAF_BAUM)*

**Request Body:**
```json
{
  "jenis": "Mobil",
  "nama": "Toyota Avanza Putih",
  "nomor_polisi": "D 1234 ABC",
  "merk": "Toyota",
  "model": "Avanza",
  "tahun": 2022
}
```

#### `PUT /api/fleet/kendaraan/{id}` *(SUPER_ADMIN, STAF_BAUM)*
#### `DELETE /api/fleet/kendaraan/{id}` *(SUPER_ADMIN, STAF_BAUM)*

#### `GET /api/fleet/kendaraan-tersedia?start={rfc3339}&end={rfc3339}` *(Semua user login)*

Mencari kendaraan yang tersedia pada rentang waktu tertentu.

**Response (200):**
```json
[
  {
    "id": "uuid",
    "jenis": "Mobil",
    "nama": "Toyota Avanza Putih",
    "nomor_polisi": "D 1234 ABC"
  }
]
```

#### `GET /api/fleet/kendaraan/{id}/summary?start_date=2025-01-01&end_date=2025-12-31` *(SUPER_ADMIN, STAF_BAUM)*

**Response (200):**
```json
{
  "total_biaya_servis": "15000000.00",
  "total_jarak_tempuh": 12500,
  "biaya_per_km": "1200.00"
}
```

---

### 4.20 Fleet - Booking

#### `POST /api/fleet/bookings` *(Semua user login)*

Membuat pemesanan kendaraan.

**Request Body:**
```json
{
  "kendaraan_id": "uuid-kendaraan",
  "tujuan": "Bandung - Pertemuan dengan mitra",
  "waktu_berangkat": "2025-10-15T07:00:00+07:00",
  "estimasi_waktu_kembali": "2025-10-15T18:00:00+07:00"
}
```

#### `GET /api/fleet/my-bookings` *(Semua user login)*

Melihat booking milik sendiri.

#### `GET /api/fleet/kendaraan/{id}/bookings` *(Semua user login)*

Melihat booking untuk kendaraan tertentu.

#### `GET /api/fleet/bookings?status={status}&start={rfc3339}&end={rfc3339}` *(SUPER_ADMIN, STAF_BAUM)*

**Response (200):**
```json
[
  {
    "id": "uuid-booking",
    "kendaraan_id": "uuid-kendaraan",
    "nama_kendaraan": "Toyota Avanza Putih",
    "user_pemesan_id": "uuid-user",
    "nama_pemesan": "Dr. Budi Santoso",
    "tujuan": "Bandung - Pertemuan dengan mitra",
    "waktu_berangkat": "2025-10-15T07:00:00+07:00",
    "estimasi_waktu_kembali": "2025-10-15T18:00:00+07:00",
    "status": "Diajukan"
  }
]
```

> **Status Booking:** `"Diajukan"` | `"Disetujui"` | `"Ditolak"` | `"Dibatalkan"` | `"Berlangsung"` | `"Selesai"`

#### `PUT /api/fleet/bookings/{id}/approve` *(SUPER_ADMIN, STAF_BAUM)*

**Request Body:**
```json
{
  "catatan": "Disetujui, gunakan jalur tol"
}
```

#### `PUT /api/fleet/bookings/{id}/reject` *(SUPER_ADMIN, STAF_BAUM)*

**Request Body:**
```json
{
  "catatan": "Kendaraan sedang dalam perawatan"
}
```

#### `POST /api/fleet/bookings/{id}/start-trip` *(SUPER_ADMIN, STAF_BAUM)*

**Request Body:**
```json
{
  "odometer_awal": 45000,
  "waktu_aktual_berangkat": "2025-10-15T07:30:00+07:00"
}
```

#### `POST /api/fleet/bookings/{id}/end-trip` *(SUPER_ADMIN, STAF_BAUM)*

**Request Body:**
```json
{
  "odometer_akhir": 45200,
  "bahan_bakar_diisi": "150000.00",
  "catatan_kondisi_kembali": "Kondisi baik",
  "waktu_aktual_kembali": "2025-10-15T17:30:00+07:00"
}
```

#### `GET /api/fleet/bookings/{id}/log` *(SUPER_ADMIN, STAF_BAUM)*

#### `GET /api/fleet/bookings/summary` *(SUPER_ADMIN, STAF_BAUM)*

**Response (200):**
```json
{
  "diajukan": 5,
  "disetujui": 3,
  "ditolak": 1,
  "dibatalkan": 0,
  "berlangsung": 2,
  "selesai": 10
}
```

---


### 4.21 Fleet - Servis

> **Roles:** SUPER_ADMIN, STAF_BAUM

#### `GET /api/fleet/kendaraan/{kendaraan_id}/servis?start_date=2025-01-01&end_date=2025-12-31`

**Response (200):**
```json
[
  {
    "id": "uuid",
    "kendaraan_id": "uuid-kendaraan",
    "tanggal_servis": "2025-06-15",
    "odometer_saat_servis": 35000,
    "deskripsi": "Service berkala 30.000 km",
    "biaya": "2500000.00",
    "user_pencatat_id": "uuid-user",
    "nama_pencatat": "Admin"
  }
]
```

#### `POST /api/fleet/kendaraan/{kendaraan_id}/servis`

**Request Body:**
```json
{
  "tanggal_servis": "2025-06-15",
  "odometer_saat_servis": 35000,
  "deskripsi": "Service berkala 30.000 km",
  "biaya": "2500000.00"
}
```

#### `GET /api/fleet/servis/{id}`
#### `PUT /api/fleet/servis/{id}`
#### `DELETE /api/fleet/servis/{id}`

---

### 4.22 SDM - Pegawai

#### `GET /api/sdm/pegawai` *(SUPER_ADMIN, STAF_BASDM)*

**Response (200):** Array dari objek Pegawai lengkap (biodata, data dosen jika ada).

```json
[
  {
    "id": "uuid-pegawai",
    "user_id": "uuid-user",
    "nik": "198901",
    "no_ktp": "3302xxxxxxxxxxxx",
    "nama_lengkap": "Dr. Budi Santoso, M.Kom",
    "gelar_depan": "Dr.",
    "gelar_belakang": "M.Kom",
    "tempat_lahir": "Tasikmalaya",
    "tanggal_lahir": "1989-04-12",
    "jenis_kelamin": "L",
    "status_nikah": "Menikah",
    "agama": "Islam",
    "kategori_pegawai": "Tenaga Pendidik",
    "status_pegawai": "Tetap",
    "is_active": true,
    "tanggal_masuk": "2015-09-01",
    "nidn": "0412098901",
    "prodi_id": "uuid-prodi",
    "nama_prodi": "Teknik Informatika",
    "created_at": "2025-06-20T10:00:00+07:00",
    "updated_at": "2025-06-20T10:00:00+07:00"
  }
]
```

> **Jenis Kelamin:** `"L"` | `"P"`  
> **Status Nikah:** `"Menikah"` | `"Belum Menikah"` | `"Cerai Hidup"` | `"Cerai Mati"`  
> **Kategori Pegawai:** `"Tenaga Pendidik"` | `"Tenaga Kependidikan"`  
> **Status Pegawai:** `"Tetap"` | `"Kontrak"` | `"Honorer"`

#### `POST /api/sdm/pegawai` *(SUPER_ADMIN, STAF_BASDM)*

**Request Body:**
```json
{
  "nik": "198901",
  "nama_lengkap": "Dr. Budi Santoso, M.Kom",
  "gelar_depan": "Dr.",
  "gelar_belakang": "M.Kom",
  "tempat_lahir": "Tasikmalaya",
  "tanggal_lahir": "1989-04-12",
  "jenis_kelamin": "L",
  "status_nikah": "Menikah",
  "agama": "Islam",
  "email": "budi@kampus.ac.id",
  "nomor_hp": "081234567890",
  "kategori_pegawai": "Tenaga Pendidik",
  "status_pegawai": "Tetap",
  "tanggal_masuk": "2015-09-01",
  "nidn": "0412098901",
  "prodi_id": "uuid-prodi",
  "password": "password123"
}
```

> Jika `kategori_pegawai` = `"Tenaga Pendidik"`, field `nidn` dan `prodi_id` akan digunakan untuk membuat data dosen.

#### `GET /api/sdm/pegawai/{id}` *(SUPER_ADMIN, STAF_BASDM)*
#### `PUT /api/sdm/pegawai/{id}` *(SUPER_ADMIN, STAF_BASDM)*
#### `DELETE /api/sdm/pegawai/{id}` *(SUPER_ADMIN)*

#### `POST /api/sdm/pegawai/{id}/create-user` *(SUPER_ADMIN)*

Membuat akun user untuk pegawai yang belum punya akun.

**Request Body:**
```json
{
  "password": "password123"
}
```

---


### 4.23 SDM - Absensi

#### Pegawai (KARYAWAN)

##### `POST /api/sdm/absensi/clock-in`

**Request Body:**
```json
{
  "latitude": "-7.3506",
  "longitude": "108.2172",
  "alamat_absensi": "Kampus Universitas Respati",
  "foto_absensi_path": "uploads/absensi/foto.jpg",
  "face_confidence_score": 0.95,
  "is_face_verified": true
}
```

**Response (200):**
```json
{
  "pesan_notifikasi": "Anda terlambat 15 menit.",
  "id": "uuid",
  "pegawai_id": "uuid-pegawai",
  "waktu_absensi": "2025-09-01T08:15:00+07:00",
  "tipe_absensi": "ClockIn",
  "latitude": "-7.3506",
  "longitude": "108.2172",
  "alamat_absensi": "Kampus Universitas Respati",
  "foto_absensi_path": "uploads/absensi/foto.jpg",
  "face_confidence_score": 0.95,
  "is_face_verified": true
}
```

##### `POST /api/sdm/absensi/clock-out`

Request Body sama dengan clock-in.

##### `GET /api/sdm/absensi/rekap-saya?bulan=9&tahun=2025`

##### `GET /api/sdm/absensi/log-saya?tanggal=2025-09-01`

#### Admin (SUPER_ADMIN, STAF_BASDM)

##### `GET /api/sdm/absensi/rekap-semua?bulan=9&tahun=2025&pegawai_id={uuid}`

##### `POST /api/sdm/absensi/rekap-manual`

**Request Body:**
```json
{
  "pegawai_id": "uuid-pegawai",
  "tanggal": "2025-09-01",
  "status": "Sakit",
  "keterangan": "Surat dokter terlampir"
}
```

> **Status Absensi:** `"Hadir"` | `"Sakit"` | `"Ijin"` | `"Cuti"` | `"Alpa"`

##### `GET /api/sdm/absensi/laporan-harian?tanggal=2025-09-01`

**Response (200):**
```json
[
  {
    "pegawai_id": "uuid",
    "nama_pegawai": "Dr. Budi Santoso",
    "tanggal": "2025-09-01",
    "clock_in": "2025-09-01T07:55:00+07:00",
    "clock_out": "2025-09-01T16:30:00+07:00",
    "keterangan": "Hadir",
    "terlambat_menit": 0,
    "terlambat_toleransi_menit": 0,
    "lembur_menit": 30,
    "foto_absensi_path_in": "...",
    "foto_absensi_path_out": "...",
    "latitude_in": "-7.3506",
    "longitude_in": "108.2172",
    "latitude_out": "-7.3506",
    "longitude_out": "108.2172"
  }
]
```

##### `GET /api/sdm/absensi/laporan-bulanan?bulan=9&tahun=2025&pegawai_id={uuid}`

##### `GET /api/sdm/absensi/biometrik-status`

**Response (200):**
```json
[
  {
    "pegawai_id": "uuid",
    "nik": "198901",
    "nama_pegawai": "Dr. Budi Santoso",
    "foto_wajah_path": "uploads/wajah/uuid.jpg",
    "status_audit_wajah": "Disetujui"
  }
]
```

#### Biometrik Wajah

##### `POST /api/sdm/absensi/enroll-wajah` *(KARYAWAN, SUPER_ADMIN, STAF_BASDM)*

Upload foto wajah. **Request:** `multipart/form-data`.

##### `GET /api/sdm/absensi/wajah-saya` *(KARYAWAN)*
##### `GET /api/sdm/absensi/wajah/{pegawai_id}` *(SUPER_ADMIN, STAF_BASDM, KARYAWAN)*
##### `PUT /api/sdm/absensi/wajah/{pegawai_id}/audit` *(SUPER_ADMIN, STAF_BASDM)*
##### `DELETE /api/sdm/absensi/wajah/{pegawai_id}` *(SUPER_ADMIN, STAF_BASDM)*

---


### 4.24 SDM - Cuti

#### Pegawai (KARYAWAN)

##### `POST /api/sdm/cuti/ajukan`

**Request Body:**
```json
{
  "tanggal_mulai": "2025-12-20",
  "tanggal_selesai": "2025-12-25",
  "jumlah_hari": 4,
  "alasan": "Liburan akhir tahun",
  "kategori": "Cuti Tahunan"
}
```

> **Kategori Cuti:** `"Cuti Tahunan"` | `"Cuti Melahirkan"` | `"Cuti Sakit Berkepanjangan"` | `"Cuti Hajatan Keluarga"` | `"Cuti Ibadah"` | `"Lainnya"`

##### `GET /api/sdm/cuti/saya`

##### `GET /api/sdm/cuti/kuota-saya?tahun=2025`

**Response (200):**
```json
{
  "kuota_total": 12,
  "kuota_terpakai": 4,
  "sisa_cuti": 8,
  "tahun": 2025
}
```

#### Admin (SUPER_ADMIN, STAF_BASDM)

##### `GET /api/sdm/cuti/semua`

##### `POST /api/sdm/cuti/jatah`

Membuat kuota cuti untuk pegawai.

**Request Body:**
```json
{
  "pegawai_id": "uuid-pegawai",
  "tahun": 2025,
  "kuota_total": 12
}
```

##### `GET /api/sdm/cuti/jatah?tahun=2025&pegawai_id={uuid}`

**Response (200):**
```json
[
  {
    "id": "uuid",
    "pegawai_id": "uuid",
    "nama_pegawai": "Dr. Budi Santoso",
    "nik": "198901",
    "tahun": 2025,
    "kuota_total": 12,
    "kuota_terpakai": 4
  }
]
```

##### `PUT /api/sdm/cuti/{id}/setujui`

**Request Body:**
```json
{
  "catatan": "Disetujui"
}
```

##### `PUT /api/sdm/cuti/{id}/tolak`

**Request Body:**
```json
{
  "catatan": "Tidak bisa karena berdekatan dengan jadwal ujian"
}
```

---

### 4.25 SDM - Ijin

#### Pegawai (KARYAWAN)

##### `POST /api/sdm/ijin/ajukan`

**Request Body:**
```json
{
  "kategori": "Sakit",
  "tanggal_mulai": "2025-09-10",
  "tanggal_selesai": "2025-09-11",
  "alasan": "Demam tinggi, perlu istirahat"
}
```

> **Kategori Ijin:** `"Sakit"` | `"Urusan Keluarga"` | `"Dinas Luar"` | `"WFH"` | `"Lainnya"`

##### `GET /api/sdm/ijin/saya`

#### Admin (SUPER_ADMIN, STAF_BASDM)

##### `GET /api/sdm/ijin/semua`
##### `PUT /api/sdm/ijin/{id}/setujui`

**Request Body:**
```json
{
  "catatan": "Disetujui, segera sembuh"
}
```

##### `PUT /api/sdm/ijin/{id}/tolak`

---

### 4.26 SDM - Surat Tugas

> **Roles:** SUPER_ADMIN, STAF_BASDM

#### `GET /api/sdm/surat-tugas`

#### `POST /api/sdm/surat-tugas`

**Request Body:**
```json
{
  "dasar_tugas": "Undangan No. 123/UN/2025",
  "tugas": "Mengikuti workshop kurikulum merdeka belajar",
  "tempat_tugas": "Jakarta",
  "tanggal_mulai": "2025-10-10",
  "tanggal_selesai": "2025-10-12",
  "penandatangan_id": "uuid-pegawai-rektor",
  "tembusan": ["Wakil Rektor I", "Dekan FMIPA"],
  "penerima_tugas": [
    {
      "pegawai_id": "uuid-pegawai-1",
      "peran": "Pelaksana Utama"
    },
    {
      "pegawai_id": "uuid-pegawai-2",
      "peran": "Pengikut"
    }
  ],
  "alasan_perjalanan": 1,
  "tujuan_kota": "Jakarta",
  "alat_angkut": "Pesawat",
  "tempat_berangkat": "Tasikmalaya",
  "lama_perjalanan": 3,
  "pembebanan_anggaran_instansi": "Universitas Respati",
  "pembebanan_anggaran_mak": "5212.001",
  "ppk_pegawai_id": "uuid-ppk",
  "kpa_pegawai_id": "uuid-kpa"
}
```

> **Peran Perjalanan:** `"Pelaksana Utama"` | `"Pengikut"`  
> **Alasan Perjalanan:** `1` = Kunjungan/Undangan, `2` = Tugas Lembaga, `3` = Pelatihan

#### `GET /api/sdm/surat-tugas/{id}`
#### `PUT /api/sdm/surat-tugas/{id}`
#### `DELETE /api/sdm/surat-tugas/{id}`

#### `GET /api/sdm/surat-tugas/{id}/preview`

Mendapatkan preview HTML SPPD (untuk cetak/PDF).

---


### 4.27 SDM - Unit Kerja

> **Roles:** SUPER_ADMIN, STAF_BASDM

#### `GET /api/sdm/unit-kerja`

**Response (200):**
```json
[
  {
    "id": "uuid",
    "induk_unit_id": null,
    "kode_unit": "FMIPA",
    "nama_unit": "Fakultas MIPA",
    "is_active": true
  },
  {
    "id": "uuid-child",
    "induk_unit_id": "uuid-parent",
    "kode_unit": "TI",
    "nama_unit": "Program Studi Teknik Informatika",
    "is_active": true
  }
]
```

#### `POST /api/sdm/unit-kerja`

**Request Body:**
```json
{
  "induk_unit_id": "uuid-parent",
  "kode_unit": "TI",
  "nama_unit": "Program Studi Teknik Informatika",
  "is_active": true
}
```

#### `GET /api/sdm/unit-kerja/{id}`
#### `PUT /api/sdm/unit-kerja/{id}`
#### `DELETE /api/sdm/unit-kerja/{id}`

---

### 4.28 SDM - Penempatan

> **Roles:** SUPER_ADMIN, STAF_BASDM

#### `GET /api/sdm/pegawai/{pegawai_id}/penempatan`

**Response (200):**
```json
[
  {
    "id": "uuid",
    "pegawai_id": "uuid-pegawai",
    "unit_kerja_id": "uuid-unit",
    "nama_unit_kerja": "Program Studi Teknik Informatika",
    "jabatan": "Ketua Program Studi",
    "nomor_sk": "SK/2025/001",
    "tanggal_mulai": "2025-01-01",
    "tanggal_selesai": null
  }
]
```

#### `POST /api/sdm/pegawai/{pegawai_id}/penempatan`

**Request Body:**
```json
{
  "unit_kerja_id": "uuid-unit",
  "jabatan": "Ketua Program Studi",
  "nomor_sk": "SK/2025/001",
  "tanggal_mulai": "2025-01-01"
}
```

#### `PUT /api/sdm/penempatan/{id}`
#### `DELETE /api/sdm/penempatan/{id}`

---

### 4.29 SDM - Dokumen

#### `POST /api/sdm/{entity_type}/{entity_id}/dokumen` *(SUPER_ADMIN, STAF_BASDM, KARYAWAN)*

Upload dokumen. **Request:** `multipart/form-data` fields: `file`, `kategori`.

> **Entity Type (URL path):** `pegawai` | `riwayat-pendidikan` | `riwayat-sk` | `riwayat-sertifikat` | `riwayat-jad` | `riwayat-serdos` | `pengajuan-ijin`  
> **Kategori Dokumen:** `FotoProfil` | `KTP` | `KK` | `Ijazah` | `Transkrip` | `SK` | `Sertifikat` | `SuratSakit` | `DokumenPendukung` | `Lainnya`

#### `GET /api/sdm/{entity_type}/{entity_id}/dokumen`
#### `DELETE /api/sdm/dokumen/{id}`
#### `GET /api/sdm/dokumen` *(SUPER_ADMIN, STAF_BASDM)* — Melihat semua dokumen

---

### 4.30 SDM - Riwayat

> **Roles:** SUPER_ADMIN, STAF_BASDM

#### Riwayat Pendidikan

##### `GET /api/sdm/pegawai/{pegawai_id}/pendidikan`
##### `POST /api/sdm/pegawai/{pegawai_id}/pendidikan`

**Request Body:**
```json
{
  "jenjang": "S2",
  "institusi": "Institut Teknologi Bandung",
  "jurusan": "Informatika",
  "tahun_lulus": 2015
}
```

##### `PUT /api/sdm/pendidikan/{id}`
##### `DELETE /api/sdm/pendidikan/{id}`

#### Riwayat SK

##### `GET /api/sdm/pegawai/{pegawai_id}/riwayat-sk`
##### `POST /api/sdm/pegawai/{pegawai_id}/riwayat-sk`

**Request Body:**
```json
{
  "nomor_sk": "SK/2025/001",
  "tanggal_sk": "2025-01-15",
  "jenis_sk": "Pengangkatan",
  "jabatan": "Lektor",
  "keterangan": "SK Pengangkatan Dosen Tetap"
}
```

##### `PUT /api/sdm/riwayat-sk/{id}`
##### `DELETE /api/sdm/riwayat-sk/{id}`

#### Riwayat Sertifikat

##### `GET /api/sdm/pegawai/{pegawai_id}/sertifikat`
##### `POST /api/sdm/pegawai/{pegawai_id}/sertifikat`

**Request Body:**
```json
{
  "jenis_sertifikat": "Pelatihan",
  "judul_sertifikat": "Machine Learning Fundamentals",
  "nomor_sertifikat": "CERT/2025/001",
  "tanggal_pelaksanaan": "2025-03-10",
  "tingkat": "Nasional",
  "penyelenggara": "Dicoding Indonesia",
  "keterangan": null
}
```

> **Jenis Sertifikat:** `"Pelatihan"` | `"BIMTEK"` | `"Seminar"` | `"Workshop"` | `"Rekognisi Dosen"`  
> **Tingkat:** `"Lokal"` | `"Nasional"` | `"Internasional"`

##### `PUT /api/sdm/sertifikat/{id}`
##### `DELETE /api/sdm/sertifikat/{id}`

#### Riwayat JAD (Jabatan Akademik Dosen)

##### `GET /api/sdm/pegawai/{pegawai_id}/jad`
##### `POST /api/sdm/pegawai/{pegawai_id}/jad`

**Request Body:**
```json
{
  "jabatan_akademik": "Lektor",
  "pangkat_golongan": "Penata / III.c",
  "nomor_sk": "SK/JAD/2025/001",
  "tmt": "2025-01-01",
  "kompetensi_mk": "Algoritma, Basis Data"
}
```

> **Jabatan Akademik:** `"Asisten Ahli"` | `"Lektor"` | `"Lektor Kepala"` | `"Guru Besar"`

##### `PUT /api/sdm/jad/{id}`
##### `DELETE /api/sdm/jad/{id}`

#### Riwayat Serdos

##### `GET /api/sdm/pegawai/{pegawai_id}/serdos`
##### `POST /api/sdm/pegawai/{pegawai_id}/serdos`

**Request Body:**
```json
{
  "nomor_sertifikat": "SERDOS/2025/001",
  "tanggal_terbit": "2025-06-01",
  "keterangan": "Sertifikasi Dosen Profesional"
}
```

##### `PUT /api/sdm/serdos/{id}`
##### `DELETE /api/sdm/serdos/{id}`

---


### 4.31 Lookup

Endpoint utilitas untuk mengambil nilai-nilai enum dan data referensi.

> **Roles:** Semua user login

#### `GET /api/lookups/enrollment-statuses`

**Response:** `["MenungguPersetujuan", "Disetujui", "Ditolak", "Selesai", "Mengulang"]`

#### `GET /api/lookups/kondisi-aset`

**Response:** `["Baik", "Rusak Ringan", "Rusak Berat", "Dalam Perbaikan", "Dihapuskan"]`

#### `GET /api/lookups/aset-histori-statuses`

**Response:** `["Ditempatkan", "Dipindahkan", "Dipinjam", "Dikembalikan", "Dalam Perbaikan", "Perbaikan Selesai", "Dihapuskan"]`

#### `GET /api/lookups/tipe-biaya`

**Response:** `["Pembelian", "Perawatan", "Perbaikan", "Upgrade", "Lain-lain"]`

#### `GET /api/lookups/peran-dosen-pengampu`

**Response:** `["Koordinator", "Anggota"]`

#### `GET /api/lookups/kategori-cuti`

**Response:** `["Cuti Tahunan", "Cuti Melahirkan", "Cuti Sakit Berkepanjangan", ...]`

#### `GET /api/lookups/users?q=budi`

Cari user berdasarkan nama/username.

**Response (200):**
```json
[
  {
    "id": "uuid",
    "username": "dosen01",
    "full_name": "Dr. Budi Santoso"
  }
]
```

#### `GET /api/lookups/ruangan-tersedia?jadwal_kuliah_id={uuid}&q=lab`

Cari ruangan yang tersedia untuk jadwal kuliah tertentu.

**Response (200):**
```json
[
  {
    "id": "uuid",
    "nama_ruangan": "Lab Komputer 1",
    "kode_ruangan": "R101",
    "kapasitas": 40
  }
]
```

---

### 4.32 Files

#### `GET /api/files/{path}` *(Semua user login)*

Akses file yang sudah diupload.

**Contoh:** `GET /api/files/uploads/akademik/rps/uuid-mk/file.pdf`

---

### 4.33 Rombel (Rombongan Belajar)

> **Roles:** SUPER_ADMIN, STAF_AKADEMIK, KAPRODI

#### `GET /api/akademik/rombel?prodi_id={uuid}&angkatan=2025`

**Response (200):**
```json
[
  {
    "prodi_id": "uuid-prodi",
    "nama_prodi": "Teknik Informatika",
    "angkatan": 2025,
    "kode_rombel": "A",
    "jumlah_mahasiswa": 30,
    "dosen_pa_id": "uuid-dosen",
    "nama_dosen_pa": "Dr. Budi Santoso"
  }
]
```

#### `GET /api/akademik/rombel/mahasiswa?prodi_id={uuid}&angkatan=2025&kode_rombel=A`

**Response (200):**
```json
[
  {
    "registrasi_id": "uuid-reg",
    "mahasiswa_id": "uuid-mhs",
    "nim": "250101001",
    "nama_mahasiswa": "Ahmad Fauzi",
    "kode_rombel": "A"
  }
]
```

#### `PUT /api/akademik/rombel/pindah`

Memindahkan mahasiswa ke rombel lain.

**Request Body:**
```json
{
  "registrasi_ids": ["uuid-reg-1", "uuid-reg-2"],
  "kode_rombel_baru": "B"
}
```

**Response (200):**
```json
{
  "message": "Berhasil memindahkan 2 mahasiswa ke rombel baru."
}
```

#### `PUT /api/akademik/rombel/rename`

Mengubah nama kode rombel.

**Request Body:**
```json
{
  "prodi_id": "uuid-prodi",
  "angkatan": 2025,
  "kode_rombel_lama": "A",
  "kode_rombel_baru": "A1"
}
```

---


### 4.34 Ujian, Asesmen, dan Nilai Akhir

> **Roles:** `DOSEN`, `KAPRODI`, `STAF_AKADEMIK`, `STAF_BAUM`,
> `MAHASISWA`, dan `SUPER_ADMIN` sesuai kewenangan endpoint.

Modul asesmen menggunakan `jadwal_kuliah` sebagai kelas dan `enrollments`
berstatus `Disetujui` sebagai peserta. Nilai akhir dihitung dari attempt terbaru
setiap asesmen dengan rumus `Σ(nilai × bobot) / 100`.

#### Manajemen asesmen

| Method | Endpoint | Keterangan |
| --- | --- | --- |
| `GET` | `/api/asesmen?tahun_akademik_id={uuid}` | Daftar asesmen sesuai lingkup akses. |
| `GET` | `/api/asesmen/jadwal?tahun_akademik_id={uuid}` | Pilihan kelas untuk membuat asesmen. |
| `POST` | `/api/asesmen` | Membuat asesmen. |
| `GET` | `/api/asesmen/{id}` | Detail, dokumen, pelaksanaan, presensi, dan nilai peserta. |
| `PUT` | `/api/asesmen/{id}` | Mengubah asesmen yang masih dapat diedit. |
| `POST` | `/api/asesmen/{id}/submit` | Mengajukan asesmen kepada Kaprodi. |
| `POST` | `/api/asesmen/{id}/review` | Review `Disetujui` atau `PerluRevisi` oleh Kaprodi. |
| `POST` | `/api/asesmen/{id}/dokumen/{jenis}` | Unggah `Soal`, `Lampiran`, atau `KunciJawaban`. |
| `GET` | `/api/asesmen/{id}/dokumen/{document_id}/download` | Mengunduh dokumen sesuai hak akses. |
| `PUT` | `/api/asesmen/{id}/penggandaan` | Memperbarui proses penggandaan ujian manual. |
| `POST` | `/api/asesmen/{id}/mulai` | Memulai pelaksanaan dan membuat kode presensi. |
| `POST` | `/api/asesmen/{id}/selesai` | Menutup pelaksanaan dan menyimpan BAP. |
| `PUT` | `/api/asesmen/{id}/presensi/{enrollment_id}` | Koreksi presensi peserta oleh dosen. |
| `PUT` | `/api/asesmen/{id}/nilai/{enrollment_id}` | Menyimpan nilai attempt peserta. |
| `POST` | `/api/asesmen/{id}/kunci` | Mengunci nilai setelah seluruh peserta dinilai. |
| `POST` | `/api/asesmen/{id}/buka-nilai` | Membuka nilai untuk revisi jika workflow nilai akhir mengizinkan. |

Workflow asesmen:

`Draft → Diajukan → Disetujui/PerluRevisi → SiapDilaksanakan → Berlangsung → Selesai → Dinilai → Dikunci`

#### Nilai akhir mata kuliah

| Method | Endpoint | Keterangan |
| --- | --- | --- |
| `GET` | `/api/asesmen/nilai-akhir?tahun_akademik_id={uuid}` | Daftar rekap nilai kelas. |
| `GET` | `/api/asesmen/nilai-akhir/{jadwal_id}` | Komponen, nilai peserta, konversi huruf, dan riwayat. |
| `POST` | `/api/asesmen/nilai-akhir/{jadwal_id}/ajukan` | Koordinator mengajukan nilai akhir. |
| `POST` | `/api/asesmen/nilai-akhir/{jadwal_id}/review` | Kaprodi menyetujui atau meminta revisi. |
| `POST` | `/api/asesmen/nilai-akhir/{jadwal_id}/publikasikan` | Staf Akademik mempublikasikan nilai ke enrollment/KHS. |
| `GET` | `/api/asesmen/skala-nilai/{prodi_id}` | Daftar skala nilai Prodi. |
| `PUT` | `/api/asesmen/skala-nilai/{prodi_id}` | Menyimpan skala nilai oleh Kaprodi/Super Admin. |

Pengajuan nilai akhir ditolak jika total bobot bukan 100%, terdapat asesmen
yang belum `Dikunci`, nilai peserta belum lengkap, atau skala nilai Prodi tidak
mencakup hasil perhitungan. Publikasi hanya dapat dilakukan setelah persetujuan
Kaprodi. Saat dipublikasikan, `nilai_angka`, `nilai_huruf`, dan `nilai_indeks`
pada `enrollments` diperbarui sehingga dapat ditampilkan pada KHS mahasiswa.

#### Akses mahasiswa

| Method | Endpoint | Keterangan |
| --- | --- | --- |
| `GET` | `/api/asesmen-saya?tahun_akademik_id={uuid}` | Jadwal dan hasil asesmen mahasiswa. |
| `POST` | `/api/asesmen-saya/check-in` | Check-in ujian menggunakan kode aktif. |

Migration yang diperlukan:

- `20260621150000_create_asesmen_tables`
- `20260621180000_create_nilai_akhir_tables`

---

## 5. Alur Penggunaan Sistem

### 5.1 Setup Awal (Admin)

```
1. Login sebagai SUPER_ADMIN
2. Buat Tahun Akademik aktif
3. Buat Program Studi (Prodi)
4. Buat Unit Kerja (SDM)
5. Daftarkan Pegawai (otomatis buat akun & data dosen jika Tenaga Pendidik)
6. Buat Mata Kuliah per Prodi
7. Buat Kurikulum dan mapping Mata Kuliah ke Kurikulum
8. Buat Ruangan
9. Buat Jadwal Kuliah (assign dosen pengampu + plot ruangan)
10. Daftarkan Mahasiswa (atau import via CSV)
11. Assign Dosen PA ke Mahasiswa
```

### 5.2 Alur KRS Mahasiswa

```
┌─────────────────────────────────────────────────────────┐
│  MAHASISWA                                              │
│                                                         │
│  1. Login (POST /api/auth/login)                        │
│  2. Lihat jadwal tersedia                               │
│     GET /api/krs/jadwal-available?tahun_akademik_id=... │
│  3. Ambil mata kuliah                                   │
│     POST /api/krs/enrollments                           │
│  4. Lihat KRS saya                                      │
│     GET /api/krs/my-enrollments?tahun_akademik_id=...   │
│  5. Batalkan MK (jika masih bisa)                       │
│     DELETE /api/krs/enrollments/{id}                    │
└─────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────┐
│  DOSEN PA                                               │
│                                                         │
│  1. Lihat mahasiswa bimbingan                           │
│     GET /api/dosen-pa/my-advisees                       │
│  2. Lihat KRS mahasiswa                                 │
│     GET /api/dosen-pa/advisee-krs/{mhs_id}?ta_id=...   │
│  3. Approve/Reject KRS                                  │
│     PUT /api/krs/enrollments/{id}/status                │
│  4. Input nilai setelah semester selesai                 │
│     PUT /api/krs/enrollments/{id}/nilai                 │
└─────────────────────────────────────────────────────────┘
```

### 5.3 Alur Manajemen Aset

```
┌─────────────────────────────────────────────────────────┐
│  STAF BAUM / ADMIN                                      │
│                                                         │
│  1. Buat Jenis Aset (POST /api/aset/jenis)              │
│  2. Buat Ruangan (POST /api/aset/ruangan)               │
│  3. Tambah Item Aset (POST /api/aset/item)              │
│  4. Kelola histori (pindah, pinjam, kondisi)            │
│  5. Catat biaya aset (POST /api/aset/item/{id}/biaya)   │
│  6. Kelola barang habis pakai (stok masuk/keluar)       │
│  7. Monitor dashboard:                                  │
│     - GET /api/aset/summary-kondisi                     │
│     - GET /api/aset/konsumsi/low-stock                  │
│     - GET /api/aset/biaya/summary                       │
└─────────────────────────────────────────────────────────┘
```

### 5.4 Alur Fleet Management

```
┌─────────────────────────────────────────────────────────┐
│  USER (Siapa saja yang login)                           │
│                                                         │
│  1. Cari kendaraan tersedia                             │
│     GET /api/fleet/kendaraan-tersedia?start=...&end=... │
│  2. Booking kendaraan                                   │
│     POST /api/fleet/bookings                            │
│  3. Lihat booking saya                                  │
│     GET /api/fleet/my-bookings                          │
└─────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────┐
│  ADMIN FLEET (STAF_BAUM)                                │
│                                                         │
│  1. Lihat semua booking (GET /api/fleet/bookings)       │
│  2. Approve/Reject booking                              │
│  3. Start trip (catat odometer awal)                    │
│  4. End trip (catat odometer akhir + BBM)               │
│  5. Catat servis kendaraan                              │
│  6. Monitor:                                            │
│     - GET /api/fleet/bookings/summary                   │
│     - GET /api/fleet/kendaraan/{id}/summary             │
└─────────────────────────────────────────────────────────┘
```

### 5.5 Alur SDM & Absensi

```
┌─────────────────────────────────────────────────────────┐
│  KARYAWAN                                               │
│                                                         │
│  1. Enroll wajah (POST /api/sdm/absensi/enroll-wajah)   │
│  2. Clock-in harian (POST /api/sdm/absensi/clock-in)    │
│  3. Clock-out (POST /api/sdm/absensi/clock-out)         │
│  4. Ajukan cuti (POST /api/sdm/cuti/ajukan)             │
│  5. Ajukan ijin (POST /api/sdm/ijin/ajukan)             │
│  6. Cek kuota cuti (GET /api/sdm/cuti/kuota-saya)       │
│  7. Lihat rekap absensi (GET /api/sdm/absensi/rekap-saya)│
└─────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────┐
│  ADMIN SDM (STAF_BASDM)                                 │
│                                                         │
│  1. Kelola pegawai (CRUD)                               │
│  2. Audit biometrik wajah                               │
│  3. Approve/Reject cuti & ijin                          │
│  4. Input rekap manual absensi                          │
│  5. Lihat laporan:                                      │
│     - Laporan harian (/api/sdm/absensi/laporan-harian)  │
│     - Laporan bulanan (/api/sdm/absensi/laporan-bulanan)│
│  6. Kelola riwayat karir (pendidikan, SK, JAD, serdos)  │
│  7. Buat surat tugas & SPPD                            │
└─────────────────────────────────────────────────────────┘
```

### 5.6 Alur Pra-KBM (Sebelum Perkuliahan Dimulai)

```
┌─────────────────────────────────────────────────────────┐
│  ADMIN AKADEMIK                                         │
│                                                         │
│  1. Buat Jadwal Kuliah (atau import CSV)                │
│  2. Plot ruangan ke jadwal kuliah                       │
│     POST /api/akademik/plot-jadwal-ruangan              │
└─────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────┐
│  DOSEN                                                  │
│                                                         │
│  1. Upload file RPS per mata kuliah                     │
│     POST /api/matakuliah/{id}/upload-rps                │
│  2. Isi RPS Header + Mingguan                           │
│  3. Set rencana penilaian (bobot)                       │
│     PUT /api/akademik/jadwal-kuliah/{id}/rencana-penilaian │
│  4. Upload kontrak perkuliahan                          │
└─────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────┐
│  KAPRODI                                                │
│                                                         │
│  1. Verifikasi RPS                                      │
│     PUT /api/matakuliah/{id}/verifikasi-rps             │
│     Body: { "status_verifikasi": "Disetujui" }         │
└─────────────────────────────────────────────────────────┘
```

---

## 6. Catatan untuk Frontend Developer

### Format Tanggal
- **Date only:** `YYYY-MM-DD` (contoh: `"2025-09-01"`)
- **DateTime (RFC3339):** `YYYY-MM-DDTHH:MM:SS+07:00` (contoh: `"2025-09-01T08:00:00+07:00"`)
- **Time with timezone:** `HH:MM+07:00` atau `HH:MM` (otomatis WIB)

### Format Angka Desimal
- Gunakan string untuk desimal: `"85.50"`, `"3.50"`, `"8500000.00"`

### Upload File
- Gunakan `multipart/form-data`
- Field name: `file` (kecuali disebutkan lain)
- Setelah upload, path file bisa diakses via `GET /api/files/{path}`

### CORS
Frontend yang diizinkan:
- `http://localhost:5173` (Vite dev)
- `http://localhost:8081`
- `https://sikt.maxtion.co.id`
- `https://satria.respati-tasikmalaya.ac.id`

### UUID
- Semua ID menggunakan format UUID v4
- Contoh: `"550e8400-e29b-41d4-a716-446655440000"`

### Pagination
- Saat ini API belum mengimplementasi pagination. Semua list endpoint mengembalikan seluruh data.

---

*Dokumentasi ini di-generate berdasarkan source code backend SIKT.*
