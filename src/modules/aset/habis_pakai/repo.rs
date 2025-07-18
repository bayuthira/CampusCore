use super::model::{AsetHabisPakai, AsetHabisPakaiPayload,StokTransaksiPayload};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

    pub async fn create_repo(pool: &DbPool, payload: AsetHabisPakaiPayload) -> Result<AsetHabisPakai, AppError> {
        let item = sqlx::query_as!(
            AsetHabisPakai,
            "INSERT INTO aset_habis_pakai (nama_barang, deskripsi, satuan, batas_minimum_stok) VALUES ($1, $2, $3, $4) RETURNING *",
            payload.nama_barang, payload.deskripsi, payload.satuan, payload.batas_minimum_stok
        ).fetch_one(pool).await?;
        Ok(item)
    }

    pub async fn get_all_repo(pool: &DbPool) -> Result<Vec<AsetHabisPakai>, AppError> {
        let list = sqlx::query_as!(AsetHabisPakai, "SELECT * FROM aset_habis_pakai ORDER BY nama_barang ASC").fetch_all(pool).await?;
        Ok(list)
    }

    pub async fn get_by_id_repo(pool: &DbPool, id: Uuid) -> Result<AsetHabisPakai, AppError> {
        let item = sqlx::query_as!(AsetHabisPakai, "SELECT * FROM aset_habis_pakai WHERE id = $1", id).fetch_one(pool).await?;
        Ok(item)
    }

    pub async fn update_repo(pool: &DbPool, id: Uuid, payload: AsetHabisPakaiPayload) -> Result<AsetHabisPakai, AppError> {
        let item = sqlx::query_as!(
            AsetHabisPakai,
            "UPDATE aset_habis_pakai SET nama_barang = $1, deskripsi = $2, satuan = $3, batas_minimum_stok = $4, updated_at = now() WHERE id = $5 RETURNING *",
            payload.nama_barang, payload.deskripsi, payload.satuan, payload.batas_minimum_stok, id
        ).fetch_one(pool).await?;
        Ok(item)
    }

    pub async fn delete_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
        let rows_affected = sqlx::query!("DELETE FROM aset_habis_pakai WHERE id = $1", id).execute(pool).await?.rows_affected();
        if rows_affected == 0 { return Err(sqlx::Error::RowNotFound.into()); }
        Ok(())
    }

pub async fn tambah_stok_repo(pool: &DbPool, id: Uuid, payload: StokTransaksiPayload, user_id: Uuid) -> Result<AsetHabisPakai, AppError> {
    let mut tx = pool.begin().await?;

    // Kunci baris aset untuk mencegah update serentak (race condition)
    let aset = sqlx::query_as!(AsetHabisPakai, "SELECT * FROM aset_habis_pakai WHERE id = $1 FOR UPDATE", id)
        .fetch_one(&mut *tx).await?;

    let saldo_sebelum = aset.stok;
    let saldo_setelah = saldo_sebelum + payload.jumlah;

    // Masukkan ke histori
    sqlx::query!(
        "INSERT INTO histori_stok (aset_id, tipe_transaksi, jumlah, saldo_sebelum, saldo_setelah, user_aksi_id, catatan) VALUES ($1, 'Pembelian', $2, $3, $4, $5, $6)",
        id, payload.jumlah, saldo_sebelum, saldo_setelah, user_id, payload.catatan
    ).execute(&mut *tx).await?;

    // Update stok di tabel utama
    let updated_aset = sqlx::query_as!(
        AsetHabisPakai,
        "UPDATE aset_habis_pakai SET stok = $1, updated_at = now() WHERE id = $2 RETURNING *",
        saldo_setelah, id
    ).fetch_one(&mut *tx).await?;

    tx.commit().await?;
    Ok(updated_aset)
}

pub async fn ambil_stok_repo(pool: &DbPool, id: Uuid, payload: StokTransaksiPayload, user_id: Uuid) -> Result<AsetHabisPakai, AppError> {
    let mut tx = pool.begin().await?;

    let aset = sqlx::query_as!(AsetHabisPakai, "SELECT * FROM aset_habis_pakai WHERE id = $1 FOR UPDATE", id)
        .fetch_one(&mut *tx).await?;

    // Validasi stok
    if aset.stok < payload.jumlah {
        return Err(AppError::Forbidden("Stok tidak mencukupi.".to_string()));
    }

    let saldo_sebelum = aset.stok;
    let saldo_setelah = saldo_sebelum - payload.jumlah;
    let jumlah_diambil = -payload.jumlah; // Jumlah di histori dicatat sebagai negatif

    // Masukkan ke histori
    sqlx::query!(
        "INSERT INTO histori_stok (aset_id, tipe_transaksi, jumlah, saldo_sebelum, saldo_setelah, user_aksi_id, catatan) VALUES ($1, 'Pengambilan', $2, $3, $4, $5, $6)",
        id, jumlah_diambil, saldo_sebelum, saldo_setelah, user_id, payload.catatan
    ).execute(&mut *tx).await?;

    // Update stok di tabel utama
    let updated_aset = sqlx::query_as!(
        AsetHabisPakai,
        "UPDATE aset_habis_pakai SET stok = $1, updated_at = now() WHERE id = $2 RETURNING *",
        saldo_setelah, id
    ).fetch_one(&mut *tx).await?;

    tx.commit().await?;
    Ok(updated_aset)
}