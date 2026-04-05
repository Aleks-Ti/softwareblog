use std::sync::Arc;

use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::domain::comment::{Comment, CommentRepository, CreateComment};
use crate::domain::errors::DomainError;
use crate::domain::post::PostRepository;

pub struct CommentService {
    comments: Arc<dyn CommentRepository>,
    posts: Arc<dyn PostRepository>,
}

impl CommentService {
    pub fn new(comments: Arc<dyn CommentRepository>, posts: Arc<dyn PostRepository>) -> Self {
        Self { comments, posts }
    }

    pub async fn for_post(
        &self,
        post_id: Uuid,
        include_unapproved: bool,
    ) -> Result<Vec<Comment>, ApplicationError> {
        Ok(self.comments.find_by_post(post_id, !include_unapproved).await?)
    }

    /// Отправить комментарий. Комментарии начинают как неодобренные.
    ///
    /// Бизнес-правило: нельзя комментировать неопубликованный пост.
    /// Это правило живёт в сервисе (application layer), а не в handler'е,
    /// потому что оно относится к логике, а не к HTTP.
    pub async fn submit(&self, data: CreateComment) -> Result<Comment, ApplicationError> {
        let post = self
            .posts
            .find_by_id(data.post_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("Post not found".into()))?;

        if !post.is_published() {
            // Возвращаем NotFound, а не Forbidden — не раскрываем факт существования черновика.
            return Err(DomainError::NotFound("Post not found".into()).into());
        }

        Ok(self.comments.create(data).await?)
    }

    pub async fn approve(&self, id: Uuid) -> Result<Comment, ApplicationError> {
        Ok(self.comments.approve(id).await?)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ApplicationError> {
        Ok(self.comments.delete(id).await?)
    }

    pub async fn pending(&self) -> Result<Vec<Comment>, ApplicationError> {
        Ok(self.comments.find_pending().await?)
    }
}
