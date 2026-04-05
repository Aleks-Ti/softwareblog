use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

/// Порт репозитория для тегов.
#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Tag>, DomainError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tag>, DomainError>;
    async fn find_by_post(&self, post_id: Uuid) -> Result<Vec<Tag>, DomainError>;
    async fn find_or_create(&self, name: &str) -> Result<Tag, DomainError>;
}
