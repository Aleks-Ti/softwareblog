use async_trait::async_trait;
use slug::slugify;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::tag::{Tag, TagRepository};
use crate::error::AppError;

pub struct PostgresTagRepository {
    pool: PgPool,
}

impl PostgresTagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagRepository for PostgresTagRepository {
    async fn list(&self) -> Result<Vec<Tag>, AppError> {
        let tags = sqlx::query_as::<_, Tag>("SELECT id, name, slug FROM tags ORDER BY name")
            .fetch_all(&self.pool)
            .await?;
        Ok(tags)
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tag>, AppError> {
        let tag =
            sqlx::query_as::<_, Tag>("SELECT id, name, slug FROM tags WHERE slug = $1")
                .bind(slug)
                .fetch_optional(&self.pool)
                .await?;
        Ok(tag)
    }

    async fn find_by_post(&self, post_id: Uuid) -> Result<Vec<Tag>, AppError> {
        let tags = sqlx::query_as::<_, Tag>(
            "SELECT t.id, t.name, t.slug FROM tags t
             INNER JOIN post_tags pt ON pt.tag_id = t.id
             WHERE pt.post_id = $1
             ORDER BY t.name",
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(tags)
    }

    async fn find_or_create(&self, name: &str) -> Result<Tag, AppError> {
        let slug = slugify(name);

        // INSERT ... ON CONFLICT DO NOTHING, then SELECT — avoids race conditions.
        sqlx::query("INSERT INTO tags (name, slug) VALUES ($1, $2) ON CONFLICT (slug) DO NOTHING")
            .bind(name)
            .bind(&slug)
            .execute(&self.pool)
            .await?;

        let tag = sqlx::query_as::<_, Tag>("SELECT id, name, slug FROM tags WHERE slug = $1")
            .bind(&slug)
            .fetch_one(&self.pool)
            .await?;

        Ok(tag)
    }
}
