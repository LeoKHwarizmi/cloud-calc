// src/calculator.rs
use anyhow::{Result, anyhow};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;

use crate::db::Db;
use std::time::Instant;

pub async fn run_calculator(
    writer: &mut OwnedWriteHalf,
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
    db: Db,
    user_id: i32,
) -> Result<()> {
    let mut line = String::new();

    writer
        .write_all(
            b"--- Calculator ---
Type 'exit' to return.
expr> ",
        )
        .await?;

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line).await?;
        if bytes == 0 {
            return Err(anyhow!("Client disconnected"));
        }

        let expr = line.trim();

        if expr == "exit" {
            return Ok(());
        }

        let start = Instant::now();

        match meval::eval_str(expr) {
            Ok(result) => {
                let cpu_seconds = start.elapsed().as_secs_f64();
                let ram_bytes = get_rss_bytes().unwrap_or(0);

                db.add_usage(user_id, cpu_seconds, ram_bytes, 1).await?;
                db.insert_calculation(user_id, expr, &result.to_string()).await?;

                writer
                    .write_all(format!("= {}\nexpr> ", result).as_bytes())
                    .await?;
            }
            Err(_) => {
                writer
                    .write_all(b"Invalid expression.\nexpr> ")
                    .await?;
            }
        }
    }
}

fn get_rss_bytes() -> Result<i64, procfs::ProcError> {
    let me = procfs::process::Process::myself()?;
    Ok((me.stat()?.rss * 4096) as i64)
}
