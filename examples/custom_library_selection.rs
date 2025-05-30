//! Пример принудительного выбора библиотеки в CPU Optimized App.
//!
//! Этот пример демонстрирует, как принудительно выбрать библиотеку
//! для использования в приложении, что может быть полезно для тестирования
//! и отладки.
//!
//! Для запуска примера выполните:
//! ```bash
//! cargo run --example custom_library_selection
//! ```

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Пример принудительного выбора библиотеки в CPU Optimized App");
    println!("===========================================================");

    // Определение пути к исполняемому файлу
    let executable_path = find_executable_path()?;
    println!("Путь к исполняемому файлу: {}", executable_path.display());

    // Получение информации о доступных библиотеках
    let lib_dir = executable_path.parent().unwrap().join("lib");
    println!("\nПоиск библиотек в директории: {}", lib_dir.display());

    if !lib_dir.exists() {
        println!("Директория библиотек не найдена. Создаем директорию...");
        fs::create_dir_all(&lib_dir)?;
    }

    let libraries = find_libraries(&lib_dir)?;

    if libraries.is_empty() {
        println!("Библиотеки не найдены. Запускаем сборку базовой библиотеки...");
        build_base_library(&executable_path)?;

        // Повторный поиск библиотек
        let libraries = find_libraries(&lib_dir)?;
        if libraries.is_empty() {
            return Err("Не удалось найти или собрать библиотеки".into());
        }
    }

    println!("\nНайденные библиотеки:");
    for (i, lib) in libraries.iter().enumerate() {
        println!("{}. {}", i + 1, lib.file_name().unwrap().to_string_lossy());
    }

    // Запуск приложения с каждой найденной библиотекой
    for (i, lib) in libraries.iter().enumerate() {
        println!(
            "\n{}/{}. Запуск приложения с библиотекой: {}",
            i + 1,
            libraries.len(),
            lib.file_name().unwrap().to_string_lossy()
        );

        // Запуск приложения с принудительным выбором библиотеки
        run_with_library(&executable_path, lib)?;

        // Запуск бенчмарка с принудительным выбором библиотеки
        run_benchmark_with_library(&executable_path, lib)?;
    }

    // Демонстрация переопределения архитектуры процессора
    println!("\nДемонстрация переопределения архитектуры процессора:");
    run_with_cpu_override(&executable_path)?;

    println!("\nПример успешно выполнен!");
    Ok(())
}

/// Находит путь к исполняемому файлу
fn find_executable_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = env::current_exe()?;
    path.pop(); // Удаляем имя файла

    // Проверяем, находимся ли мы в директории target
    if path.ends_with("examples") || path.ends_with("debug") || path.ends_with("release") {
        path.pop(); // Выходим из директории examples/debug/release
    }

    if path.ends_with("target") {
        path.pop(); // Выходим из директории target
    }

    // Добавляем путь к исполняемому файлу
    path.push("target");
    path.push("debug");
    path.push("cpu_optimized_app");

    // Добавляем расширение .exe для Windows
    if cfg!(windows) {
        path.set_extension("exe");
    }

    if !path.exists() {
        // Пробуем путь к release версии
        path.pop();
        path.push("release");
        path.push("cpu_optimized_app");

        if cfg!(windows) {
            path.set_extension("exe");
        }
    }

    if !path.exists() {
        return Err(format!("Исполняемый файл не найден: {}", path.display()).into());
    }

    Ok(path)
}

/// Находит все библиотеки в указанной директории
fn find_libraries(lib_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    if !lib_dir.exists() {
        return Ok(Vec::new());
    }

    let mut libraries = Vec::new();

    let lib_ext = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };

    for entry in fs::read_dir(lib_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == lib_ext) {
            libraries.push(path);
        }
    }

    // Сортировка библиотек по имени
    libraries.sort();

    Ok(libraries)
}

/// Собирает базовую библиотеку
fn build_base_library(executable_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = executable_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // Проверяем наличие скрипта сборки
    let build_script = project_dir.join("scripts").join("build.sh");
    let build_script_windows = project_dir.join("scripts").join("build.bat");

    if build_script.exists() || build_script_windows.exists() {
        // Используем скрипт сборки
        let script = if build_script.exists() {
            build_script
        } else {
            build_script_windows
        };

        println!("Запуск скрипта сборки: {}", script.display());

        let status = Command::new(&script)
            .arg("--arch")
            .arg(env::consts::ARCH)
            .arg("--features")
            .arg("base")
            .arg("--allocator")
            .arg("system")
            .current_dir(project_dir)
            .status()?;

        if !status.success() {
            return Err(format!("Скрипт сборки завершился с ошибкой: {}", status).into());
        }
    } else {
        // Используем cargo напрямую
        println!("Скрипт сборки не найден, используем cargo напрямую");

        let status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("-p")
            .arg("core_lib")
            .current_dir(project_dir)
            .status()?;

        if !status.success() {
            return Err(format!("Сборка библиотеки завершилась с ошибкой: {}", status).into());
        }

        // Копирование библиотеки в директорию lib
        let lib_dir = executable_path.parent().unwrap().join("lib");
        fs::create_dir_all(&lib_dir)?;

        let lib_name = format!(
            "{}{}_{}_system.{}",
            if cfg!(windows) { "" } else { "lib" },
            env::consts::ARCH,
            "base",
            if cfg!(windows) {
                "dll"
            } else if cfg!(target_os = "macos") {
                "dylib"
            } else {
                "so"
            }
        );

        let src_lib = project_dir
            .join("target")
            .join("release")
            .join(if cfg!(windows) {
                "core_lib.dll"
            } else {
                "libcore_lib.so"
            });

        let dst_lib = lib_dir.join(lib_name);

        println!(
            "Копирование библиотеки из {} в {}",
            src_lib.display(),
            dst_lib.display()
        );
        fs::copy(src_lib, dst_lib)?;
    }

    Ok(())
}

/// Запускает приложение с принудительным выбором библиотеки
fn run_with_library(executable: &Path, library: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Запуск приложения с библиотекой: {}", library.display());

    let output = Command::new(executable)
        .env("FORCE_LIB_PATH", library)
        .env("RUST_LOG", "info")
        .arg("info")
        .output()?;

    if !output.status.success() {
        println!("Команда завершилась с ошибкой: {}", output.status);
    }

    println!("Стандартный вывод:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("Стандартный вывод ошибок:");
    println!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}

/// Запускает бенчмарк с принудительным выбором библиотеки
fn run_benchmark_with_library(
    executable: &Path,
    library: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Запуск бенчмарка с библиотекой: {}", library.display());

    let output = Command::new(executable)
        .env("FORCE_LIB_PATH", library)
        .env("RUST_LOG", "info")
        .arg("benchmark")
        .arg("--iterations=10")
        .arg("--size=64")
        .output()?;

    if !output.status.success() {
        println!("Команда завершилась с ошибкой: {}", output.status);
    }

    println!("Стандартный вывод:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("Стандартный вывод ошибок:");
    println!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}

/// Запускает приложение с переопределением архитектуры процессора
fn run_with_cpu_override(executable: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Запуск приложения с переопределением архитектуры процессора");

    let output = Command::new(executable)
        .env("CPU_VENDOR", "TestVendor")
        .env("CPU_MODEL", "TestModel")
        .env("CPU_FEATURES", "avx2,avx,sse4.2")
        .env("RUST_LOG", "debug")
        .arg("info")
        .output()?;

    if !output.status.success() {
        println!("Команда завершилась с ошибкой: {}", output.status);
    }

    println!("Стандартный вывод:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("Стандартный вывод ошибок:");
    println!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}
