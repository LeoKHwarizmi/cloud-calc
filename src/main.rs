// src/main.rs
mod server;
mod session;
mod menu;
mod auth;
mod db;
mod calculator;
mod admin;
mod user;
mod utils;

use anyhow::Result;
use db::Db;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let db = Db::connect_from_env().await?;

    // Only WebSocket server (Render Web Service)
    server::run_ws_server(db.clone()).await?;

    Ok(())
}
