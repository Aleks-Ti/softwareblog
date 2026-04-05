// AppError переехал в web::errors — там ему место (он знает про HTTP).
//
// Этот re-export сохраняет все существующие `use crate::error::AppError`
// работоспособными без изменений в handler'ах.
//
// TODO (самостоятельная работа): когда будешь рефакторить handler'ы,
// замени `use crate::error::AppError` на `use crate::web::errors::AppError`.
// После этого этот файл можно удалить.
pub use crate::web::errors::AppError;
