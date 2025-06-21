// src/config.rs
use once_cell::sync::Lazy;
use std::env;

// Struct untuk menampung konfigurasi aplikasi kita
pub struct Config {
    pub database_url: String,
    pub server_address: String,
    pub jwt_secret: String,
    pub jwt_expires_in: i64,
}

// Gunakan `Lazy` dari `once_cell` untuk memastikan konfigurasi
// hanya dimuat sekali saat pertama kali diakses.
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    // Memuat variabel dari file .env
    dotenvy::dotenv().expect("Failed to read .env file");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let server_address = env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_expires_in = env::var("JWT_EXPIRES_IN")
        .expect("JWT_EXPIRES_IN must be set")
        .parse::<i64>()
        .expect("JWT_EXPIRES_IN must be a number");

    Config {
        database_url,
        server_address,
        jwt_secret,
        jwt_expires_in,
    }
});
