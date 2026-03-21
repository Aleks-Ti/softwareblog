use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::PrivateCookieJar;

use crate::web::state::AppState;

/// Middleware that protects /admin/* routes.
/// Redirects to /admin/login if the auth_session cookie is absent.
pub async fn auth_guard(
    State(_state): State<AppState>,
    jar: PrivateCookieJar,
    request: Request,
    next: Next,
) -> Response {
    if jar.get("auth_session").is_some() {
        next.run(request).await
    } else {
        Redirect::to("/admin/login").into_response()
    }
}
