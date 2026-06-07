// src/server.rs
use tokio::net::TcpListener;
use anyhow::Result;

use crate::session::handle_client;
use crate::db::Db;

pub async fn run_server(db: Db) -> Result<()> {
    // Render provides PORT env var
    let port = std::env::var("PORT").unwrap_or_else(|_| "9000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("cloud-calc listening on {addr}");

    let listener = TcpListener::bind(&addr).await?;

    loop {
        let (stream, peer) = listener.accept().await?;
        let db_clone = db.clone();

        tokio::spawn(async move {
            tracing::info!("New connection from {peer}");
            if let Err(e) = handle_client(stream, db_clone).await {
                tracing::warn!("client error: {e:?}");
            }
        });
    }
}
