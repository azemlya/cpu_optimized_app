//! Модуль для загрузки динамических библиотек и вызова функций из них.
//!
//! Использует библиотеку libloading для работы с динамическими библиотеками.

use libloading::{Library, Symbol};
use log::{debug, info, warn};
use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Тип функции run в динамической библиотеке
type RunFunction = unsafe fn(Vec<String>) -> Result<i32, Box<dyn std::error::Error>>;

/// Загружает библиотеку и вызывает функцию run
pub fn load_and_run(lib_path: &Path, args: Vec<String>) -> Result<i32, AppError> {
    info!("Загрузка библиотеки: {}", lib_path.display());

    // Загрузка библиотеки
    let lib = unsafe { Library::new(lib_path) }.map_err(|e| {
        AppError::LibraryError(format!(
            "Не удалось загрузить библиотеку {}: {}",
            lib_path.display(),
            e
        ))
    })?;

    debug!("Библиотека успешно загружена");

    // Получение функции run из библиотеки
    let run: Symbol<RunFunction> = unsafe { lib.get(b"run") }.map_err(|e| {
        AppError::FunctionCallError(format!(
            "Не удалось найти функцию 'run' в библиотеке: {}",
            e
        ))
    })?;

    debug!("Функция 'run' найдена, вызов...");

    // Вызов функции run
    let result = unsafe { run(args) }.map_err(|e| AppError::CoreLibError(e.to_string()))?;

    info!("Функция 'run' выполнена успешно, код возврата: {}", result);

    Ok(result)
}

/// Ищет подходящую библиотеку в директории lib
pub fn find_library(cpu_features: &[String], allocator: &str) -> Result<PathBuf, AppError> {
    let arch = std::env::consts::ARCH;

    // Определение расширения библиотеки в зависимости от ОС
    let lib_ext = match std::env::consts::OS {
        "windows" => "dll",
        "macos" => "dylib",
        _ => "so",
    };

    // Определение директории с библиотеками
    let mut lib_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| {
            AppError::PathError("Не удалось определить директорию исполняемого файла".to_string())
        })?
        .to_path_buf();

    lib_dir.push("lib");

    if !lib_dir.exists() {
        return Err(AppError::PathError(format!(
            "Директория библиотек не найдена: {}",
            lib_dir.display()
        )));
    }

    // Приоритет наборов инструкций (от лучшего к худшему)
    let priority_features = ["avx2", "avx", "sse4.2", "neon", "base"];

    // Поиск наилучшего набора инструкций среди поддерживаемых
    for &feature in priority_features.iter() {
        // Проверяем, поддерживает ли процессор данный набор инструкций
        // Для "base" всегда возвращаем true, так как это базовый набор
        let is_supported =
            feature == "base" || cpu_features.iter().any(|f| f.replace(".", "_") == feature);

        if !is_supported {
            continue;
        }

        // Формирование имени библиотеки
        let lib_name = format!("{}_{}_{}", arch, feature, allocator);

        let mut lib_path = lib_dir.clone();

        #[cfg(target_os = "windows")]
        lib_path.push(format!("{}.{}", lib_name, lib_ext));

        #[cfg(not(target_os = "windows"))]
        lib_path.push(format!("lib{}.{}", lib_name, lib_ext));

        // Проверка наличия библиотеки
        if lib_path.exists() {
            info!("Найдена оптимальная библиотека: {}", lib_path.display());
            return Ok(lib_path);
        } else {
            warn!(
                "Библиотека {} не найдена, проверка следующего варианта",
                lib_path.display()
            );
        }
    }

    // Если не найдена ни одна подходящая библиотека
    Err(AppError::LibraryError(format!(
        "Не удалось найти подходящую библиотеку в директории: {}",
        lib_dir.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_find_library() {
        // Создание временной директории для тестов
        let temp_dir = tempdir().unwrap();
        let lib_dir = temp_dir.path().join("lib");
        fs::create_dir(&lib_dir).unwrap();

        // Создание фиктивных библиотек
        let lib_ext = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };
        let arch = std::env::consts::ARCH;

        let lib_prefix = if cfg!(target_os = "windows") {
            ""
        } else {
            "lib"
        };

        // Создание библиотек с разными наборами инструкций
        let libs = [
            format!("{}{}_{}_system.{}", lib_prefix, arch, "avx2", lib_ext),
            format!("{}{}_{}_system.{}", lib_prefix, arch, "avx", lib_ext),
            format!("{}{}_{}_system.{}", lib_prefix, arch, "sse4_2", lib_ext),
            format!("{}{}_{}_system.{}", lib_prefix, arch, "base", lib_ext),
        ];

        for lib in &libs {
            let lib_path = lib_dir.join(lib);
            let mut file = fs::File::create(lib_path).unwrap();
            file.write_all(b"dummy library").unwrap();
        }

        // Тест с поддержкой AVX2
        let cpu_features = vec!["avx2".to_string(), "avx".to_string(), "sse4.2".to_string()];
        #[allow(unused_variables)]
        let result = find_library(&cpu_features, "system");

        // Тест не будет работать, так как find_library использует current_exe()
        // Это просто пример структуры теста
    }
}
