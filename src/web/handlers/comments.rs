use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Form,
};
use serde::Deserialize;
use super::render;
use crate::domain::comment::CreateComment;
use crate::error::AppError;
use crate::web::state::AppState;

#[derive(Deserialize)]
pub struct CommentForm {
    pub author_name: String,
    pub author_email: String,
    pub content: String,
}

/// POST /posts/:slug/comments  (HTMX form submission)
///
/// Returns an HTML fragment with the pending-approval notice,
/// which HTMX appends to the comments list.
pub async fn submit(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Form(form): Form<CommentForm>,
) -> Result<impl IntoResponse, AppError> {
    let post = state
        .posts
        .get_by_slug(&slug)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".into()))?;

    let _comment = state
        .comments
        .submit(CreateComment {
            post_id: post.id,
            author_name: form.author_name,
            author_email: form.author_email,
            content: form.content,
        })
        .await?;

    // Return a small HTML fragment that HTMX swaps in.
    let mut ctx = tera::Context::new();
    ctx.insert("message", "Your comment has been submitted and is awaiting approval.");
    render(&state.tera, "comments/_submitted.html", ctx)
}
