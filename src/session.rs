// src/session.rs
use anyhow::Result;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};

use crate::menu;
use crate::db::Db;
use crate::auth::{AuthService, AuthUser};
use crate::calculator::run_calculator;

pub async fn handle_client(stream: TcpStream, db: Db) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    let auth = AuthService::new(db.clone());

    menu::print_welcome(&mut writer).await?;

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line).await?;
        if bytes == 0 {
            break;
        }

        let choice = line.trim();

        match choice {
            // -----------------------------
            // LOGIN
            // -----------------------------
            "1" => {
                writer.write_all(b"--- Login ---\nUsername: ").await?;
                line.clear();
                reader.read_line(&mut line).await?;
                let username = line.trim().to_string();

                writer.write_all(b"Password: ").await?;
                line.clear();
                reader.read_line(&mut line).await?;
                let password = line.trim().to_string();

                match auth.login_user(&username, &password).await {
                    Ok(user) => {
                        writer
                            .write_all(
                                format!("Login successful! Role: {}\n", user.role).as_bytes(),
                            )
                            .await?;

                        if user.role == "admin" {
                            admin_panel(&mut writer, &mut reader, db.clone(), user).await?;
                        } else {
                            user_panel(&mut writer, &mut reader, db.clone(), user.id).await?;
                        }

                        menu::print_welcome(&mut writer).await?;
                    }
                    Err(e) => {
                        writer
                            .write_all(format!("Login failed: {e}\nchoice>\n").as_bytes())
                            .await?;
                    }
                }
            }

            // -----------------------------
            // REGISTER
            // -----------------------------
            "2" => {
                writer.write_all(b"--- Register ---\nChoose username: ").await?;
                line.clear();
                reader.read_line(&mut line).await?;
                let username = line.trim().to_string();

                writer.write_all(b"Choose password: ").await?;
                line.clear();
                reader.read_line(&mut line).await?;
                let password = line.trim().to_string();

                writer.write_all(b"Confirm password: ").await?;
                line.clear();
                reader.read_line(&mut line).await?;
                let confirm = line.trim().to_string();

                match auth.register_user(&username, &password, &confirm).await {
                    Ok(_) => writer.write_all(b"Account created!\nchoice>\n").await?,
                    Err(e) => writer
                        .write_all(format!("Register failed: {e}\nchoice>\n").as_bytes())
                        .await?,
                }
            }

            // -----------------------------
            // QUIT
            // -----------------------------
            "3" => {
                writer.write_all(b"Goodbye.\n").await?;
                break;
            }

            _ => {
                writer.write_all(b"Invalid choice.\nchoice>\n").await?;
            }
        }
    }

    Ok(())
}

// -----------------------------
// USER PANEL
// -----------------------------
async fn user_panel(
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
    db: Db,
    user_id: i32,
) -> Result<()> {
    loop {
        menu::print_user_panel(writer).await?;

        let mut line = String::new();
        reader.read_line(&mut line).await?;
        let choice = line.trim();

        match choice {
            // CALCULATOR (now with CPU/RAM tracking)
            "1" => {
                run_calculator(writer, reader, db.clone(), user_id).await?;
            }

            // USAGE
            "2" => {
                let (cpu, ram, count) = db.get_usage(user_id).await?;
                writer
                    .write_all(
                        format!(
                            "CPU: {cpu} sec\nRAM: {ram} bytes\nCalculations: {count}\n"
                        )
                        .as_bytes(),
                    )
                    .await?;
            }

            // HISTORY
            "3" => {
                let history = db.get_history(user_id).await?;
                for (expr, res) in history {
                    writer
                        .write_all(format!("{expr} = {res}\n").as_bytes())
                        .await?;
                }
            }

            // ACCOUNT INFO
            "4" => {
                writer
                    .write_all(format!("Your user ID: {user_id}\n").as_bytes())
                    .await?;
            }

            // LOGOUT
            "5" => {
                writer.write_all(b"Logging out...\n").await?;
                break;
            }

            _ => {
                writer.write_all(b"Invalid choice.\n").await?;
            }
        }
    }

    Ok(())
}

// -----------------------------
// ADMIN PANEL
// -----------------------------
async fn admin_panel(
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
    db: Db,
    _admin: AuthUser,
) -> Result<()> {
    loop {
        menu::print_admin_panel(writer).await?;

        let mut line = String::new();
        reader.read_line(&mut line).await?;
        let choice = line.trim();

        match choice {
            // LIST USERS
            "1" => {
                let users = db.list_users().await?;
                for (id, username, role) in users {
                    writer
                        .write_all(format!("ID: {id}, {username} ({role})\n").as_bytes())
                        .await?;
                }
            }

            // CREATE USER
            "2" => {
                writer.write_all(b"New username: ").await?;
                line.clear();
                reader.read_line(&mut line).await?;
                let username = line.trim().to_string();

                writer.write_all(b"Password: ").await?;
                line.clear();
                reader.read_line(&mut line).await?;
                let password = line.trim().to_string();

                writer.write_all(b"Role (user/admin): ").await?;
                line.clear();
                reader.read_line(&mut line).await?;
                let role = line.trim().to_string();

                let auth = AuthService::new(db.clone());
                match auth.admin_create_user(&username, &password, &role).await {
                    Ok(_) => writer.write_all(b"User created.\n").await?,
                    Err(e) => writer
                        .write_all(format!("Failed: {e}\n").as_bytes())
                        .await?,
                }
            }

            // CHANGE ROLE
            "3" => {
                writer.write_all(b"User ID to change role: ").await?;
                line.clear();
                reader.read_line(&mut line).await?;
                let id: i32 = line.trim().parse().unwrap_or(-1);

                writer.write_all(b"New role (user/admin): ").await?;
                line.clear();
                reader.read_line(&mut line).await?;
                let role = line.trim().to_string();

                db.update_user_role(id, &role).await?;
                writer.write_all(b"Role updated.\n").await?;
            }

            // DELETE USER
            "4" => {
                writer.write_all(b"User ID to delete: ").await?;
                line.clear();
                reader.read_line(&mut line).await?;
                let id: i32 = line.trim().parse().unwrap_or(-1);

                db.delete_user(id).await?;
                writer.write_all(b"User deleted.\n").await?;
            }

            // VIEW USAGE STATS
            "5" => {
                let usage = db.get_all_usage().await?;
                for (username, cpu, ram, count) in usage {
                    writer
                        .write_all(
                            format!(
                                "{username}: CPU={cpu} sec, RAM={ram} bytes, Calcs={count}\n"
                            )
                            .as_bytes(),
                        )
                        .await?;
                }
            }

            // BACK
            "6" => {
                writer.write_all(b"Back to main menu.\n").await?;
                break;
            }

            _ => {
                writer.write_all(b"Invalid choice.\n").await?;
            }
        }
    }

    Ok(())
}
