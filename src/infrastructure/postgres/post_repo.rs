use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::domain::post::{
    CreatePost, Post, PostFilters, PostRepository, UpdatePost,
};

/// Адаптер: реализация PostRepository поверх PostgreSQL.
///
/// DDD-принцип: инфраструктурный слой реализует порт (trait), определённый в домене.
/// Домен не знает про PgPool — знает только про PostRepository.
///
/// sqlx::Error конвертируется в DomainError автоматически через From<sqlx::Error>,
/// определённый в infrastructure/postgres/mod.rs. Поэтому `?` работает
/// несмотря на то что trait возвращает DomainError, а не sqlx::Error.
pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Post>, DomainError> {
        let post = sqlx::query_as::<_, Post>(
            "SELECT id, title, slug, content, excerpt, status, created_at, updated_at, published_at
             FROM posts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(post)
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Post>, DomainError> {
        let post = sqlx::query_as::<_, Post>(
            "SELECT id, title, slug, content, excerpt, status, created_at, updated_at, published_at
             FROM posts WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(post)
    }

    async fn list(&self, filters: PostFilters) -> Result<(Vec<Post>, u64), DomainError> {
        let per_page = filters.per_page.max(1) as i64;
        let offset = ((filters.page.saturating_sub(1)) as i64) * per_page;

        let (status_filter, status_val) = match &filters.status {
            Some(s) => ("AND p.status = $3", Some(s.clone())),
            None => ("", None),
        };

        let posts = if let Some(status) = status_val {
            sqlx::query_as::<_, Post>(&format!(
                "SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.status,
                        p.created_at, p.updated_at, p.published_at
                 FROM posts p
                 WHERE 1=1 {status_filter}
                 ORDER BY COALESCE(p.published_at, p.created_at) DESC
                 LIMIT $1 OFFSET $2"
            ))
            .bind(per_page)
            .bind(offset)
            .bind(status)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Post>(
                "SELECT id, title, slug, content, excerpt, status, created_at, updated_at, published_at
                 FROM posts
                 ORDER BY COALESCE(published_at, created_at) DESC
                 LIMIT $1 OFFSET $2",
            )
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await?;

        Ok((posts, total as u64))
    }

    async fn create(&self, data: CreatePost) -> Result<Post, DomainError> {
        let mut tx = self.pool.begin().await?;

        let post = sqlx::query_as::<_, Post>(
            "INSERT INTO posts (title, slug, content, excerpt)
             VALUES ($1, $2, $3, $4)
             RETURNING id, title, slug, content, excerpt, status, created_at, updated_at, published_at",
        )
        .bind(&data.title)
        .bind(&data.slug)
        .bind(&data.content)
        .bind(&data.excerpt)
        .fetch_one(&mut *tx)
        .await?;

        for tag_id in &data.tag_ids {
            sqlx::query("INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2)")
                .bind(post.id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(post)
    }

    async fn update(&self, id: Uuid, data: UpdatePost) -> Result<Post, DomainError> {
        let post = sqlx::query_as::<_, Post>(
            "UPDATE posts SET
               title   = COALESCE($2, title),
               content = COALESCE($3, content),
               excerpt = COALESCE($4, excerpt)
             WHERE id = $1
             RETURNING id, title, slug, content, excerpt, status, created_at, updated_at, published_at",
        )
        .bind(id)
        .bind(data.title)
        .bind(data.content)
        .bind(data.excerpt)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Post {id} not found")))?;

        Ok(post)
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!("Post {id} not found")));
        }
        Ok(())
    }

    async fn publish(&self, id: Uuid) -> Result<Post, DomainError> {
        let post = sqlx::query_as::<_, Post>(
            "UPDATE posts SET status = 'published', published_at = COALESCE(published_at, $2)
             WHERE id = $1
             RETURNING id, title, slug, content, excerpt, status, created_at, updated_at, published_at",
        )
        .bind(id)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Post {id} not found")))?;

        Ok(post)
    }

    async fn unpublish(&self, id: Uuid) -> Result<Post, DomainError> {
        let post = sqlx::query_as::<_, Post>(
            "UPDATE posts SET status = 'draft'
             WHERE id = $1
             RETURNING id, title, slug, content, excerpt, status, created_at, updated_at, published_at",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Post {id} not found")))?;

        Ok(post)
    }
}
