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
    server::run_server(db).await
}
