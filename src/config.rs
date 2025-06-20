// src/config.rs
use once_cell::sync::Lazy;
use std::env;

// Struct untuk menampung konfigurasi aplikasi kita
pub struct Config {
    pub database_url: String,
    pub server_address: String,
}

// Gunakan `Lazy` dari `once_cell` untuk memastikan konfigurasi
// hanya dimuat sekali saat pertama kali diakses.
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    // Memuat variabel dari file .env
    dotenvy::dotenv().expect("Failed to read .env file");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let server_address = env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set");

    Config {
        database_url,
        server_address,
    }
});