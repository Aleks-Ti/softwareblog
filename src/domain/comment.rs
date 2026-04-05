use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_name: String,
    pub author_email: String,
    pub content: String,
    pub is_approved: bool,
    pub created_at: DateTime<Utc>,
}

pub struct CreateComment {
    pub post_id: Uuid,
    pub author_name: String,
    pub author_email: String,
    pub content: String,
}

/// Порт репозитория для комментариев.
/// Возвращает DomainError — инфраструктурные детали здесь скрыты.
#[async_trait]
pub trait CommentRepository: Send + Sync {
    async fn find_by_post(&self, post_id: Uuid, only_approved: bool) -> Result<Vec<Comment>, DomainError>;
    async fn find_pending(&self) -> Result<Vec<Comment>, DomainError>;
    async fn create(&self, data: CreateComment) -> Result<Comment, DomainError>;
    async fn approve(&self, id: Uuid) -> Result<Comment, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}
