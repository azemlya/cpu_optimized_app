//! Пример базового использования CPU Optimized App.
//!
//! Этот пример демонстрирует, как использовать основные возможности приложения.
//! Для запуска примера выполните:
//! ```bash
//! cargo run --example basic_usage
//! ```

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Пример базового использования CPU Optimized App");
    println!("===============================================");

    // Определение пути к исполняемому файлу
    let executable_path = find_executable_path()?;
    println!("Путь к исполняемому файлу: {}", executable_path.display());

    // Запуск приложения с выводом версии
    println!("\n1. Запуск приложения с выводом версии:");
    run_command(&executable_path, &["--version"])?;

    // Запуск приложения с выводом справки
    println!("\n2. Запуск приложения с выводом справки:");
    run_command(&executable_path, &["--help"])?;

    // Запуск приложения с командой info
    println!("\n3. Запуск приложения с командой info:");
    run_command(&executable_path, &["info"])?;

    // Запуск бенчмарка с небольшим количеством итераций
    println!("\n4. Запуск бенчмарка с небольшим количеством итераций:");
    run_command(
        &executable_path,
        &["benchmark", "--iterations=10", "--size=64"],
    )?;

    // Запуск приложения с переменными окружения
    println!("\n5. Запуск приложения с переменными окружения:");
    run_command_with_env(&executable_path, &["info"], &[("RUST_LOG", "debug")])?;

    // Запуск приложения с принудительным выбором библиотеки
    // Примечание: этот пример может не работать, если библиотека не существует
    println!("\n6. Запуск приложения с принудительным выбором библиотеки (может не работать):");
    let lib_path = executable_path
        .parent()
        .unwrap()
        .join("lib")
        .join(get_lib_name());
    if lib_path.exists() {
        run_command_with_env(
            &executable_path,
            &["info"],
            &[("FORCE_LIB_PATH", lib_path.to_str().unwrap())],
        )?;
    } else {
        println!(
            "Библиотека {} не найдена, пропуск этого примера",
            lib_path.display()
        );
    }

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

/// Запускает команду и выводит результат
fn run_command(executable: &PathBuf, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(executable).args(args).output()?;

    if !output.status.success() {
        println!("Команда завершилась с ошибкой: {}", output.status);
    }

    println!("Стандартный вывод:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("Стандартный вывод ошибок:");
    println!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}

/// Запускает команду с переменными окружения и выводит результат
fn run_command_with_env(
    executable: &PathBuf,
    args: &[&str],
    env_vars: &[(&str, &str)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::new(executable);
    command.args(args);

    for (key, value) in env_vars {
        command.env(key, value);
    }

    let output = command.output()?;

    if !output.status.success() {
        println!("Команда завершилась с ошибкой: {}", output.status);
    }

    println!("Стандартный вывод:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("Стандартный вывод ошибок:");
    println!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}

/// Возвращает имя библиотеки в зависимости от ОС и архитектуры
fn get_lib_name() -> String {
    let arch = env::consts::ARCH;
    let lib_prefix = if cfg!(windows) { "" } else { "lib" };
    let lib_ext = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };

    // Выбираем базовую библиотеку, которая должна существовать
    format!("{}{}_{}_system.{}", lib_prefix, arch, "base", lib_ext)
}
