use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Tag>, AppError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tag>, AppError>;
    async fn find_by_post(&self, post_id: Uuid) -> Result<Vec<Tag>, AppError>;
    async fn find_or_create(&self, name: &str) -> Result<Tag, AppError>;
}
