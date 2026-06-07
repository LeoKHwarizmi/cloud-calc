use anyhow::Result;
use thrussh::*;
use thrussh_keys::key;
use tokio::net::TcpStream;

use crate::db::Db;
use crate::session::handle_client;

#[derive(Clone)]
pub struct Server {
    pub db: Db,
}

impl server::Server for Server {
    type Handler = Handler;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        Handler {
            db: self.db.clone(),
        }
    }
}

pub struct Handler {
    pub db: Db,
}

#[async_trait::async_trait]
impl server::Handler for Handler {
    type Error = anyhow::Error;

    async fn auth_password(
        self,
        user: &str,
        password: &str,
    ) -> Result<(Self, server::Auth)> {
        // You can either:
        // 1) accept any SSH user and do app-level login later
        // 2) wire this to your AuthService
        // For now, accept all:
        Ok((self, server::Auth::Accept))
    }

    async fn channel_open_session(
        mut self,
        channel: ChannelId,
        session: server::Session,
    ) -> Result<(Self, server::Session)> {
        // When SSH session opens, we bridge to your existing TCP logic
        tokio::spawn(async move {
            // Connect to internal TCP server (your existing handle_client)
            if let Ok(stream) = TcpStream::connect("127.0.0.1:9000").await {
                let _ = handle_client(stream, self.db.clone()).await;
            }
        });

        Ok((self, session))
    }
}

pub async fn run_ssh_server(db: Db) -> Result<()> {
    let mut config = server::Config::default();
    config.keys.push(key::KeyPair::generate_ed25519().unwrap());
    config.auth_rejection_time = std::time::Duration::from_secs(0);
    let config = std::sync::Arc::new(config);

    let server = Server { db };

    server::run(config, "0.0.0.0:2222", server).await?;
    Ok(())
}
