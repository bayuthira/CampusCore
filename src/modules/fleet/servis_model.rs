use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::{Date};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ServisDetail {
    pub id: Uuid,
    pub kendaraan_id: Uuid,
    pub tanggal_servis: Date,
    pub odometer_saat_servis: i32,
    pub deskripsi: String,
    pub biaya: Decimal,
    pub user_pencatat_id: Uuid,
    pub nama_pencatat: String,
}

#[derive(Debug, Deserialize)]
pub struct ServisPayload {
    pub tanggal_servis: Date,
    pub odometer_saat_servis: i32,
    pub deskripsi: String,
    pub biaya: Decimal,
}
