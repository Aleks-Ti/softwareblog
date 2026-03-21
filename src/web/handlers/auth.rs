use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form,
};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::PrivateCookieJar;
use serde::Deserialize;
use time::Duration;

use super::render;
use crate::error::AppError;
use crate::web::state::AppState;

#[derive(Deserialize)]
pub struct LoginForm {
    pub password: String,
}

/// GET /admin/login
pub async fn login_page(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    render(&state.tera, "admin/login.html", tera::Context::new())
}

/// POST /admin/login
pub async fn login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(form): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let login_error = |msg: &str| {
        let mut ctx = tera::Context::new();
        ctx.insert("error", msg);
        render(&state.tera, "admin/login.html", ctx).map(IntoResponse::into_response)
    };

    let Ok(hash) = PasswordHash::new(&state.config.admin_password_hash) else {
        tracing::error!("ADMIN_PASSWORD_HASH in .env is malformed — did you wrap it in quotes?");
        return login_error("Server misconfiguration: password hash is invalid");
    };

    let valid = Argon2::default()
        .verify_password(form.password.as_bytes(), &hash)
        .is_ok();

    if !valid {
        return login_error("Wrong password");
    }

    let cookie = Cookie::build(("auth_session", "1"))
        .path("/admin")
        .http_only(true)
        .max_age(Duration::days(7))
        .build();

    Ok((jar.add(cookie), Redirect::to("/admin")).into_response())
}

/// POST /admin/logout
pub async fn logout(jar: PrivateCookieJar) -> impl IntoResponse {
    let jar = jar.remove(Cookie::from("auth_session"));
    (jar, Redirect::to("/admin/login"))
}
