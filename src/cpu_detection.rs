//! Модуль для определения архитектуры процессора и поддерживаемых инструкций.
//!
//! Использует библиотеку raw-cpuid для получения информации о процессоре.

use crate::error::AppError;
use raw_cpuid::CpuId;
use std::env;

/// Структура с информацией о процессоре
#[derive(Debug)]
pub struct CpuInfo {
    /// Производитель процессора
    pub vendor: String,
    /// Модель процессора
    pub model: String,
    /// Поддерживаемые наборы инструкций
    pub features: Vec<String>,
}

/// Определяет архитектуру процессора и поддерживаемые инструкции
pub fn detect_cpu() -> Result<CpuInfo, AppError> {
    // Проверка переопределения через переменные окружения
    if let Ok(vendor) = env::var("CPU_VENDOR") {
        if let Ok(features) = env::var("CPU_FEATURES") {
            let features_vec = features.split(',').map(String::from).collect();
            return Ok(CpuInfo {
                vendor,
                model: env::var("CPU_MODEL").unwrap_or_else(|_| "Unknown".to_string()),
                features: features_vec,
            });
        }
    }

    // Определение архитектуры процессора
    match env::consts::ARCH {
        "x86_64" => detect_x86_64(),
        "aarch64" => detect_aarch64(),
        arch => Err(AppError::CpuDetectionError(format!(
            "Неподдерживаемая архитектура: {}",
            arch
        ))),
    }
}

/// Определяет информацию о процессоре x86_64
fn detect_x86_64() -> Result<CpuInfo, AppError> {
    let cpuid = CpuId::new();

    // Получение информации о производителе
    let vendor_info = cpuid.get_vendor_info().ok_or_else(|| {
        AppError::CpuDetectionError("Не удалось получить информацию о производителе".to_string())
    })?;

    // Получение информации о модели процессора
    let processor_info = cpuid.get_processor_brand_string().ok_or_else(|| {
        AppError::CpuDetectionError(
            "Не удалось получить информацию о модели процессора".to_string(),
        )
    })?;

    // Получение информации о поддерживаемых инструкциях
    let feature_info = cpuid.get_feature_info().ok_or_else(|| {
        AppError::CpuDetectionError(
            "Не удалось получить информацию о поддерживаемых инструкциях".to_string(),
        )
    })?;

    let extended_features = cpuid.get_extended_feature_info().ok_or_else(|| {
        AppError::CpuDetectionError(
            "Не удалось получить расширенную информацию о поддерживаемых инструкциях".to_string(),
        )
    })?;

    // Формирование списка поддерживаемых инструкций
    let mut features = Vec::new();

    // Проверка поддержки SSE4.2
    if feature_info.has_sse42() {
        features.push("sse4.2".to_string());
    }

    // Проверка поддержки AVX
    if feature_info.has_avx() {
        features.push("avx".to_string());
    }

    // Проверка поддержки AVX2
    if extended_features.has_avx2() {
        features.push("avx2".to_string());
    }

    Ok(CpuInfo {
        vendor: vendor_info.as_str().to_string(),
        model: processor_info.as_str().to_string(),
        features,
    })
}

/// Определяет информацию о процессоре aarch64
fn detect_aarch64() -> Result<CpuInfo, AppError> {
    // На ARM архитектуре нет прямого аналога CPUID
    // Можно использовать информацию из файловой системы или другие методы

    #[cfg(target_os = "linux")]
    {
        // Попытка получить информацию из /proc/cpuinfo
        match std::fs::read_to_string("/proc/cpuinfo") {
            Ok(cpuinfo) => {
                let mut vendor = String::from("ARM");
                let mut model = String::from("Unknown ARM Processor");

                // Парсинг информации о процессоре
                for line in cpuinfo.lines() {
                    if line.starts_with("Hardware") {
                        if let Some(hw) = line.split(':').nth(1) {
                            vendor = hw.trim().to_string();
                        }
                    } else if line.starts_with("model name") || line.starts_with("Processor") {
                        if let Some(mdl) = line.split(':').nth(1) {
                            model = mdl.trim().to_string();
                        }
                    }
                }

                // NEON обычно доступен на всех современных ARM процессорах
                return Ok(CpuInfo {
                    vendor,
                    model,
                    features: vec!["neon".to_string()],
                });
            },
            Err(_) => {
                // Если не удалось прочитать /proc/cpuinfo, используем базовую информацию
            },
        }
    }

    // Для других ОС или если не удалось получить информацию, возвращаем базовую информацию
    Ok(CpuInfo {
        vendor: "ARM".to_string(),
        model: "Unknown ARM Processor".to_string(),
        features: vec!["neon".to_string()], // NEON обычно доступен на всех современных ARM процессорах
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_override() {
        // Установка переменных окружения для теста
        std::env::set_var("CPU_VENDOR", "TestVendor");
        std::env::set_var("CPU_MODEL", "TestModel");
        std::env::set_var("CPU_FEATURES", "feature1,feature2,feature3");

        // Проверка, что функция использует переменные окружения
        let cpu_info = detect_cpu().unwrap();
        assert_eq!(cpu_info.vendor, "TestVendor");
        assert_eq!(cpu_info.model, "TestModel");
        assert_eq!(cpu_info.features, vec!["feature1", "feature2", "feature3"]);

        // Очистка переменных окружения
        std::env::remove_var("CPU_VENDOR");
        std::env::remove_var("CPU_MODEL");
        std::env::remove_var("CPU_FEATURES");
    }
}
