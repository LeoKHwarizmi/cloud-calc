// src/db.rs
use anyhow::Result;
use sqlx::{PgPool, postgres::PgRow};
use sqlx::Row;

#[derive(Clone)]
pub struct Db {
    pub pool: PgPool,
}

impl Db {
    pub async fn connect_from_env() -> Result<Self> {
        let url = std::env::var("DATABASE_URL")?;
        let pool = PgPool::connect(&url).await?;
        Ok(Self { pool })
    }

    // -----------------------------
    // USERS
    // -----------------------------
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<PgRow>> {
        let row = sqlx::query("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    pub async fn create_user(
        &self,
        username: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO users (username, password_hash, role) VALUES ($1, $2, $3)",
        )
        .bind(username)
        .bind(password_hash)
        .bind(role)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // -----------------------------
    // HISTORY
    // -----------------------------
    pub async fn insert_calculation(
        &self,
        user_id: i32,
        expression: &str,
        result: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO calculations (user_id, expression, result) VALUES ($1, $2, $3)",
        )
        .bind(user_id)
        .bind(expression)
        .bind(result)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_history(&self, user_id: i32) -> Result<Vec<(String, String)>> {
        let rows = sqlx::query(
            "SELECT expression, result FROM calculations WHERE user_id = $1 ORDER BY id DESC LIMIT 20",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::new();
        for row in rows {
            out.push((
                row.try_get("expression")?,
                row.try_get("result")?,
            ));
        }
        Ok(out)
    }

    // -----------------------------
    // USAGE
    // -----------------------------
    pub async fn get_usage(&self, user_id: i32) -> Result<(f64, i64, i64)> {
        let row = sqlx::query(
            "SELECT cpu_seconds, ram_bytes, calc_count FROM usage WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok((
                row.try_get("cpu_seconds")?,
                row.try_get("ram_bytes")?,
                row.try_get("calc_count")?,
            ))
        } else {
            Ok((0.0, 0, 0))
        }
    }

    /// NEW: Add CPU/RAM usage + increment calc_count
    pub async fn add_usage(
        &self,
        user_id: i32,
        cpu_seconds: f64,
        ram_bytes: i64,
        calc_inc: i64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO usage (user_id, cpu_seconds, ram_bytes, calc_count)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id)
            DO UPDATE SET
                cpu_seconds = usage.cpu_seconds + EXCLUDED.cpu_seconds,
                ram_bytes   = usage.ram_bytes   + EXCLUDED.ram_bytes,
                calc_count  = usage.calc_count  + EXCLUDED.calc_count,
                updated_at  = NOW()
            "#,
        )
        .bind(user_id)
        .bind(cpu_seconds)
        .bind(ram_bytes)
        .bind(calc_inc)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // -----------------------------
    // ADMIN
    // -----------------------------
    pub async fn list_users(&self) -> Result<Vec<(i32, String, String)>> {
        let rows = sqlx::query("SELECT id, username, role FROM users ORDER BY id")
            .fetch_all(&self.pool)
            .await?;

        let mut out = Vec::new();
        for row in rows {
            out.push((
                row.try_get("id")?,
                row.try_get("username")?,
                row.try_get("role")?,
            ));
        }
        Ok(out)
    }

    pub async fn update_user_role(&self, user_id: i32, role: &str) -> Result<()> {
        sqlx::query("UPDATE users SET role = $1 WHERE id = $2")
            .bind(role)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_user(&self, user_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_all_usage(&self) -> Result<Vec<(String, f64, i64, i64)>> {
        let rows = sqlx::query(
            r#"
            SELECT u.username, us.cpu_seconds, us.ram_bytes, us.calc_count
            FROM usage us
            JOIN users u ON us.user_id = u.id
            ORDER BY us.cpu_seconds DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::new();
        for row in rows {
            out.push((
                row.try_get("username")?,
                row.try_get("cpu_seconds")?,
                row.try_get("ram_bytes")?,
                row.try_get("calc_count")?,
            ));
        }
        Ok(out)
    }
}
