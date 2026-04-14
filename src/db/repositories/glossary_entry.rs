use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::GlossaryEntryRow;
use crate::error::{AdeptusError, AdeptusResult};
use crate::models::GlossaryEntry;

#[derive(Clone)]
pub struct GlossaryEntryRepository {
    pool: PgPool,
}

impl GlossaryEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        term: &str,
        slug: &str,
        definition: &str,
        extended_description: Option<&str>,
        language: &str,
        category_id: Option<Uuid>,
        status: &str,
        created_by: Uuid,
    ) -> AdeptusResult<GlossaryEntry> {
        let row = sqlx::query_as::<_, GlossaryEntryRow>(
            r#"
            INSERT INTO glossary_entries (term, slug, definition, extended_description, language, category_id, status, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(term)
        .bind(slug)
        .bind(definition)
        .bind(extended_description)
        .bind(language)
        .bind(category_id)
        .bind(status)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }

    pub async fn get_by_id(&self, id: Uuid) -> AdeptusResult<Option<GlossaryEntry>> {
        let row =
            sqlx::query_as::<_, GlossaryEntryRow>("SELECT * FROM glossary_entries WHERE id = $1")
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
        term: Option<&str>,
        definition: Option<&str>,
        extended_description: Option<&str>,
        language: Option<&str>,
        category_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AdeptusResult<GlossaryEntry> {
        let existing =
            sqlx::query_as::<_, GlossaryEntryRow>("SELECT * FROM glossary_entries WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AdeptusError::DatabaseError {
                    message: e.to_string(),
                })?
                .ok_or_else(|| AdeptusError::GlossaryEntryNotFound { id: id.to_string() })?;

        let row = sqlx::query_as::<_, GlossaryEntryRow>(
            r#"
            UPDATE glossary_entries
            SET term = $2, definition = $3, extended_description = $4, language = $5,
                category_id = $6, status = $7, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(term.unwrap_or(&existing.term))
        .bind(definition.unwrap_or(&existing.definition))
        .bind(extended_description.or(existing.extended_description.as_deref()))
        .bind(language.unwrap_or(&existing.language))
        .bind(category_id.or(existing.category_id))
        .bind(status.unwrap_or(&existing.status))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(row.into())
    }

    pub async fn delete(&self, id: Uuid) -> AdeptusResult<()> {
        sqlx::query("DELETE FROM glossary_entries WHERE id = $1")
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
        limit: i64,
        offset: i64,
    ) -> AdeptusResult<Vec<GlossaryEntry>> {
        let rows = sqlx::query_as::<_, GlossaryEntryRow>(
            r#"
            SELECT * FROM glossary_entries
            WHERE ($1::UUID IS NULL OR category_id = $1)
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::TEXT IS NULL OR language = $3)
            ORDER BY term ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(category_id)
        .bind(status)
        .bind(language)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn search(
        &self,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> AdeptusResult<Vec<GlossaryEntry>> {
        let rows = sqlx::query_as::<_, GlossaryEntryRow>(
            r#"
            SELECT * FROM glossary_entries
            WHERE term ILIKE $1 OR definition ILIKE $1
            ORDER BY term ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn get_by_category(&self, category_id: Uuid) -> AdeptusResult<Vec<GlossaryEntry>> {
        let rows = sqlx::query_as::<_, GlossaryEntryRow>(
            "SELECT * FROM glossary_entries WHERE category_id = $1 ORDER BY term ASC",
        )
        .bind(category_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn get_by_language(&self, language: &str) -> AdeptusResult<Vec<GlossaryEntry>> {
        let rows = sqlx::query_as::<_, GlossaryEntryRow>(
            "SELECT * FROM glossary_entries WHERE language = $1 ORDER BY term ASC",
        )
        .bind(language)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdeptusError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn get_related(&self, entry_id: Uuid) -> AdeptusResult<Vec<GlossaryEntry>> {
        let rows = sqlx::query_as::<_, GlossaryEntryRow>(
            r#"
            SELECT ge.* FROM glossary_entries ge
            INNER JOIN glossary_relationships gr
                ON (gr.target_entry_id = ge.id AND gr.source_entry_id = $1)
                OR (gr.source_entry_id = ge.id AND gr.target_entry_id = $1)
            ORDER BY ge.term ASC
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
}
