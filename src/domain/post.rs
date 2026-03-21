use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "post_status", rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Published,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String, // stored as Markdown
    pub excerpt: Option<String>,
    pub status: PostStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

impl Post {
    /// Render Markdown content to HTML.
    pub fn render_content(&self) -> String {
        md_to_html(&self.content)
    }

    pub fn is_published(&self) -> bool {
        self.status == PostStatus::Published
    }
}

pub fn md_to_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

// --- Commands (input DTOs for the application layer) ---

pub struct CreatePost {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub tag_ids: Vec<Uuid>,
}

pub struct UpdatePost {
    pub title: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub tag_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Default)]
pub struct PostFilters {
    pub status: Option<PostStatus>,
    pub tag_slug: Option<String>,
    pub page: u32,
    pub per_page: u32,
}

// --- Repository trait (port) ---

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Post>, AppError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Post>, AppError>;
    /// Returns (posts, total_count)
    async fn list(&self, filters: PostFilters) -> Result<(Vec<Post>, u64), AppError>;
    async fn create(&self, data: CreatePost) -> Result<Post, AppError>;
    async fn update(&self, id: Uuid, data: UpdatePost) -> Result<Post, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
    async fn publish(&self, id: Uuid) -> Result<Post, AppError>;
    async fn unpublish(&self, id: Uuid) -> Result<Post, AppError>;
}
