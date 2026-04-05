use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use serde::Deserialize;

use super::{is_htmx, render};
use crate::error::AppError;
use crate::web::state::AppState;

#[derive(Deserialize, Default)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub tag: Option<String>,
}

/// GET /
pub async fn index(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1);
    let (posts, total) = state.posts.list_published(page, 10, params.tag.clone()).await?;

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("total", &total);
    ctx.insert("page", &page);
    ctx.insert("tag_filter", &params.tag);

    let template = if is_htmx(&headers) { "posts/_list.html" } else { "posts/list.html" };
    render(&state.tera, template, ctx)
}

/// GET /posts/:slug
///
/// Эталонный тонкий handler в DDD-стиле.
///
/// Handler делает только три вещи:
/// 1. Парсит HTTP-запрос (Path extractor)
/// 2. Вызывает use case
/// 3. Рендерит ответ
///
/// Бизнес-правило "только опубликованные посты видны публично" теперь живёт
/// в GetPostBySlug::execute(), а не здесь. Handler не знает о PostStatus.
///
/// Сравни с исходной версией в git history: там был `if !post.is_published()` прямо в handler'е.
///
/// Поток ошибок (неявный, через ? и From<>):
///   DomainError::NotFound
///     → ApplicationError::Domain(NotFound)
///     → AppError::NotFound
///     → HTTP 404
pub async fn show(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Весь сценарий (найти + проверить статус) инкапсулирован в use case.
    // ? конвертирует ApplicationError → AppError через From (web/errors.rs).
    let post = state.get_post_by_slug.execute(&slug).await?;

    // TODO: загружать теги конкретного поста, а не все теги.
    // Сейчас state.posts.all_tags() возвращает все — это ошибка проектирования,
    // помечена в оригинале как TODO. Пример того что аудит должен был найти.
    let tags = state.posts.all_tags().await?;
    let comments = state.comments.for_post(post.id, false).await?;
    let rendered_content = post.render_content();

    let mut ctx = tera::Context::new();
    ctx.insert("post", &post);
    ctx.insert("rendered_content", &rendered_content);
    ctx.insert("tags", &tags);
    ctx.insert("comments", &comments);

    render(&state.tera, "posts/detail.html", ctx)
}

/// GET /tags/:slug
pub async fn by_tag(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<ListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1);
    let (posts, total) = state
        .posts
        .list_published(page, 10, Some(slug.clone()))
        .await?;

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("total", &total);
    ctx.insert("page", &page);
    ctx.insert("tag_filter", &Some(&slug));

    render(&state.tera, "posts/list.html", ctx)
}
