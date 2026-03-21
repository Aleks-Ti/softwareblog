use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;

use super::{
    handlers::{admin, auth, comments, posts},
    middleware::auth_guard,
    state::AppState,
};

pub fn build(state: AppState) -> Router {
    let public = Router::new()
        .route("/", get(posts::index))
        .route("/posts/:slug", get(posts::show))
        .route("/posts/:slug/comments", post(comments::submit))
        .route("/tags/:slug", get(posts::by_tag));

    let admin_protected = Router::new()
        .route("/admin", get(admin::dashboard))
        .route("/admin/posts/new", get(admin::new_post))
        .route("/admin/posts", post(admin::create_post))
        .route("/admin/posts/:id/edit", get(admin::edit_post))
        .route("/admin/posts/:id", post(admin::update_post))
        .route("/admin/posts/:id/publish", post(admin::publish_post))
        .route("/admin/posts/:id/unpublish", post(admin::unpublish_post))
        .route("/admin/posts/:id/delete", post(admin::delete_post))
        .route("/admin/comments", get(admin::list_comments))
        .route("/admin/comments/:id/approve", post(admin::approve_comment))
        .route("/admin/comments/:id/delete", post(admin::delete_comment))
        .layer(middleware::from_fn_with_state(state.clone(), auth_guard));

    let auth_routes = Router::new()
        .route("/admin/login", get(auth::login_page).post(auth::login))
        .route("/admin/logout", post(auth::logout));

    Router::new()
        .merge(public)
        .merge(admin_protected)
        .merge(auth_routes)
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
