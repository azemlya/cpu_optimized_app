//! Бенчмарки для измерения производительности различных вариантов библиотек.
//!
//! Использует крейт criterion для написания бенчмарков.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use std::time::Duration;

// Функция для тестирования производительности обработки массива данных
fn process_array(data: &[u8]) -> u64 {
    let mut sum = 0u64;

    // Используем разные оптимизации в зависимости от доступных инструкций
    #[cfg(feature = "avx2")]
    {
        // Оптимизированная версия с использованием AVX2
        // В реальном коде здесь был бы вызов функции с SIMD-инструкциями
        for chunk in data.chunks(32) {
            for &byte in chunk {
                sum = sum.wrapping_add(byte as u64);
            }
        }
    }

    #[cfg(all(feature = "avx", not(feature = "avx2")))]
    {
        // Оптимизированная версия с использованием AVX
        // В реальном коде здесь был бы вызов функции с SIMD-инструкциями
        for chunk in data.chunks(16) {
            for &byte in chunk {
                sum = sum.wrapping_add(byte as u64);
            }
        }
    }

    #[cfg(all(feature = "sse4_2", not(feature = "avx"), not(feature = "avx2")))]
    {
        // Оптимизированная версия с использованием SSE4.2
        // В реальном коде здесь был бы вызов функции с SIMD-инструкциями
        for chunk in data.chunks(8) {
            for &byte in chunk {
                sum = sum.wrapping_add(byte as u64);
            }
        }
    }

    #[cfg(not(any(feature = "avx2", feature = "avx", feature = "sse4_2")))]
    {
        // Базовая версия без оптимизаций
        for &byte in data {
            sum = sum.wrapping_add(byte as u64);
        }
    }

    sum
}

// Функция для тестирования производительности матричных операций
fn matrix_multiply(a: &[f32], b: &[f32], c: &mut [f32], n: usize) {
    assert_eq!(a.len(), n * n);
    assert_eq!(b.len(), n * n);
    assert_eq!(c.len(), n * n);

    // Используем разные оптимизации в зависимости от доступных инструкций
    #[cfg(feature = "avx2")]
    {
        // Оптимизированная версия с использованием AVX2
        // В реальном коде здесь был бы вызов функции с SIMD-инструкциями
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                for k in 0..n {
                    sum += a[i * n + k] * b[k * n + j];
                }
                c[i * n + j] = sum;
            }
        }
    }

    #[cfg(all(feature = "avx", not(feature = "avx2")))]
    {
        // Оптимизированная версия с использованием AVX
        // В реальном коде здесь был бы вызов функции с SIMD-инструкциями
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                for k in 0..n {
                    sum += a[i * n + k] * b[k * n + j];
                }
                c[i * n + j] = sum;
            }
        }
    }

    #[cfg(not(any(feature = "avx2", feature = "avx")))]
    {
        // Базовая версия без оптимизаций
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                for k in 0..n {
                    sum += a[i * n + k] * b[k * n + j];
                }
                c[i * n + j] = sum;
            }
        }
    }
}

// Функция для тестирования производительности аллокаций
fn allocation_benchmark(size: usize, iterations: usize) -> Vec<Vec<u8>> {
    let mut result = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let data = vec![0u8; size];
        result.push(data);
    }

    result
}

// Бенчмарк для обработки массива данных
fn bench_process_array(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_array");
    group.measurement_time(Duration::from_secs(10));

    // Тестирование на разных размерах данных
    for size in [1024, 4096, 16384, 65536].iter() {
        // Создание тестовых данных
        let data: Vec<u8> = (0..*size).map(|i| (i % 256) as u8).collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| process_array(black_box(&data)));
        });
    }

    group.finish();
}

// Бенчмарк для матричных операций
fn bench_matrix_multiply(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_multiply");
    group.measurement_time(Duration::from_secs(10));

    // Тестирование на разных размерах матриц
    for size in [16, 32, 64, 128].iter() {
        // Создание тестовых данных
        let n = *size;
        let a: Vec<f32> = (0..n * n).map(|i| (i % 100) as f32).collect();
        let b: Vec<f32> = (0..n * n).map(|i| ((i + 50) % 100) as f32).collect();
        let mut c = vec![0.0; n * n];

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bencher, _| {
            bencher.iter(|| {
                matrix_multiply(
                    black_box(&a),
                    black_box(&b),
                    black_box(&mut c),
                    black_box(n),
                )
            });
        });
    }

    group.finish();
}

// Бенчмарк для аллокаций
fn bench_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation");
    group.measurement_time(Duration::from_secs(10));

    // Тестирование на разных размерах аллокаций
    for size in [1024, 4096, 16384, 65536].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bencher, &size| {
            bencher.iter(|| allocation_benchmark(black_box(size), black_box(100)));
        });
    }

    group.finish();
}

// Определение групп бенчмарков
criterion_group!(
    benches,
    bench_process_array,
    bench_matrix_multiply,
    bench_allocation
);

// Запуск бенчмарков
criterion_main!(benches);
