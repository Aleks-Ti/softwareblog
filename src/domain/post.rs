use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// DDD-принцип: домен использует свои ошибки, а не ошибки верхних слоёв.
// Раньше здесь был AppError — это было нарушением границ слоёв:
// домен "знал" о существовании HTTP-уровня.
use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "post_status", rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Published,
}

/// Post — агрегат доменного слоя.
///
/// Содержит не только данные, но и бизнес-методы (is_published, render_content).
/// В Python/FastAPI аналог — Pydantic-модель + методы на ней, но здесь граница чётче:
/// entity не знает про HTTP, ORM, шаблоны.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String, // хранится как Markdown
    pub excerpt: Option<String>,
    pub status: PostStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

impl Post {
    /// Бизнес-метод: рендер Markdown → HTML.
    /// Логика рендеринга живёт в домене — это бизнес-правило "контент публикуется как HTML".
    pub fn render_content(&self) -> String {
        md_to_html(&self.content)
    }

    /// Бизнес-метод: инвариант "пост опубликован".
    ///
    /// Обрати внимание — это метод на entity, не булевое поле.
    /// В DDD бизнес-правила выражаются через методы, а не через условия в сервисах.
    /// Use case просто вызывает `post.is_published()` вместо `post.status == PostStatus::Published`.
    pub fn is_published(&self) -> bool {
        self.status == PostStatus::Published
    }

    // ЗАГЛУШКА ДЛЯ САМОСТОЯТЕЛЬНОЙ РАБОТЫ:
    // Добавь метод validate_for_publish(&self) -> Result<(), DomainError>
    // который проверяет инварианты перед публикацией (непустой заголовок, контент и т.д.)
    // Это пример доменной валидации — бизнес-правило живёт в entity, не в сервисе.
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

// --- Commands (входные DTO для application layer) ---
//
// DDD-принцип: команды отделены от entity.
// CreatePost — это намерение ("создать пост"), Post — результат.
// В Python/FastAPI аналог — отдельные Pydantic-схемы для request body vs response.

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

// --- Repository trait (порт в терминах Hexagonal Architecture) ---
//
// DDD-принцип: домен определяет интерфейс (trait), инфраструктура его реализует.
// PostRepository — это ПОРТ. PostgresPostRepository — АДАПТЕР.
//
// Ключевое: trait возвращает DomainError, а не sqlx::Error.
// Инфраструктурный адаптер сам конвертирует sqlx::Error → DomainError::Internal.
//
// Аналог в Python: ABC (абстрактный базовый класс) в domain/, реализация в infra/.

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Post>, DomainError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Post>, DomainError>;
    /// Возвращает (посты, total_count).
    async fn list(&self, filters: PostFilters) -> Result<(Vec<Post>, u64), DomainError>;
    async fn create(&self, data: CreatePost) -> Result<Post, DomainError>;
    async fn update(&self, id: Uuid, data: UpdatePost) -> Result<Post, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    async fn publish(&self, id: Uuid) -> Result<Post, DomainError>;
    async fn unpublish(&self, id: Uuid) -> Result<Post, DomainError>;
}
