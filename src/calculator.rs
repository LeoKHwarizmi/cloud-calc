// src/calculator.rs
use anyhow::{Result, anyhow};
use axum::extract::ws::{WebSocket, Message};
use futures_util::StreamExt;

use crate::db::Db;
use std::time::Instant;

pub async fn run_calculator_ws(
    socket: &mut WebSocket,
    db: Db,
    user_id: i32,
) -> Result<()> {
    send(socket, "--- Calculator ---\nType 'exit' to return.\nexpr> ").await?;

    loop {
        let expr = recv(socket).await?;

        if expr.is_empty() {
            return Err(anyhow!("Client disconnected"));
        }

        if expr == "exit" {
            return Ok(());
        }

        let start = Instant::now();

        match meval::eval_str(&expr) {
            Ok(result) => {
                let cpu_seconds = start.elapsed().as_secs_f64();
                let ram_bytes = get_rss_bytes().unwrap_or(0);

                db.add_usage(user_id, cpu_seconds, ram_bytes, 1).await?;
                db.insert_calculation(user_id, &expr, &result.to_string()).await?;

                send(socket, &format!("= {}\nexpr> ", result)).await?;
            }
            Err(_) => {
                send(socket, "Invalid expression.\nexpr> ").await?;
            }
        }
    }
}

fn get_rss_bytes() -> Result<i64, procfs::ProcError> {
    let me = procfs::process::Process::myself()?;
    Ok((me.stat()?.rss * 4096) as i64)
}

// -----------------------------
// WebSocket helpers
// -----------------------------
async fn send(socket: &mut WebSocket, text: &str) -> Result<()> {
    socket.send(Message::Text(text.to_string())).await?;
    Ok(())
}

async fn recv(socket: &mut WebSocket) -> Result<String> {
    while let Some(msg) = socket.next().await {
        match msg? {
            Message::Text(t) => return Ok(t.trim().to_string()),
            Message::Close(_) => return Ok("".into()),
            _ => {}
        }
    }
    Ok("".into())
}
