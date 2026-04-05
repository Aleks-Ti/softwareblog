/// Use case: получить опубликованный пост по slug.
///
/// # Почему use case — отдельная struct, а не метод PostService?
///
/// 1. **Единственная ответственность**: этот use case делает ровно одно —
///    найти и вернуть опубликованный пост. Ничего лишнего.
///
/// 2. **Явные зависимости**: конструктор показывает, что нужно только PostRepository.
///    Метод PostService::get_by_slug имел бы доступ ко всему PostService (tags, и т.д.).
///
/// 3. **Тестируемость**: чтобы протестировать этот use case, нужен только MockPostRepository.
///    Не нужен TagRepository, не нужен CommentRepository.
///
/// 4. **В Python/FastAPI**: аналог — отдельный класс GetPostBySlugUseCase(repo: PostRepository),
///    который регистрируется через Depends(). Здесь то же самое, но через Arc в AppState.
///
/// # DDD-термины:
/// Это Application Service в DDD (не Domain Service).
/// Application Service координирует доменные объекты и инфраструктуру.
use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::domain::errors::DomainError;
use crate::domain::post::{Post, PostRepository};

pub struct GetPostBySlug {
    /// Зависимость через порт (trait), не через конкретный тип.
    /// GetPostBySlug не знает про PostgreSQL — знает только про PostRepository.
    post_repo: Arc<dyn PostRepository>,
}

impl GetPostBySlug {
    /// DI через конструктор — единственный способ создать use case.
    /// Явные зависимости лучше скрытых (service locator, глобальный стейт).
    pub fn new(post_repo: Arc<dyn PostRepository>) -> Self {
        Self { post_repo }
    }

    /// Выполнить use case.
    ///
    /// Возвращает `Post` (не `Option<Post>`) — если пост не найден или не опубликован,
    /// это одинаково `DomainError::NotFound`. Клиент не должен знать разницы.
    ///
    /// Обрати внимание на поток ошибок:
    ///   sqlx::Error  →(From в infra/postgres/mod.rs)→  DomainError::Internal
    ///   DomainError  →(From в application/errors.rs)→  ApplicationError
    ///   ApplicationError  →(From в web/errors.rs)→  AppError  →(IntoResponse)→  HTTP 404/500
    pub async fn execute(&self, slug: &str) -> Result<Post, ApplicationError> {
        // ? конвертирует DomainError → ApplicationError
        let post = self
            .post_repo
            .find_by_slug(slug)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Post '{slug}' not found")))?;

        // Бизнес-правило домена: публичный endpoint видит только опубликованные посты.
        // Черновики "не существуют" для внешнего мира.
        // Используем NotFound, а не Forbidden — не раскрываем факт существования черновика.
        //
        // Это правило раньше жило в handler'е (posts.rs::show) — нарушение DDD.
        // Теперь оно здесь, в use case, который можно протестировать без HTTP.
        if !post.is_published() {
            return Err(DomainError::NotFound(format!("Post '{slug}' not found")).into());
        }

        Ok(post)
    }
}
