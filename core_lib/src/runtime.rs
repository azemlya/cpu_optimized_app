//! Модуль с основной логикой выполнения библиотеки.
//!
//! Содержит функции для обработки аргументов командной строки
//! и запуска асинхронной среды выполнения.

use clap::{Parser, Subcommand};
use log::{debug, info};
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;

use crate::error::{CoreError, CoreResult};

/// Структура для разбора аргументов командной строки
#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about = "Оптимизированная библиотека для различных архитектур процессоров"
)]
struct Args {
    /// Подкоманда для выполнения
    #[clap(subcommand)]
    command: Option<Command>,

    /// Режим подробного вывода
    #[clap(short, long)]
    verbose: bool,

    /// Дополнительные аргументы
    #[clap(flatten)]
    global_opts: GlobalOpts,
}

/// Глобальные опции, доступные для всех команд
#[derive(Parser, Debug)]
struct GlobalOpts {
    /// Количество потоков для асинхронной среды выполнения
    #[clap(long, default_value = "0")]
    threads: usize,

    /// Размер стека для асинхронных задач (в КБ)
    #[clap(long, default_value = "2048")]
    stack_size: usize,

    /// Максимальное количество одновременных задач
    #[clap(long, default_value = "100")]
    max_tasks: usize,
}

/// Подкоманды приложения
#[derive(Subcommand, Debug)]
enum Command {
    /// Запуск тестовой нагрузки
    Benchmark {
        /// Количество итераций
        #[clap(short, long, default_value = "1000")]
        iterations: usize,

        /// Размер данных для обработки (в КБ)
        #[clap(short, long, default_value = "1024")]
        size: usize,
    },

    /// Обработка данных
    Process {
        /// Путь к входному файлу
        #[clap(short, long)]
        input: String,

        /// Путь к выходному файлу
        #[clap(short, long)]
        output: String,

        /// Тип обработки (simple, advanced)
        #[clap(short, long, default_value = "simple")]
        mode: String,
    },

    /// Вывод информации о системе
    Info,
}

/// Запускает основную логику с аргументами командной строки
pub fn run_with_args(args: Vec<String>) -> CoreResult<i32> {
    // Разбор аргументов командной строки
    let args = match Args::try_parse_from(args) {
        Ok(args) => args,
        Err(err) => {
            // Для --help и --version возвращаем успешный код
            if err.kind() == clap::error::ErrorKind::DisplayHelp
                || err.kind() == clap::error::ErrorKind::DisplayVersion
            {
                eprintln!("{}", err);
                return Ok(0);
            }
            return Err(CoreError::ArgParseError(err.to_string()));
        },
    };

    // Настройка уровня логирования
    if args.verbose {
        debug!("Включен режим подробного вывода");
    }

    // Создание асинхронной среды выполнения
    let runtime = create_runtime(&args.global_opts)?;

    // Запуск основной логики в асинхронной среде
    runtime.block_on(async {
        match args.command {
            Some(Command::Benchmark { iterations, size }) => run_benchmark(iterations, size).await,
            Some(Command::Process {
                input,
                output,
                mode,
            }) => process_data(&input, &output, &mode).await,
            Some(Command::Info) => print_system_info().await,
            None => {
                // Если команда не указана, выполняем действие по умолчанию
                run_default().await
            },
        }
    })
}

/// Создает асинхронную среду выполнения с заданными параметрами
fn create_runtime(opts: &GlobalOpts) -> CoreResult<Runtime> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();

    // Настройка количества потоков
    if opts.threads > 0 {
        builder.worker_threads(opts.threads);
        debug!("Установлено количество потоков: {}", opts.threads);
    }

    // Настройка размера стека
    builder.thread_stack_size(opts.stack_size * 1024);
    debug!("Установлен размер стека: {} КБ", opts.stack_size);

    // Настройка максимального количества задач
    builder.max_blocking_threads(opts.max_tasks);
    debug!(
        "Установлено максимальное количество задач: {}",
        opts.max_tasks
    );

    // Включение всех возможностей
    builder.enable_all();

    // Создание среды выполнения
    builder.build().map_err(|e| {
        CoreError::AsyncError(format!(
            "Не удалось создать асинхронную среду выполнения: {}",
            e
        ))
    })
}

