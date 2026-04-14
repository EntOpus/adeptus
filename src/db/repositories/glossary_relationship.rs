use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::GlossaryRelationshipRow;
use crate::error::{AdeptusError, AdeptusResult};
use crate::models::GlossaryRelationship;

#[derive(Clone)]
pub struct GlossaryRelationshipRepository {
    pool: PgPool,
}

impl GlossaryRelationshipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        source_entry_id: Uuid,
        target_entry_id: Uuid,
        relationship_type: &str,
    ) -> AdeptusResult<GlossaryRelationship> {
        let row = sqlx::query_as::<_, GlossaryRelationshipRow>(
            r#"
            INSERT INTO glossary_relationships (source_entry_id, target_entry_id, relationship_type)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(source_entry_id)
        .bind(target_entry_id)
        .bind(relationship_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }

    pub async fn get_by_entry(&self, entry_id: Uuid) -> AdeptusResult<Vec<GlossaryRelationship>> {
        let rows = sqlx::query_as::<_, GlossaryRelationshipRow>(
            r#"
            SELECT * FROM glossary_relationships
            WHERE source_entry_id = $1 OR target_entry_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(entry_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn delete(&self, id: Uuid) -> AdeptusResult<()> {
        sqlx::query("DELETE FROM glossary_relationships WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AdeptusError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(())
    }
}
