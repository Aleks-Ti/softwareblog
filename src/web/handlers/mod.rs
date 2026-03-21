pub mod admin;
pub mod auth;
pub mod comments;
pub mod posts;

use axum::{http::HeaderMap, response::Html};
use tera::Tera;

use crate::error::AppError;

/// Render a Tera template to HTML.
pub fn render(
    tera: &Tera,
    template: &str,
    ctx: tera::Context,
) -> Result<Html<String>, AppError> {
    let html = tera.render(template, &ctx)?;
    Ok(Html(html))
}

/// Returns true if the request was made by HTMX (hx-request header present).
pub fn is_htmx(headers: &HeaderMap) -> bool {
    headers.contains_key("hx-request")
}
