// src/menu.rs
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;

pub async fn print_welcome(writer: &mut OwnedWriteHalf) -> Result<()> {
    writer
        .write_all(
            b"========================================\r\n\
Welcome to cloud-calc\r\n\
========================================\r\n\
1) Login\r\n\
2) Register\r\n\
3) Quit\r\n\
choice> ",
        )
        .await?;
    writer.flush().await?;
    Ok(())
}

pub async fn print_user_panel(writer: &mut OwnedWriteHalf) -> Result<()> {
    writer
        .write_all(
            b"========================================\r\n\
User Panel\r\n\
========================================\r\n\
1) Calculator\r\n\
2) My Usage\r\n\
3) My History\r\n\
4) Account Info\r\n\
5) Logout\r\n\
choice> ",
        )
        .await?;
    writer.flush().await?;
    Ok(())
}

pub async fn print_admin_panel(writer: &mut OwnedWriteHalf) -> Result<()> {
    writer
        .write_all(
            b"========================================\r\n\
Admin Panel\r\n\
========================================\r\n\
1) List users\r\n\
2) Create user\r\n\
3) Change user role\r\n\
4) Delete user\r\n\
5) View usage stats\r\n\
6) Back\r\n\
choice> ",
        )
        .await?;
    writer.flush().await?;
    Ok(())
}
