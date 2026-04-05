pub mod comment_repo;
pub mod post_repo;
pub mod tag_repo;

use crate::domain::errors::DomainError;

/// Конвертация sqlx::Error → DomainError.
///
/// Это граница адаптера в Hexagonal Architecture:
/// внешний мир (sqlx, PostgreSQL) переводится на язык домена.
///
/// Почему impl здесь, а не в domain/errors.rs?
/// — domain/errors.rs не должен знать про sqlx (иначе домен зависит от инфраструктуры).
/// — Этот impl определён в инфраструктурном модуле, который sqlx и так уже импортирует.
/// — В Rust можно имплементировать трейт из одного крейта для типа из другого,
///   если хотя бы одно из них — local type (DomainError наш → всё ок).
///
/// Благодаря этому impl, во всех repo-файлах работает `sqlx_result?`
/// когда функция возвращает `Result<_, DomainError>`.
impl From<sqlx::Error> for DomainError {
    fn from(e: sqlx::Error) -> Self {
        // Логируем здесь, потому что выше (в domain/application) это уже просто Internal(String).
        tracing::error!("Database error: {e}");
        DomainError::Internal(e.to_string())
    }
}
