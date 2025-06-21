// src/main.rs
mod config;
mod db;
mod errors;
mod handlers;
mod routes;

mod models;
mod repositories;
mod auth;

use crate::config::CONFIG;
use crate::db::create_pool;
use crate::routes::create_router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Memuat konfigurasi
    // Akses via CONFIG.database_url atau CONFIG.server_address
    println!("->> LOADING CONFIGURATION");
    let _ = &CONFIG.server_address; // Memaksa lazy_static untuk inisialisasi

    // Membuat koneksi pool ke database
    println!("->> INITIALIZING DATABASE POOL");
    let pool = create_pool(&CONFIG.database_url)
        .await
        .expect("Failed to create database pool");

    // Membuat router aplikasi
    let app = create_router(pool);

    // Mendapatkan alamat server dari konfigurasi
    let addr: SocketAddr = CONFIG.server_address.parse()
        .expect("Invalid server address format");

    println!("->> LISTENING on http://{}", addr);

    // Membuat listener dan menjalankan server
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}