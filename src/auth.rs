// src/auth.rs
use anyhow::{Result, anyhow};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, PasswordHash};
use rand_core::OsRng;
use sqlx::Row;

use crate::db::Db;

#[derive(Clone)]
pub struct AuthService {
    db: Db,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
    pub role: String,
}

impl AuthService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn register_user(
        &self,
        username: &str,
        password: &str,
        confirm: &str,
    ) -> Result<()> {
        if password != confirm {
            return Err(anyhow!("Passwords do not match"));
        }

        if self.db.get_user_by_username(username).await?.is_some() {
            return Err(anyhow!("Username already exists"));
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon = Argon2::default();

        let password_hash = argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Password hashing failed: {e}"))?
            .to_string();

        self.db.create_user(username, &password_hash, "user").await?;
        Ok(())
    }

    pub async fn login_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthUser> {
        let row = self
            .db
            .get_user_by_username(username)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        let stored_hash: String = row.try_get("password_hash")?;

        let parsed_hash = PasswordHash::new(&stored_hash)
            .map_err(|e| anyhow!("Password hash parse error: {e}"))?;

        let argon = Argon2::default();
        argon.verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow!("Invalid password"))?;

        Ok(AuthUser {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            role: row.try_get("role")?,
        })
    }

    pub async fn admin_create_user(
        &self,
        username: &str,
        password: &str,
        role: &str,
    ) -> Result<()> {
        let salt = SaltString::generate(&mut OsRng);
        let argon = Argon2::default();

        let password_hash = argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Password hashing failed: {e}"))?
            .to_string();

        self.db.create_user(username, &password_hash, role).await?;
        Ok(())
    }
}
