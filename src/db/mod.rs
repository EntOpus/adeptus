pub mod models;
pub mod repositories;

pub use repositories::RepositoryManager;

use crate::error::{AdeptusError, AdeptusResult};
use sqlx::PgPool;
use std::time::Duration;
use tracing::info;

#[derive(Clone)]
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    pub async fn new(
        database_url: &str,
        max_connections: u32,
        min_connections: u32,
    ) -> AdeptusResult<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .max_lifetime(Duration::from_secs(30 * 60))
            .idle_timeout(Duration::from_secs(10 * 60))
            .acquire_timeout(Duration::from_secs(30))
            .test_before_acquire(true)
            .connect(database_url)
            .await
            .map_err(|e| AdeptusError::DatabaseError {
                message: format!("Failed to create connection pool: {}", e),
            })?;

        info!(
            max_connections,
            min_connections, "Database connection pool created"
        );
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn health_check(&self) -> AdeptusResult<()> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AdeptusError::DatabaseError {
                message: format!("Database health check failed: {}", e),
            })?;

        Ok(())
    }

    pub async fn run_migrations(&self) -> AdeptusResult<()> {
        info!("Running database migrations...");

        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| AdeptusError::DatabaseError {
                message: format!("Failed to run migrations: {}", e),
            })?;

        info!("Database migrations completed");
        Ok(())
    }

    pub async fn close(&self) {
        info!("Closing database connection pool...");
        self.pool.close().await;
        info!("Database connection pool closed");
    }
}
