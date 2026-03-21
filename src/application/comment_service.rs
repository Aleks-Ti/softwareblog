use std::sync::Arc;

use uuid::Uuid;

use crate::domain::comment::{Comment, CommentRepository, CreateComment};
use crate::domain::post::PostRepository;
use crate::error::AppError;

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
    ) -> Result<Vec<Comment>, AppError> {
        self.comments
            .find_by_post(post_id, !include_unapproved)
            .await
    }

    /// Submit a new comment. Comments start unapproved.
    pub async fn submit(&self, data: CreateComment) -> Result<Comment, AppError> {
        let post = self
            .posts
            .find_by_id(data.post_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".into()))?;

        if !post.is_published() {
            return Err(AppError::NotFound("Post not found".into()));
        }

        self.comments.create(data).await
    }

    pub async fn approve(&self, id: Uuid) -> Result<Comment, AppError> {
        self.comments.approve(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.comments.delete(id).await
    }

    pub async fn pending(&self) -> Result<Vec<Comment>, AppError> {
        self.comments.find_pending().await
    }
}
