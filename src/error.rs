//! Модуль для обработки ошибок в приложении.
//!
//! Определяет типы ошибок и реализует трейты для работы с ними.

// use std::fmt;
use std::io;
use thiserror::Error;

/// Основной тип ошибки приложения
#[derive(Error, Debug)]
pub enum AppError {
    /// Ошибки ввода-вывода
    #[error("Ошибка ввода-вывода: {0}")]
    IoError(#[from] io::Error),

    /// Ошибки при работе с путями файловой системы
    #[error("Ошибка пути: {0}")]
    PathError(String),

    /// Ошибки при загрузке библиотеки
    #[error("Ошибка загрузки библиотеки: {0}")]
    LibraryError(String),

    /// Ошибки при вызове функций из библиотеки
    #[error("Ошибка вызова функции: {0}")]
    FunctionCallError(String),

    /// Ошибки при определении архитектуры процессора
    #[error("Ошибка определения процессора: {0}")]
    CpuDetectionError(String),

    /// Ошибки из библиотеки core_lib
    #[error("Ошибка в core_lib: {0}")]
    CoreLibError(String),

    /// Прочие ошибки
    #[error("Неизвестная ошибка: {0}")]
    Unknown(String),
}

impl From<libloading::Error> for AppError {
    fn from(err: libloading::Error) -> Self {
        AppError::LibraryError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AppError::Unknown(err.to_string())
    }
}

/// Тип результата для функций, которые могут вернуть ошибку
#[allow(dead_code)]
pub type AppResult<T> = Result<T, AppError>;
