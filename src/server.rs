// src/server.rs
use anyhow::Result;
use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket},
    routing::get,
    Router,
};
use futures_util::StreamExt;
use tokio::net::TcpListener;

use crate::session::handle_ws_session;
use crate::db::Db;

pub async fn run_ws_server(db: Db) -> Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "10000".to_string());
    let addr = format!("0.0.0.0:{port}");

    tracing::info!("WebSocket server listening on {addr}");

    let app = Router::new().route(
        "/",
        get(move |ws: WebSocketUpgrade| {
            let db = db.clone();
            async move { ws.on_upgrade(move |socket| handle_ws(socket, db)) }
        }),
    );

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_ws(mut socket: WebSocket, db: Db) {
    if let Err(e) = handle_ws_session(&mut socket, db).await {
        tracing::warn!("WS session error: {e:?}");
    }
}
