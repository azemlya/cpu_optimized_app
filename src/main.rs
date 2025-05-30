//! Основной исполняемый файл проекта.
//!
//! Определяет архитектуру процессора и загружает соответствующую
//! динамическую библиотеку, оптимизированную для этой архитектуры.

mod cpu_detection;
mod error;
mod lib_loader;

use std::env;
use std::path::PathBuf;
use std::process;

use error::AppError;
use log::{debug, error, info};

fn main() {
    // Инициализация логгера
    env_logger::init();

    // Получение аргументов командной строки
    let args: Vec<String> = env::args().collect();

    // Запуск основной логики программы
    match run(args) {
        Ok(exit_code) => {
            debug!("Программа завершилась с кодом: {}", exit_code);
            process::exit(exit_code);
        },
        Err(err) => {
            error!("Ошибка выполнения программы: {}", err);
            eprintln!("Ошибка: {}", err);
            process::exit(1);
        },
    }
}

/// Основная логика программы
fn run(args: Vec<String>) -> Result<i32, AppError> {
    // Вывод информации о системе
    print_system_info()?;

    // Определение архитектуры процессора
    let cpu_info = cpu_detection::detect_cpu()?;
    eprintln!("Тип процессора: {}", cpu_info.vendor);
    eprintln!("Модель процессора: {}", cpu_info.model);
    eprintln!("Поддерживаемые наборы инструкций: {:?}", cpu_info.features);

    // Проверка принудительного выбора библиотеки через переменные окружения
    let lib_path = if let Ok(forced_lib) = env::var("FORCE_LIB_PATH") {
        let path = PathBuf::from(forced_lib);
        if !path.exists() {
            return Err(AppError::PathError(format!(
                "Принудительно указанная библиотека не найдена: {}",
                path.display()
            )));
        }
        info!(
            "Используется принудительно указанная библиотека: {}",
            path.display()
        );
        path
    } else {
        // Определение аллокатора (по умолчанию system)
        let allocator = env::var("ALLOCATOR").unwrap_or_else(|_| "system".to_string());
        eprintln!("Используемый аллокатор: {}", allocator);

        // Поиск подходящей библиотеки
        lib_loader::find_library(&cpu_info.features, &allocator)?
    };

    eprintln!("Выбранная динамическая библиотека: {}", lib_path.display());

    // Загрузка библиотеки и вызов функции run
    let result = lib_loader::load_and_run(&lib_path, args)?;

    Ok(result)
}

/// Выводит информацию о системе
fn print_system_info() -> Result<(), AppError> {
    // Вывод информации об ОС
    let os_info = format!("{} {}", env::consts::OS, env::consts::ARCH);
    eprintln!("Операционная система: {}", os_info);

    // Вывод информации о версии libc
    let libc_version = get_libc_version()?;
    eprintln!("Версия libc: {}", libc_version);

    // Вывод информации о версии Rust
    eprintln!("Версия Rust: {}", env!("CARGO_PKG_RUST_VERSION", "unknown"));

    // Вывод информации о версии приложения
    eprintln!("Версия приложения: {}", env!("CARGO_PKG_VERSION"));

    Ok(())
}

/// Получает версию libc
fn get_libc_version() -> Result<String, AppError> {
    // Реализация зависит от платформы
    #[cfg(target_os = "linux")]
    {
        // На Linux можно получить версию через libc
        use std::ffi::CStr;
        unsafe {
            let version = libc::gnu_get_libc_version();
            let c_str = CStr::from_ptr(version);
            Ok(c_str.to_string_lossy().into_owned())
        }
    }

    // Для других ОС просто возвращаем заглушку
    #[cfg(not(target_os = "linux"))]
    {
        Ok("Неизвестно".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_system_info() {
        // Проверка, что функция не вызывает панику
        print_system_info().unwrap();
    }
}
