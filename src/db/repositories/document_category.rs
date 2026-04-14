use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::DocumentCategoryRow;
use crate::error::{AdeptusError, AdeptusResult};
use crate::models::DocumentCategory;

#[derive(Clone)]
pub struct DocumentCategoryRepository {
    pool: PgPool,
}

impl DocumentCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        name: &str,
        slug: &str,
        description: Option<&str>,
        parent_id: Option<Uuid>,
    ) -> AdeptusResult<DocumentCategory> {
        let row = sqlx::query_as::<_, DocumentCategoryRow>(
            r#"
            INSERT INTO document_categories (name, slug, description, parent_id)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(slug)
        .bind(description)
        .bind(parent_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }

    pub async fn get_by_id(&self, id: Uuid) -> AdeptusResult<Option<DocumentCategory>> {
        let row = sqlx::query_as::<_, DocumentCategoryRow>(
            "SELECT * FROM document_categories WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.map(Into::into))
    }

    pub async fn list(&self) -> AdeptusResult<Vec<DocumentCategory>> {
        let rows = sqlx::query_as::<_, DocumentCategoryRow>(
            "SELECT * FROM document_categories ORDER BY sort_order, name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
