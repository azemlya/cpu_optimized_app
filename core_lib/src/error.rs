//! Модуль для обработки ошибок в библиотеке core_lib.
//!
//! Определяет типы ошибок и реализует трейты для работы с ними.

// use std::fmt;
use std::io;
use thiserror::Error;

/// Основной тип ошибки библиотеки
#[derive(Error, Debug)]
pub enum CoreError {
    /// Ошибки ввода-вывода
    #[error("Ошибка ввода-вывода: {0}")]
    IoError(#[from] io::Error),

    /// Ошибки при разборе аргументов
    #[error("Ошибка разбора аргументов: {0}")]
    ArgParseError(String),

    /// Ошибки при выполнении асинхронных операций
    #[error("Ошибка асинхронного выполнения: {0}")]
    AsyncError(String),

    /// Ошибки при работе с данными
    #[error("Ошибка обработки данных: {0}")]
    #[allow(dead_code)]
    DataError(String),

    /// Ошибки при сериализации/десериализации
    #[error("Ошибка сериализации/десериализации: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// Прочие ошибки
    #[error("Неизвестная ошибка: {0}")]
    Unknown(String),
}

impl From<Box<dyn std::error::Error>> for CoreError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        CoreError::Unknown(err.to_string())
    }
}

impl From<tokio::task::JoinError> for CoreError {
    fn from(err: tokio::task::JoinError) -> Self {
        CoreError::AsyncError(format!("Ошибка при выполнении асинхронной задачи: {}", err))
    }
}

impl From<clap::Error> for CoreError {
    fn from(err: clap::Error) -> Self {
        CoreError::ArgParseError(err.to_string())
    }
}

/// Тип результата для функций, которые могут вернуть ошибку
pub type CoreResult<T> = Result<T, CoreError>;

/// Создает ошибку с сообщением
#[allow(dead_code)]
pub fn error<T, S: Into<String>>(message: S) -> CoreResult<T> {
    Err(CoreError::Unknown(message.into()))
}

/// Создает ошибку ввода-вывода с сообщением
#[allow(dead_code)]
pub fn io_error<T, S: Into<String>>(message: S) -> CoreResult<T> {
    Err(CoreError::IoError(io::Error::new(
        io::ErrorKind::Other,
        message.into(),
    )))
}

/// Создает ошибку обработки данных с сообщением
#[allow(dead_code)]
pub fn data_error<T, S: Into<String>>(message: S) -> CoreResult<T> {
    Err(CoreError::DataError(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CoreError::Unknown("тестовая ошибка".to_string());
        assert_eq!(err.to_string(), "Неизвестная ошибка: тестовая ошибка");

        let err = CoreError::IoError(io::Error::new(io::ErrorKind::NotFound, "файл не найден"));
        assert!(err.to_string().contains("Ошибка ввода-вывода"));
    }

    #[test]
    fn test_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "нет доступа");
        let core_err: CoreError = io_err.into();

        match core_err {
            CoreError::IoError(_) => assert!(true),
            _ => panic!("Неправильная конвертация ошибки"),
        }
    }
}
