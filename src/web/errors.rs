use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use thiserror::Error;

use crate::application::errors::ApplicationError;
use crate::domain::errors::DomainError;

/// Ошибки HTTP-слоя.
///
/// # Зачем три уровня ошибок?
///
/// ```
/// DomainError          — что случилось (бизнес-язык)
///      ↓ From<>
/// ApplicationError     — что пошло не так в use case
///      ↓ From<>
/// AppError             — как сообщить клиенту (HTTP-язык)
/// ```
///
/// В Python/FastAPI это выглядит как:
/// - raise PostNotFoundError() в домене/сервисе
/// - @app.exception_handler(PostNotFoundError) → JSONResponse(status_code=404)
///
/// В Rust это делается через From<> трейты — конверсия явная и проверяется компилятором.
/// Добавил новый вариант DomainError → получил ошибку компиляции в `From<DomainError> for AppError`
/// → не забудешь обработать новый случай.
///
/// # Что убрано по сравнению с исходным error.rs
/// - `Database(#[from] sqlx::Error)` — web-слой не знает про sqlx. Инфра конвертирует сама.
/// - `Internal(#[from] anyhow::Error)` — заменён на `Internal(String)`, достаточно.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Ошибки рендеринга шаблонов — это web-уровень, правомерно быть здесь.
    #[error("Template error: {0}")]
    Template(#[from] tera::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Маппинг ApplicationError → AppError.
///
/// Здесь web-слой **принимает решение** о том, как преподнести ошибку клиенту.
/// Например, DomainError::Internal → AppError::Internal (детали скрыты, только 500).
///
/// Это намеренно **явный** код — не `#[from]`, а ручной match.
/// Явность означает: добавление нового варианта ApplicationError вызовет ошибку компиляции.
impl From<ApplicationError> for AppError {
    fn from(err: ApplicationError) -> Self {
        match err {
            ApplicationError::Domain(domain_err) => domain_err.into(),
        }
    }
}

/// Маппинг DomainError → AppError напрямую (используется в handler'ах где удобно).
impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound(msg) => AppError::NotFound(msg),
            DomainError::Conflict(msg) => AppError::Conflict(msg),
            DomainError::ValidationError(msg) => AppError::BadRequest(msg),
            DomainError::Unauthorized => AppError::Unauthorized,
            DomainError::Internal(msg) => {
                // Логируем полную информацию, но клиенту отдаём только "500".
                // Детали инфраструктурных ошибок не должны утекать наружу.
                tracing::error!("Internal domain/infra error: {msg}");
                AppError::Internal("Internal server error".to_string())
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Template(e) => {
                tracing::error!("Template error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let html = format!(
            r#"<!DOCTYPE html><html><body><h1>{status}</h1><p>{message}</p></body></html>"#
        );
        (status, Html(html)).into_response()
    }
}
