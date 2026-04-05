/// Use case: создать новый пост (СКЕЛЕТ ДЛЯ САМОСТОЯТЕЛЬНОЙ РЕАЛИЗАЦИИ).
///
/// Реализуй по образцу `get_post_by_slug.rs`.
///
/// Шаги:
/// 1. Прочитай get_post_by_slug.rs и пойми структуру use case.
/// 2. Реализуй метод execute() — по шагам из комментариев внутри.
/// 3. Добавь handler в web/handlers/admin.rs по образцу handlers/posts.rs::show.
///
/// Подсказка: PostService::create уже содержит нужную логику.
/// Задача — перенести её в отдельный use case, получив те же преимущества:
/// явные зависимости, единственная ответственность, тестируемость без HTTP.
use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::domain::post::{Post, PostRepository};
use crate::domain::tag::TagRepository;

/// Команда (входные данные для use case).
///
/// DDD-паттерн Command: отдельная структура для входных данных операции.
/// Не путай с Post (entity) — CreatePostCommand это намерение, Post это результат.
///
/// В Python/FastAPI аналог — Pydantic-схема для request body (отдельно от response schema).
pub struct CreatePostCommand {
    pub title: String,
    pub content: String,
    pub excerpt: Option<String>,
    /// Имена тегов (строки), а не ID — use case сам создаст теги если нужно.
    pub tag_names: Vec<String>,
}

/// Use case: создание поста.
pub struct CreatePost {
    post_repo: Arc<dyn PostRepository>,
    tag_repo: Arc<dyn TagRepository>,
}

impl CreatePost {
    pub fn new(post_repo: Arc<dyn PostRepository>, tag_repo: Arc<dyn TagRepository>) -> Self {
        Self { post_repo, tag_repo }
    }

    /// Выполнить use case.
    ///
    /// TODO: реализуй следующие шаги:
    ///
    /// 1. Сгенерируй slug из title через `slug::slugify(&command.title)`.
    ///
    /// 2. Проверь уникальность slug:
    ///    `self.post_repo.find_by_slug(&slug).await?`
    ///    Если пост найден → вернуть `Err(DomainError::Conflict(...).into())`.
    ///
    /// 3. Resolve tags: для каждого имени из command.tag_names вызови
    ///    `self.tag_repo.find_or_create(&name).await?`
    ///    Собери Vec<Uuid> из id полученных тегов.
    ///
    /// 4. Создай пост через
    ///    `self.post_repo.create(CreatePostCmd { title, slug, content, excerpt, tag_ids }).await?`
    ///    (обрати внимание: CreatePostCmd — это domain команда из domain/post.rs,
    ///    не путать с CreatePostCommand выше)
    ///
    /// 5. Верни Ok(post).
    ///
    /// Подсказка по ошибкам: `?` автоматически конвертирует DomainError → ApplicationError
    /// благодаря `impl From<DomainError> for ApplicationError` в application/errors.rs.
    pub async fn execute(&self, _command: CreatePostCommand) -> Result<Post, ApplicationError> {
        todo!("Реализуй use case CreatePost по образцу get_post_by_slug.rs")
    }
}
