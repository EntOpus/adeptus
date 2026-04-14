use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::DocumentFileRow;
use crate::error::{AdeptusError, AdeptusResult};
use crate::models::DocumentFile;

#[derive(Clone)]
pub struct DocumentFileRepository {
    pool: PgPool,
}

impl DocumentFileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        document_id: Uuid,
        filename: &str,
        original_filename: &str,
        file_path: &str,
        cdn_url: Option<&str>,
        mime_type: &str,
        file_size: i64,
        uploaded_by: Uuid,
    ) -> AdeptusResult<DocumentFile> {
        let row = sqlx::query_as::<_, DocumentFileRow>(
            r#"
            INSERT INTO document_files (document_id, filename, original_filename, file_path, cdn_url, mime_type, file_size, uploaded_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(document_id)
        .bind(filename)
        .bind(original_filename)
        .bind(file_path)
        .bind(cdn_url)
        .bind(mime_type)
        .bind(file_size)
        .bind(uploaded_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }

    pub async fn get_by_id(&self, id: Uuid) -> AdeptusResult<Option<DocumentFile>> {
        let row =
            sqlx::query_as::<_, DocumentFileRow>("SELECT * FROM document_files WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AdeptusError::DatabaseError {
                    message: e.to_string(),
                })?;

        Ok(row.map(Into::into))
    }

    pub async fn get_by_document(&self, document_id: Uuid) -> AdeptusResult<Vec<DocumentFile>> {
        let rows = sqlx::query_as::<_, DocumentFileRow>(
            "SELECT * FROM document_files WHERE document_id = $1 ORDER BY created_at",
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
        sqlx::query("DELETE FROM document_files WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AdeptusError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(())
    }
}
