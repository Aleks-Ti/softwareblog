use std::sync::Arc;

use async_trait::async_trait;
use slug::slugify;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::domain::errors::DomainError;
use crate::domain::post::{CreatePost, Post, PostFilters, PostRepository, PostStatus, UpdatePost};
use crate::domain::tag::{Tag, TagRepository};

// ─── Port (интерфейс для web-слоя) ──────────────────────────────────────────
//
// DDD-принцип: AppState хранит Arc<dyn PostServicePort>, а не Arc<PostService>.
//
// Зачем? Web-слой не должен зависеть от конкретной реализации сервиса.
// Это открывает путь к подмене реализации (тесты с моком, несколько имплементаций).
//
// Аналог в Python/FastAPI: Protocol или ABC в зависимости от DI-контейнера,
// а не конкретный класс PostService в type hints handler'а.
//
// Почему trait здесь, а не в domain/?
// PostServicePort оперирует ApplicationError — это application-level концепция.
// Domain не знает про Application layer, поэтому порт живёт здесь.

#[async_trait]
pub trait PostServicePort: Send + Sync {
    async fn get_by_slug(&self, slug: &str) -> Result<Option<Post>, ApplicationError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Post>, ApplicationError>;
    async fn list_published(
        &self,
        page: u32,
        per_page: u32,
        tag: Option<String>,
    ) -> Result<(Vec<Post>, u64), ApplicationError>;
    async fn list_all(&self, page: u32, per_page: u32) -> Result<(Vec<Post>, u64), ApplicationError>;
    async fn create(
        &self,
        title: String,
        content: String,
        excerpt: Option<String>,
        tag_names: Vec<String>,
    ) -> Result<Post, ApplicationError>;
    async fn update(&self, id: Uuid, data: UpdatePost) -> Result<Post, ApplicationError>;
    async fn publish(&self, id: Uuid) -> Result<Post, ApplicationError>;
    async fn unpublish(&self, id: Uuid) -> Result<Post, ApplicationError>;
    async fn delete(&self, id: Uuid) -> Result<(), ApplicationError>;
    async fn all_tags(&self) -> Result<Vec<Tag>, ApplicationError>;
}

// ─── Concrete Service ────────────────────────────────────────────────────────

/// Сервис-оркестратор для постов.
///
/// Принимает Arc<dyn PostRepository> — зависит от абстракции, не от PostgreSQL.
/// В тестах можно подать MockPostRepository.
pub struct PostService {
    posts: Arc<dyn PostRepository>,
    tags: Arc<dyn TagRepository>,
}

impl PostService {
    pub fn new(posts: Arc<dyn PostRepository>, tags: Arc<dyn TagRepository>) -> Self {
        Self { posts, tags }
    }
}

/// Реализуем порт для конкретного сервиса.
/// Web-слой знает только про PostServicePort — PostService ему невидим.
#[async_trait]
impl PostServicePort for PostService {
    async fn get_by_slug(&self, slug: &str) -> Result<Option<Post>, ApplicationError> {
        // ? конвертирует DomainError → ApplicationError через From (определён в errors.rs)
        Ok(self.posts.find_by_slug(slug).await?)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Post>, ApplicationError> {
        Ok(self.posts.find_by_id(id).await?)
    }

    async fn list_published(
        &self,
        page: u32,
        per_page: u32,
        tag: Option<String>,
    ) -> Result<(Vec<Post>, u64), ApplicationError> {
        Ok(self
            .posts
            .list(PostFilters {
                status: Some(PostStatus::Published),
                tag_slug: tag,
                page,
                per_page,
            })
            .await?)
    }

    async fn list_all(&self, page: u32, per_page: u32) -> Result<(Vec<Post>, u64), ApplicationError> {
        Ok(self
            .posts
            .list(PostFilters { page, per_page, ..Default::default() })
            .await?)
    }

    async fn create(
        &self,
        title: String,
        content: String,
        excerpt: Option<String>,
        tag_names: Vec<String>,
    ) -> Result<Post, ApplicationError> {
        let slug = slugify(&title);

        if self.posts.find_by_slug(&slug).await?.is_some() {
            return Err(DomainError::Conflict(format!(
                "Post with slug '{slug}' already exists"
            ))
            .into());
        }

        let mut tag_ids = Vec::new();
        for name in tag_names {
            let tag = self.tags.find_or_create(&name).await?;
            tag_ids.push(tag.id);
        }

        Ok(self
            .posts
            .create(CreatePost { title, slug, content, excerpt, tag_ids })
            .await?)
    }

    async fn update(&self, id: Uuid, data: UpdatePost) -> Result<Post, ApplicationError> {
        Ok(self.posts.update(id, data).await?)
    }

    async fn publish(&self, id: Uuid) -> Result<Post, ApplicationError> {
        Ok(self.posts.publish(id).await?)
    }

    async fn unpublish(&self, id: Uuid) -> Result<Post, ApplicationError> {
        Ok(self.posts.unpublish(id).await?)
    }

    async fn delete(&self, id: Uuid) -> Result<(), ApplicationError> {
        Ok(self.posts.delete(id).await?)
    }

    async fn all_tags(&self) -> Result<Vec<Tag>, ApplicationError> {
        Ok(self.tags.list().await?)
    }
}
