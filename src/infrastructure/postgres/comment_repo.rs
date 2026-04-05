use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::comment::{Comment, CommentRepository, CreateComment};
use crate::domain::errors::DomainError;

pub struct PostgresCommentRepository {
    pool: PgPool,
}

impl PostgresCommentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommentRepository for PostgresCommentRepository {
    async fn find_by_post(
        &self,
        post_id: Uuid,
        only_approved: bool,
    ) -> Result<Vec<Comment>, DomainError> {
        let sql = if only_approved {
            "SELECT id, post_id, author_name, author_email, content, is_approved, created_at
             FROM comments WHERE post_id = $1 AND is_approved = TRUE
             ORDER BY created_at ASC"
        } else {
            "SELECT id, post_id, author_name, author_email, content, is_approved, created_at
             FROM comments WHERE post_id = $1
             ORDER BY created_at ASC"
        };

        let comments = sqlx::query_as::<_, Comment>(sql)
            .bind(post_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(comments)
    }

    async fn find_pending(&self) -> Result<Vec<Comment>, DomainError> {
        let comments = sqlx::query_as::<_, Comment>(
            "SELECT id, post_id, author_name, author_email, content, is_approved, created_at
             FROM comments WHERE is_approved = FALSE
             ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    async fn create(&self, data: CreateComment) -> Result<Comment, DomainError> {
        let comment = sqlx::query_as::<_, Comment>(
            "INSERT INTO comments (post_id, author_name, author_email, content)
             VALUES ($1, $2, $3, $4)
             RETURNING id, post_id, author_name, author_email, content, is_approved, created_at",
        )
        .bind(data.post_id)
        .bind(&data.author_name)
        .bind(&data.author_email)
        .bind(&data.content)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn approve(&self, id: Uuid) -> Result<Comment, DomainError> {
        let comment = sqlx::query_as::<_, Comment>(
            "UPDATE comments SET is_approved = TRUE WHERE id = $1
             RETURNING id, post_id, author_name, author_email, content, is_approved, created_at",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Comment {id} not found")))?;

        Ok(comment)
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query("DELETE FROM comments WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!("Comment {id} not found")));
        }
        Ok(())
    }
}
