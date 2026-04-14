use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::DocumentRelationshipRow;
use crate::error::{AdeptusError, AdeptusResult};
use crate::models::DocumentRelationship;

#[derive(Clone)]
pub struct DocumentRelationshipRepository {
    pool: PgPool,
}

impl DocumentRelationshipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        source_document_id: Uuid,
        target_document_id: Uuid,
        relationship_type: &str,
    ) -> AdeptusResult<DocumentRelationship> {
        let row = sqlx::query_as::<_, DocumentRelationshipRow>(
            r#"
            INSERT INTO document_relationships (source_document_id, target_document_id, relationship_type)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(source_document_id)
        .bind(target_document_id)
        .bind(relationship_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }

    pub async fn get_by_document(
        &self,
        document_id: Uuid,
    ) -> AdeptusResult<Vec<DocumentRelationship>> {
        let rows = sqlx::query_as::<_, DocumentRelationshipRow>(
            r#"
            SELECT * FROM document_relationships
            WHERE source_document_id = $1 OR target_document_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn delete(&self, id: Uuid) -> AdeptusResult<()> {
        sqlx::query("DELETE FROM document_relationships WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AdeptusError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(())
    }
}
