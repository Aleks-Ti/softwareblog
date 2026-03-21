use std::sync::Arc;

use slug::slugify;
use uuid::Uuid;

use crate::domain::post::{CreatePost, Post, PostFilters, PostRepository, PostStatus, UpdatePost};
use crate::domain::tag::TagRepository;
use crate::error::AppError;

pub struct PostService {
    posts: Arc<dyn PostRepository>,
    tags: Arc<dyn TagRepository>,
}

impl PostService {
    pub fn new(posts: Arc<dyn PostRepository>, tags: Arc<dyn TagRepository>) -> Self {
        Self { posts, tags }
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Post>, AppError> {
        self.posts.find_by_slug(slug).await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Post>, AppError> {
        self.posts.find_by_id(id).await
    }

    pub async fn list_published(
        &self,
        page: u32,
        per_page: u32,
        tag: Option<String>,
    ) -> Result<(Vec<Post>, u64), AppError> {
        self.posts
            .list(PostFilters {
                status: Some(PostStatus::Published),
                tag_slug: tag,
                page,
                per_page,
            })
            .await
    }

    /// Admin: list all posts regardless of status.
    pub async fn list_all(&self, page: u32, per_page: u32) -> Result<(Vec<Post>, u64), AppError> {
        self.posts
            .list(PostFilters { page, per_page, ..Default::default() })
            .await
    }

    pub async fn create(
        &self,
        title: String,
        content: String,
        excerpt: Option<String>,
        tag_names: Vec<String>,
    ) -> Result<Post, AppError> {
        let slug = slugify(&title);

        if self.posts.find_by_slug(&slug).await?.is_some() {
            return Err(AppError::Conflict(format!(
                "Post with slug '{slug}' already exists"
            )));
        }

        // Resolve/create tags
        let mut tag_ids = Vec::new();
        for name in tag_names {
            let tag = self.tags.find_or_create(&name).await?;
            tag_ids.push(tag.id);
        }

        self.posts
            .create(CreatePost { title, slug, content, excerpt, tag_ids })
            .await
    }

    pub async fn update(&self, id: Uuid, data: UpdatePost) -> Result<Post, AppError> {
        self.posts.update(id, data).await
    }

    pub async fn publish(&self, id: Uuid) -> Result<Post, AppError> {
        self.posts.publish(id).await
    }

    pub async fn unpublish(&self, id: Uuid) -> Result<Post, AppError> {
        self.posts.unpublish(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.posts.delete(id).await
    }

    pub async fn all_tags(&self) -> Result<Vec<crate::domain::tag::Tag>, AppError> {
        self.tags.list().await
    }
}
