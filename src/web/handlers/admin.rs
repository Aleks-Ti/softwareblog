use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use uuid::Uuid;

use super::render;
use crate::domain::post::UpdatePost;
use crate::error::AppError;
use crate::web::state::AppState;

// ─── Posts ───────────────────────────────────────────────────────────────────

/// GET /admin
pub async fn dashboard(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let (posts, _) = state.posts.list_all(1, 20).await?;
    let pending = state.comments.pending().await?;

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("pending_count", &pending.len());
    render(&state.tera, "admin/dashboard.html", ctx)
}

/// GET /admin/posts/new
pub async fn new_post(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    render(&state.tera, "admin/post_form.html", tera::Context::new())
}

#[derive(Deserialize)]
pub struct PostForm {
    pub title: String,
    pub content: String,
    pub excerpt: Option<String>,
    /// Comma-separated tag names
    pub tags: Option<String>,
}

/// POST /admin/posts
pub async fn create_post(
    State(state): State<AppState>,
    Form(form): Form<PostForm>,
) -> Result<impl IntoResponse, AppError> {
    let tag_names: Vec<String> = form
        .tags
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let post = state
        .posts
        .create(form.title, form.content, form.excerpt, tag_names)
        .await?;

    Ok(Redirect::to(&format!("/admin/posts/{}/edit", post.id)).into_response())
}

/// GET /admin/posts/:id/edit
pub async fn edit_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let post = state
        .posts
        .get_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Post {id} not found")))?;

    let mut ctx = tera::Context::new();
    ctx.insert("post", &post);
    render(&state.tera, "admin/post_form.html", ctx)
}

/// POST /admin/posts/:id  (HTML forms don't support PUT, so we use POST)
pub async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(form): Form<PostForm>,
) -> Result<impl IntoResponse, AppError> {
    state
        .posts
        .update(
            id,
            UpdatePost {
                title: Some(form.title),
                content: Some(form.content),
                excerpt: form.excerpt,
                tag_ids: None, // TODO: resolve tag names to IDs
            },
        )
        .await?;

    Ok(Redirect::to(&format!("/admin/posts/{id}/edit")).into_response())
}

/// POST /admin/posts/:id/publish
pub async fn publish_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.posts.publish(id).await?;
    Ok(Redirect::to("/admin").into_response())
}

/// POST /admin/posts/:id/unpublish
pub async fn unpublish_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.posts.unpublish(id).await?;
    Ok(Redirect::to("/admin").into_response())
}

/// POST /admin/posts/:id/delete
pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.posts.delete(id).await?;
    Ok(Redirect::to("/admin").into_response())
}

// ─── Comments ────────────────────────────────────────────────────────────────

/// GET /admin/comments
pub async fn list_comments(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let pending = state.comments.pending().await?;
    let mut ctx = tera::Context::new();
    ctx.insert("comments", &pending);
    render(&state.tera, "admin/comments.html", ctx)
}

/// POST /admin/comments/:id/approve
pub async fn approve_comment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.comments.approve(id).await?;
    Ok(Redirect::to("/admin/comments").into_response())
}

/// POST /admin/comments/:id/delete
pub async fn delete_comment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.comments.delete(id).await?;
    Ok(Redirect::to("/admin/comments").into_response())
}
