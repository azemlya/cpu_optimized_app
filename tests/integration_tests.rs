//! Интеграционные тесты для проверки взаимодействия компонентов.
//!
//! Проверяет корректность работы основного приложения и библиотеки.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Вспомогательная функция для получения пути к исполняемому файлу
fn get_executable_path() -> PathBuf {
    let mut path =
        env::current_exe().expect("Не удалось получить путь к текущему исполняемому файлу");
    path.pop(); // Удаляем имя файла
    path.pop(); // Удаляем директорию debug или release
    path.push("debug"); // Добавляем debug, так как тесты обычно запускаются в режиме отладки
    path.push("cpu_optimized_app");

    #[cfg(target_os = "windows")]
    path.set_extension("exe");

    path
}

// Вспомогательная функция для создания тестовой библиотеки
fn create_test_library(lib_dir: &Path, name: &str) -> PathBuf {
    fs::create_dir_all(lib_dir).expect("Не удалось создать директорию для библиотек");

    let lib_ext = if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    let lib_prefix = if cfg!(target_os = "windows") {
        ""
    } else {
        "lib"
    };

    let lib_name = format!("{}{}.{}", lib_prefix, name, lib_ext);
    let lib_path = lib_dir.join(&lib_name);

    // Создаем пустой файл для имитации библиотеки
    fs::write(&lib_path, b"dummy library").expect("Не удалось создать тестовую библиотеку");

    lib_path
}

#[test]
fn test_app_help_output() {
    let executable = get_executable_path();
    if !executable.exists() {
        panic!("Исполняемый файл не найден: {}", executable.display());
    }

    let output = Command::new(&executable)
        .arg("--help")
        .output()
        .expect("Не удалось запустить приложение");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cpu_optimized_app"));
}

#[test]
fn test_app_version_output() {
    let executable = get_executable_path();
    if !executable.exists() {
        panic!("Исполняемый файл не найден: {}", executable.display());
    }

    let output = Command::new(&executable)
        .arg("--version")
        .output()
        .expect("Не удалось запустить приложение");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cpu_optimized_app"));
}

#[test]
#[ignore] // Этот тест требует наличия реальной библиотеки, поэтому игнорируем его при автоматическом запуске
fn test_app_with_forced_library() {
    let executable = get_executable_path();
    if !executable.exists() {
        panic!("Исполняемый файл не найден: {}", executable.display());
    }

    // Создаем тестовую библиотеку
    let mut lib_dir = executable.clone();
    lib_dir.pop();
    lib_dir.push("lib");

    let arch = env::consts::ARCH;
    let lib_path = create_test_library(&lib_dir, &format!("{}_base_system", arch));

    // Запускаем приложение с принудительным указанием библиотеки
    let output = Command::new(&executable)
        .env("FORCE_LIB_PATH", lib_path.to_str().unwrap())
        .output()
        .expect("Не удалось запустить приложение");

    // Проверяем, что приложение запустилось, но завершилось с ошибкой,
    // так как тестовая библиотека не содержит функцию run
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Ошибка") || stderr.contains("error"));
}
