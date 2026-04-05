use thiserror::Error;

use crate::domain::errors::DomainError;

/// Ошибки уровня application (use cases, сервисы-оркестраторы).
///
/// Зачем отдельный тип, а не просто DomainError?
///
/// 1. **Изоляция слоёв**: application layer знает о domain, но не о web/HTTP.
///    Если бы use case возвращал AppError, он бы знал про HTTP-статусы — это нарушение.
///
/// 2. **Расширяемость**: в будущем здесь появятся варианты для внешних сервисов,
///    таймаутов, rate limiting и т.д. — без засорения DomainError.
///
/// 3. **Явность маппинга**: From<ApplicationError> for AppError в web-слое
///    это явный код, который компилятор проверяет. Добавил новый вариант —
///    получил ошибку компиляции, если забыл его обработать.
///
/// Цепочка конверсии (через ? оператор):
///   DomainError  →(From)→  ApplicationError  →(From)→  AppError
///
/// Аналог в Python/FastAPI: HTTPException в handler'е — это "ApplicationError → HTTP mapping".
/// Здесь то же самое, но явно разделено на уровни.
#[derive(Error, Debug)]
pub enum ApplicationError {
    /// Доменная ошибка прозрачно пробрасывается наверх.
    ///
    /// `#[from]` генерирует `impl From<DomainError> for ApplicationError`.
    /// Благодаря этому в сервисах работает `repo.find()?` — ? автоматически
    /// конвертирует DomainError в ApplicationError::Domain(e).
    #[error(transparent)]
    Domain(#[from] DomainError),
}
