use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::DocumentRow;
use crate::error::{AdeptusError, AdeptusResult};
use crate::models::Document;

#[derive(Clone)]
pub struct DocumentRepository {
    pool: PgPool,
}

impl DocumentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        title: &str,
        slug: &str,
        description: Option<&str>,
        content: &str,
        language: &str,
        category_id: Option<Uuid>,
        tags: &[String],
        status: &str,
        created_by: Uuid,
    ) -> AdeptusResult<Document> {
        let row = sqlx::query_as::<_, DocumentRow>(
            r#"
            INSERT INTO documents (title, slug, description, content, language, category_id, tags, status, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(title)
        .bind(slug)
        .bind(description)
        .bind(content)
        .bind(language)
        .bind(category_id)
        .bind(tags)
        .bind(status)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }

    pub async fn get_by_id(&self, id: Uuid) -> AdeptusResult<Option<Document>> {
        let row = sqlx::query_as::<_, DocumentRow>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AdeptusError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(row.map(Into::into))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: Uuid,
        title: Option<&str>,
        description: Option<&str>,
        content: Option<&str>,
        language: Option<&str>,
        category_id: Option<Uuid>,
        tags: Option<&[String]>,
        status: Option<&str>,
    ) -> AdeptusResult<Document> {
        let existing = sqlx::query_as::<_, DocumentRow>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AdeptusError::DatabaseError {
                message: e.to_string(),
            })?
            .ok_or_else(|| AdeptusError::DocumentNotFound { id: id.to_string() })?;

        let row = sqlx::query_as::<_, DocumentRow>(
            r#"
            UPDATE documents
            SET title = $2, description = $3, content = $4, language = $5,
                category_id = $6, tags = $7, status = $8, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(title.unwrap_or(&existing.title))
        .bind(description.or(existing.description.as_deref()))
        .bind(content.unwrap_or(&existing.content))
        .bind(language.unwrap_or(&existing.language))
        .bind(category_id.or(existing.category_id))
        .bind(tags.unwrap_or(&existing.tags.clone().unwrap_or_default()))
        .bind(status.unwrap_or(&existing.status))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }

    pub async fn delete(&self, id: Uuid) -> AdeptusResult<()> {
        sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AdeptusError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(())
    }

    pub async fn list(
        &self,
        category_id: Option<Uuid>,
        status: Option<&str>,
        language: Option<&str>,
        created_by: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> AdeptusResult<Vec<Document>> {
        let rows = sqlx::query_as::<_, DocumentRow>(
            r#"
            SELECT * FROM documents
            WHERE ($1::UUID IS NULL OR category_id = $1)
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::TEXT IS NULL OR language = $3)
              AND ($4::UUID IS NULL OR created_by = $4)
            ORDER BY updated_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(category_id)
        .bind(status)
        .bind(language)
        .bind(created_by)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn count(
        &self,
        category_id: Option<Uuid>,
        status: Option<&str>,
        language: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AdeptusResult<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM documents
            WHERE ($1::UUID IS NULL OR category_id = $1)
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::TEXT IS NULL OR language = $3)
              AND ($4::UUID IS NULL OR created_by = $4)
            "#,
        )
        .bind(category_id)
        .bind(status)
        .bind(language)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(count.0)
    }

    pub async fn set_status(
        &self,
        id: Uuid,
        status: &str,
        published_at: Option<DateTime<Utc>>,
    ) -> AdeptusResult<Document> {
        let row = sqlx::query_as::<_, DocumentRow>(
            r#"
            UPDATE documents
            SET status = $2, published_at = COALESCE($3, published_at), updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(published_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }
}
