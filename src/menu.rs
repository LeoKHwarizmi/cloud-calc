// src/menu.rs
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;

pub async fn print_welcome(writer: &mut OwnedWriteHalf) -> Result<()> {
    writer
        .write_all(
            b"========================================
Welcome to cloud-calc
========================================
1) Login
2) Register
3) Quit
choice>
",
        )
        .await?;
    Ok(())
}

pub async fn print_user_panel(writer: &mut OwnedWriteHalf) -> Result<()> {
    writer
        .write_all(
            b"========================================
User Panel
========================================
1) Calculator
2) My Usage
3) My History
4) Account Info
5) Logout
choice>
",
        )
        .await?;
    Ok(())
}

pub async fn print_admin_panel(writer: &mut OwnedWriteHalf) -> Result<()> {
    writer
        .write_all(
            b"========================================
Admin Panel
========================================
1) List users
2) Create user
3) Change user role
4) Delete user
5) View usage stats
6) Back
choice>
",
        )
        .await?;
    Ok(())
}
