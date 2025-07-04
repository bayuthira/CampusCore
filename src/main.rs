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
use tower_http::cors::{CorsLayer, Any};
use axum::http::{HeaderValue, Method}; // <-- 1. TAMBAHKAN USE STATEMENT INI

#[tokio::main]
async fn main() {
    // Memuat konfigurasi
    println!("->> LOADING CONFIGURATION");
    let _ = &CONFIG.server_address; 

    // Membuat koneksi pool ke database
    println!("->> INITIALIZING DATABASE POOL");
    let pool = create_pool(&CONFIG.database_url)
        .await
        .expect("Failed to create database pool");

    // <-- 2. DEFINISIKAN CORS DI SINI -->
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS])
        .allow_headers(Any);

    // Membuat router aplikasi dan menerapkan layer CORS
    // <-- 3. TERAPKAN LAYER PADA APP -->
    let app = create_router(pool).layer(cors);

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