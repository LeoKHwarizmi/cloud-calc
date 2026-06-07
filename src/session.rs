// src/session.rs
use anyhow::Result;
use axum::extract::ws::{WebSocket, Message};
use futures_util::StreamExt;

use crate::menu;
use crate::db::Db;
use crate::auth::{AuthService, AuthUser};
use crate::calculator::run_calculator_ws;

// ---------------------------------------------------------
// MAIN WS SESSION
// ---------------------------------------------------------
pub async fn handle_ws_session(socket: &mut WebSocket, db: Db) -> Result<()> {
    let auth = AuthService::new(db.clone());

    send(socket,
"========================================
Welcome to cloud-calc
========================================
1) Login
2) Register
3) Quit
choice> ").await?;

    loop {
        let choice = recv(socket).await?;

        match choice.as_str() {
            // LOGIN
            "1" => {
                send(socket, "--- Login ---\nUsername: ").await?;
                let username = recv(socket).await?;

                send(socket, "Password: ").await?;
                let password = recv(socket).await?;

                match auth.login_user(&username, &password).await {
                    Ok(user) => {
                        send(socket, &format!("Login successful! Role: {}\n", user.role)).await?;

                        if user.role == "admin" {
                            admin_panel_ws(socket, db.clone(), user).await?;
                        } else {
                            user_panel_ws(socket, db.clone(), user.id).await?;
                        }

                        send(socket,
"========================================
Welcome to cloud-calc
1) Login
2) Register
3) Quit
choice> ").await?;
                    }
                    Err(e) => {
                        send(socket, &format!("Login failed: {e}\nchoice> ")).await?;
                    }
                }
            }

            // REGISTER
            "2" => {
                send(socket, "--- Register ---\nChoose username: ").await?;
                let username = recv(socket).await?;

                send(socket, "Choose password: ").await?;
                let password = recv(socket).await?;

                send(socket, "Confirm password: ").await?;
                let confirm = recv(socket).await?;

                match auth.register_user(&username, &password, &confirm).await {
                    Ok(_) => send(socket, "Account created!\nchoice> ").await?,
                    Err(e) => send(socket, &format!("Register failed: {e}\nchoice> ")).await?,
                }
            }

            // QUIT
            "3" => {
                send(socket, "Goodbye.\n").await?;
                break;
            }

            _ => {
                send(socket, "Invalid choice.\nchoice> ").await?;
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------
// USER PANEL
// ---------------------------------------------------------
async fn user_panel_ws(
    socket: &mut WebSocket,
    db: Db,
    user_id: i32,
) -> Result<()> {
    loop {
        send(socket,
"========================================
User Panel
========================================
1) Calculator
2) My Usage
3) My History
4) Account Info
5) Logout
choice> ").await?;

        let choice = recv(socket).await?;

        match choice.as_str() {
            // CALCULATOR
            "1" => {
                run_calculator_ws(socket, db.clone(), user_id).await?;
            }

            // USAGE
            "2" => {
                let (cpu, ram, count) = db.get_usage(user_id).await?;
                send(socket, &format!("CPU: {cpu} sec\nRAM: {ram} bytes\nCalculations: {count}\n")).await?;
            }

            // HISTORY
            "3" => {
                let history = db.get_history(user_id).await?;
                for (expr, res) in history {
                    send(socket, &format!("{expr} = {res}\n")).await?;
                }
            }

            // ACCOUNT INFO
            "4" => {
                send(socket, &format!("Your user ID: {user_id}\n")).await?;
            }

            // LOGOUT
            "5" => {
                send(socket, "Logging out...\n").await?;
                break;
            }

            _ => {
                send(socket, "Invalid choice.\n").await?;
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------
// ADMIN PANEL
// ---------------------------------------------------------
async fn admin_panel_ws(
    socket: &mut WebSocket,
    db: Db,
    _admin: AuthUser,
) -> Result<()> {
    loop {
        send(socket,
"========================================
Admin Panel
========================================
1) List users
2) Create user
3) Change user role
4) Delete user
5) View usage stats
6) Back
choice> ").await?;

        let choice = recv(socket).await?;

        match choice.as_str() {
            // LIST USERS
            "1" => {
                let users = db.list_users().await?;
                for (id, username, role) in users {
                    send(socket, &format!("ID: {id}, {username} ({role})\n")).await?;
                }
            }

            // CREATE USER
            "2" => {
                send(socket, "New username: ").await?;
                let username = recv(socket).await?;

                send(socket, "Password: ").await?;
                let password = recv(socket).await?;

                send(socket, "Role (user/admin): ").await?;
                let role = recv(socket).await?;

                let auth = crate::auth::AuthService::new(db.clone());
                match auth.admin_create_user(&username, &password, &role).await {
                    Ok(_) => send(socket, "User created.\n").await?,
                    Err(e) => send(socket, &format!("Failed: {e}\n")).await?,
                }
            }

            // CHANGE ROLE
            "3" => {
                send(socket, "User ID to change role: ").await?;
                let id: i32 = recv(socket).await?.parse().unwrap_or(-1);

                send(socket, "New role (user/admin): ").await?;
                let role = recv(socket).await?;

                db.update_user_role(id, &role).await?;
                send(socket, "Role updated.\n").await?;
            }

            // DELETE USER
            "4" => {
                send(socket, "User ID to delete: ").await?;
                let id: i32 = recv(socket).await?.parse().unwrap_or(-1);

                db.delete_user(id).await?;
                send(socket, "User deleted.\n").await?;
            }

            // VIEW USAGE
            "5" => {
                let usage = db.get_all_usage().await?;
                for (username, cpu, ram, count) in usage {
                    send(socket, &format!("{username}: CPU={cpu} sec, RAM={ram} bytes, Calcs={count}\n")).await?;
                }
            }

            // BACK
            "6" => {
                send(socket, "Back to main menu.\n").await?;
                break;
            }

            _ => {
                send(socket, "Invalid choice.\n").await?;
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------
// WebSocket helpers
// ---------------------------------------------------------
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
