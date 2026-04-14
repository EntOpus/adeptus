use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::AuditLogRow;
use crate::error::{AdeptusError, AdeptusResult};
use crate::models::AuditLog;

#[derive(Clone)]
pub struct AuditRepository {
    pool: PgPool,
}

impl AuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_audit_log(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        action: &str,
        actor_id: Uuid,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        changes: Option<serde_json::Value>,
    ) -> AdeptusResult<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (entity_type, entity_id, action, actor_id, ip_address, user_agent, changes)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .bind(action)
        .bind(actor_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(changes)
        .execute(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(())
    }

    pub async fn get_audit_logs(
        &self,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
        actor_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> AdeptusResult<Vec<AuditLog>> {
        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT * FROM audit_logs
            WHERE ($1::TEXT IS NULL OR entity_type = $1)
              AND ($2::UUID IS NULL OR entity_id = $2)
              AND ($3::UUID IS NULL OR actor_id = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .bind(actor_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