/// Запускает тестовую нагрузку
async fn run_benchmark(iterations: usize, size: usize) -> CoreResult<i32> {
    info!(
        "Запуск бенчмарка: {} итераций, размер данных: {} КБ",
        iterations, size
    );

    let start = Instant::now();

    // Создание тестовых данных
    let data = vec![0u8; size * 1024];
    let data = Arc::new(data);

    // Запуск задач
    let mut handles = Vec::with_capacity(iterations);

    for i in 0..iterations {
        let data_clone = Arc::clone(&data);
        let handle = tokio::spawn(async move {
            // Имитация обработки данных
            let mut sum = 0u64;
            for byte in data_clone.iter() {
                sum = sum.wrapping_add(*byte as u64);
            }
            (i, sum)
        });
        handles.push(handle);
    }

    // Ожидание завершения всех задач
    let mut results = Vec::with_capacity(iterations);
    for handle in handles {
        let result = handle.await?;
        results.push(result);
    }

    let elapsed = start.elapsed();
    info!("Бенчмарк завершен за {:?}", elapsed);
    info!(
        "Среднее время на итерацию: {:?}",
        elapsed / iterations as u32
    );

    Ok(0)
}

/// Обрабатывает данные из входного файла и сохраняет в выходной
async fn process_data(input: &str, output: &str, mode: &str) -> CoreResult<i32> {
    info!(
        "Обработка данных: вход={}, выход={}, режим={}",
        input, output, mode
    );

    // Чтение входного файла
    let data = tokio::fs::read(input).await.map_err(|e| {
        CoreError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Не удалось прочитать входной файл: {}", e),
        ))
    })?;

    // Обработка данных в зависимости от режима
    let processed_data = match mode {
        "simple" => {
            // Простая обработка (например, копирование)
            data
        },
        "advanced" => {
            // Продвинутая обработка (например, инверсия байтов)
            let mut result = data.clone();
            for byte in result.iter_mut() {
                *byte = !*byte;
            }
            result
        },
        _ => {
            return Err(CoreError::ArgParseError(format!(
                "Неизвестный режим обработки: {}",
                mode
            )));
        },
    };

    // Запись в выходной файл
    tokio::fs::write(output, processed_data)
        .await
        .map_err(|e| {
            CoreError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Не удалось записать выходной файл: {}", e),
            ))
        })?;

    info!("Обработка данных завершена успешно");
    Ok(0)
}

/// Выводит информацию о системе
async fn print_system_info() -> CoreResult<i32> {
    info!("Вывод информации о системе");

    // Информация о библиотеке
    println!("Версия библиотеки: {}", env!("CARGO_PKG_VERSION"));

    // Информация об используемых оптимизациях
    #[cfg(feature = "avx2")]
    println!("Оптимизация: AVX2");

    #[cfg(feature = "avx")]
    println!("Оптимизация: AVX");

    #[cfg(feature = "sse4_2")]
    println!("Оптимизация: SSE4.2");

    #[cfg(feature = "neon")]
    println!("Оптимизация: NEON");

    // Информация об используемом аллокаторе
    #[cfg(feature = "jemalloc-allocator")]
    println!("Аллокатор: jemalloc");

    #[cfg(feature = "mimalloc-allocator")]
    println!("Аллокатор: mimalloc");

    #[cfg(feature = "system-allocator")]
    println!("Аллокатор: system (стандартный)");

    // Информация о Tokio
    println!(
        "Tokio runtime: {}",
        tokio::runtime::Handle::current().metrics().num_workers()
    );

    Ok(0)
}

/// Выполняет действие по умолчанию
async fn run_default() -> CoreResult<i32> {
    info!("Запуск действия по умолчанию");

    // Здесь можно реализовать логику по умолчанию
    println!("Библиотека core_lib успешно загружена и запущена.");
    println!("Используйте --help для получения справки по доступным командам.");

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        // Проверка разбора аргументов
        let args = vec![
            "program".to_string(),
            "--verbose".to_string(),
            "benchmark".to_string(),
            "--iterations".to_string(),
            "100".to_string(),
        ];

        let parsed = Args::try_parse_from(args).unwrap();
        assert!(parsed.verbose);

        match parsed.command {
            Some(Command::Benchmark { iterations, size }) => {
                assert_eq!(iterations, 100);
                assert_eq!(size, 1024); // значение по умолчанию
            },
            _ => panic!("Неправильный разбор команды"),
        }
    }
}
