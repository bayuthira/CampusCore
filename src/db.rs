use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

// Tipe alias untuk Pool database agar lebih mudah digunakan
pub type DbPool = Pool<Postgres>;

pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5) // Jumlah koneksi maksimum
        .connect(database_url)
        .await
}