use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use tera::Tera;

use crate::application::comment_service::CommentService;
use crate::application::get_post_by_slug::GetPostBySlug;
// Используем трейт-порт, а не конкретный PostService.
// DDD-принцип: web-слой зависит от абстракции (PostServicePort), а не от реализации.
//
// Это открывает возможность:
// - В тестах подать MockPostService
// - Добавить кэширующий декоратор PostServiceWithCache без изменения handler'ов
// - Заменить реализацию без пересборки web-слоя
//
// Arc<dyn PostServicePort> реализует Clone — Arc<T: ?Sized> всегда Clone.
use crate::application::post_service::PostServicePort;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub tera: Arc<Tera>,

    /// Сервис постов — через трейт-порт, не конкретный тип.
    /// Используется большинством admin handler'ов (CRUD).
    pub posts: Arc<dyn PostServicePort>,

    /// Use case для публичного endpoint GET /posts/:slug.
    ///
    /// DDD-принцип: use case — отдельная зависимость, не метод сервиса.
    /// Это явно показывает, что у handler'а одна зависимость (репозиторий),
    /// а не весь PostService с его TagRepository внутри.
    ///
    /// Аналог в Python/FastAPI: Depends(get_use_case) вместо Depends(PostService).
    pub get_post_by_slug: Arc<GetPostBySlug>,

    pub comments: Arc<CommentService>,
    pub config: Arc<Config>,
    pub cookie_key: Key,
}

/// Позволяет Axum извлекать Key напрямую из AppState.
/// Требуется для работы PrivateCookieJar в handler'ах и middleware.
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}
