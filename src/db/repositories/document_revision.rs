use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::DocumentRevisionRow;
use crate::error::{AdeptusError, AdeptusResult};
use crate::models::DocumentRevision;

#[derive(Clone)]
pub struct DocumentRevisionRepository {
    pool: PgPool,
}

impl DocumentRevisionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        document_id: Uuid,
        revision_number: i32,
        title: &str,
        content: &str,
        change_summary: Option<&str>,
        created_by: Uuid,
    ) -> AdeptusResult<DocumentRevision> {
        let row = sqlx::query_as::<_, DocumentRevisionRow>(
            r#"
            INSERT INTO document_revisions (document_id, revision_number, title, content, change_summary, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(document_id)
        .bind(revision_number)
        .bind(title)
        .bind(content)
        .bind(change_summary)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }

    pub async fn get_by_document(&self, document_id: Uuid) -> AdeptusResult<Vec<DocumentRevision>> {
        let rows = sqlx::query_as::<_, DocumentRevisionRow>(
            "SELECT * FROM document_revisions WHERE document_id = $1 ORDER BY revision_number DESC",
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn get_by_document_and_number(
        &self,
        document_id: Uuid,
        revision_number: i32,
    ) -> AdeptusResult<Option<DocumentRevision>> {
        let row = sqlx::query_as::<_, DocumentRevisionRow>(
            "SELECT * FROM document_revisions WHERE document_id = $1 AND revision_number = $2",
        )
        .bind(document_id)
        .bind(revision_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.map(Into::into))
    }

    pub async fn get_latest_revision_number(&self, document_id: Uuid) -> AdeptusResult<i32> {
        let result: Option<(Option<i32>,)> = sqlx::query_as(
            "SELECT MAX(revision_number) FROM document_revisions WHERE document_id = $1",
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(result.and_then(|r| r.0).unwrap_or(0))
    }
}
