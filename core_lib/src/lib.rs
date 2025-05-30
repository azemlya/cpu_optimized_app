//! Основная библиотека с оптимизациями под различные архитектуры процессоров.
//!
//! Эта библиотека компилируется отдельно для каждой архитектуры процессора
//! с соответствующими оптимизациями.

mod error;
mod runtime;

use log::{debug, info, warn};
use std::sync::Once;
// use error::CoreError;

// Инициализация логгера
static INIT_LOGGER: Once = Once::new();

// Выбор аллокатора памяти в зависимости от features
#[cfg(feature = "jemalloc-allocator")]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(feature = "mimalloc-allocator")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Основная функция, вызываемая из основного приложения.
///
/// # Аргументы
///
/// * `args` - Аргументы командной строки
///
/// # Возвращаемое значение
///
/// Код возврата программы или ошибка
#[no_mangle]
pub fn run(args: Vec<String>) -> Result<i32, Box<dyn std::error::Error>> {
    // Инициализация логгера
    INIT_LOGGER.call_once(|| {
        env_logger::init();
    });

    info!("Запуск библиотеки core_lib");
    debug!("Аргументы: {:?}", args);

    // Вывод информации об используемых оптимизациях
    print_optimization_info();

    // Вывод информации об используемом аллокаторе
    print_allocator_info();

    // Запуск основной логики
    match runtime::run_with_args(args) {
        Ok(exit_code) => {
            info!("Выполнение завершено с кодом: {}", exit_code);
            Ok(exit_code)
        },
        Err(err) => {
            warn!("Ошибка выполнения: {}", err);
            Err(Box::new(err))
        },
    }
}

/// Выводит информацию об используемых оптимизациях
fn print_optimization_info() {
    #[allow(unused_mut)]
    let mut optimizations: Vec<&str> = Vec::new();

    #[cfg(feature = "avx2")]
    optimizations.push("AVX2");

    #[cfg(feature = "avx")]
    optimizations.push("AVX");

    #[cfg(feature = "sse4_2")]
    optimizations.push("SSE4.2");

    #[cfg(feature = "neon")]
    optimizations.push("NEON");

    if optimizations.is_empty() {
        info!("Используются базовые оптимизации");
    } else {
        info!("Используемые оптимизации: {}", optimizations.join(", "));
    }
}

/// Выводит информацию об используемом аллокаторе памяти
fn print_allocator_info() {
    #[cfg(feature = "jemalloc-allocator")]
    info!("Используемый аллокатор: jemalloc");

    #[cfg(feature = "mimalloc-allocator")]
    info!("Используемый аллокатор: mimalloc");

    #[cfg(feature = "system-allocator")]
    info!("Используемый аллокатор: system (стандартный)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_with_empty_args() {
        // Проверка, что функция не вызывает панику с пустыми аргументами
        let args = vec!["program".to_string()];
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimization_info() {
        // Проверка, что функция не вызывает панику
        print_optimization_info();
    }

    #[test]
    fn test_allocator_info() {
        // Проверка, что функция не вызывает панику
        print_allocator_info();
    }
}
