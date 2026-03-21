use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use tera::Tera;

use crate::application::{comment_service::CommentService, post_service::PostService};
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub tera: Arc<Tera>,
    pub posts: Arc<PostService>,
    pub comments: Arc<CommentService>,
    pub config: Arc<Config>,
    /// Key for signing/encrypting private cookies (auth session).
    pub cookie_key: Key,
}

/// Lets Axum extract `Key` directly from `AppState`.
/// Required for `PrivateCookieJar` to work in handlers and middleware.
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}
