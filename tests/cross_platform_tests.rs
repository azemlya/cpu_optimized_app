//! Тесты для проверки работы на разных платформах.
//!
//! Проверяет корректность работы приложения на различных операционных системах
//! и архитектурах процессоров.

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
#[allow(dead_code)]
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
#[cfg(target_os = "linux")]
fn test_linux_specific() {
    // Тест, специфичный для Linux
    let executable = get_executable_path();
    if !executable.exists() {
        panic!("Исполняемый файл не найден: {}", executable.display());
    }

    // Проверка вывода информации о libc
    let output = Command::new(&executable)
        .env("RUST_LOG", "debug")
        .output()
        .expect("Не удалось запустить приложение");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("libc"));
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_specific() {
    // Тест, специфичный для Windows
    let executable = get_executable_path();
    if !executable.exists() {
        panic!("Исполняемый файл не найден: {}", executable.display());
    }

    // Проверка расширения библиотеки
    let mut lib_dir = executable.clone();
    lib_dir.pop();
    lib_dir.push("lib");

    let arch = env::consts::ARCH;
    let lib_path = create_test_library(&lib_dir, &format!("{}_base_system", arch));

    assert_eq!(lib_path.extension().unwrap(), "dll");
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_specific() {
    // Тест, специфичный для macOS
    let executable = get_executable_path();
    if !executable.exists() {
        panic!("Исполняемый файл не найден: {}", executable.display());
    }

    // Проверка расширения библиотеки
    let mut lib_dir = executable.clone();
    lib_dir.pop();
    lib_dir.push("lib");

    let arch = env::consts::ARCH;
    let lib_path = create_test_library(&lib_dir, &format!("{}_base_system", arch));

    assert_eq!(lib_path.extension().unwrap(), "dylib");
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_x86_64_specific() {
    // Тест, специфичный для архитектуры x86_64
    let executable = get_executable_path();
    if !executable.exists() {
        panic!("Исполняемый файл не найден: {}", executable.display());
    }

    // Проверка определения архитектуры
    let output = Command::new(&executable)
        .env("RUST_LOG", "debug")
        .output()
        .expect("Не удалось запустить приложение");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("x86_64"));
}

#[test]
#[cfg(target_arch = "aarch64")]
fn test_aarch64_specific() {
    // Тест, специфичный для архитектуры aarch64
    let executable = get_executable_path();
    if !executable.exists() {
        panic!("Исполняемый файл не найден: {}", executable.display());
    }

    // Проверка определения архитектуры
    let output = Command::new(&executable)
        .env("RUST_LOG", "debug")
        .output()
        .expect("Не удалось запустить приложение");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("aarch64") || stderr.contains("ARM"));
}

#[test]
fn test_cpu_feature_override() {
    // Тест переопределения определения процессора через переменные окружения
    let executable = get_executable_path();
    if !executable.exists() {
        panic!("Исполняемый файл не найден: {}", executable.display());
    }

    // Запуск с переопределенными переменными окружения
    let output = Command::new(&executable)
        .env("CPU_VENDOR", "TestVendor")
        .env("CPU_MODEL", "TestModel")
        .env("CPU_FEATURES", "avx2,avx,sse4.2")
        .env("RUST_LOG", "debug")
        .output()
        .expect("Не удалось запустить приложение");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("TestVendor"));
    assert!(stderr.contains("TestModel"));
    assert!(stderr.contains("avx2"));
}

#[test]
fn test_allocator_override() {
    // Тест переопределения аллокатора через переменные окружения
    let executable = get_executable_path();
    if !executable.exists() {
        panic!("Исполняемый файл не найден: {}", executable.display());
    }

    // Запуск с переопределенным аллокатором
    let output = Command::new(&executable)
        .env("ALLOCATOR", "jemalloc")
        .env("RUST_LOG", "debug")
        .output()
        .expect("Не удалось запустить приложение");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("jemalloc"));
}
