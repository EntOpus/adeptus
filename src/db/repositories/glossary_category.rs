use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::GlossaryCategoryRow;
use crate::error::{AdeptusError, AdeptusResult};
use crate::models::GlossaryCategory;

#[derive(Clone)]
pub struct GlossaryCategoryRepository {
    pool: PgPool,
}

impl GlossaryCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        name: &str,
        slug: &str,
        description: Option<&str>,
    ) -> AdeptusResult<GlossaryCategory> {
        let row = sqlx::query_as::<_, GlossaryCategoryRow>(
            r#"
            INSERT INTO glossary_categories (name, slug, description)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(slug)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }

    pub async fn get_by_id(&self, id: Uuid) -> AdeptusResult<Option<GlossaryCategory>> {
        let row = sqlx::query_as::<_, GlossaryCategoryRow>(
            "SELECT * FROM glossary_categories WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.map(Into::into))
    }

    pub async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        sort_order: Option<i32>,
    ) -> AdeptusResult<GlossaryCategory> {
        let existing = sqlx::query_as::<_, GlossaryCategoryRow>(
            "SELECT * FROM glossary_categories WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?
        .ok_or_else(|| AdeptusError::GlossaryCategoryNotFound { id: id.to_string() })?;

        let row = sqlx::query_as::<_, GlossaryCategoryRow>(
            r#"
            UPDATE glossary_categories
            SET name = $2, description = $3, sort_order = $4, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name.unwrap_or(&existing.name))
        .bind(description.or(existing.description.as_deref()))
        .bind(sort_order.unwrap_or(existing.sort_order))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }

    pub async fn delete(&self, id: Uuid) -> AdeptusResult<()> {
        sqlx::query("DELETE FROM glossary_categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AdeptusError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(())
    }

    pub async fn list(&self) -> AdeptusResult<Vec<GlossaryCategory>> {
        let rows = sqlx::query_as::<_, GlossaryCategoryRow>(
            "SELECT * FROM glossary_categories ORDER BY sort_order, name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
