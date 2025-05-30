#!/bin/bash
# Скрипт для запуска бенчмарков и генерации отчетов
# Позволяет сравнивать производительность различных вариантов библиотек

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Функция для вывода справки
print_help() {
    echo -e "${BLUE}Скрипт запуска бенчмарков CPU Optimized App${NC}"
    echo ""
    echo "Использование: $0 [опции]"
    echo ""
    echo "Опции:"
    echo "  --help                  Показать эту справку"
    echo "  --all                   Запустить все бенчмарки"
    echo "  --bench=BENCH           Запустить конкретный бенчмарк (process_array, matrix_multiply, allocation)"
    echo "  --arch=ARCH             Архитектура (x86_64, aarch64)"
    echo "  --features=FEATURES     Набор инструкций (avx2, avx, sse4_2, neon, base)"
    echo "  --allocator=ALLOCATOR   Аллокатор памяти (system, jemalloc, mimalloc)"
    echo "  --compare               Сравнить результаты разных вариантов"
    echo "  --save=FILE             Сохранить результаты в файл"
    echo "  --verbose               Подробный вывод"
    echo ""
    echo "Примеры:"
    echo "  $0 --all                                # Запустить все бенчмарки"
    echo "  $0 --bench=process_array                # Запустить только бенчмарк process_array"
    echo "  $0 --compare --features=avx2,avx,base   # Сравнить разные наборы инструкций"
    echo ""
}

# Функция для вывода сообщений
log() {
    local level=$1
    local message=$2
    
    case $level in
        "info")
            echo -e "${GREEN}[INFO]${NC} $message"
            ;;
        "warn")
            echo -e "${YELLOW}[WARN]${NC} $message"
            ;;
        "error")
            echo -e "${RED}[ERROR]${NC} $message"
            ;;
        *)
            echo -e "${BLUE}[DEBUG]${NC} $message"
            ;;
    esac
}

# Функция для проверки наличия необходимых инструментов
check_requirements() {
    log "info" "Проверка наличия необходимых инструментов..."
    
    if ! command -v cargo &> /dev/null; then
        log "error" "Cargo не найден. Установите Rust и Cargo: https://rustup.rs/"
        exit 1
    fi
    
    if ! command -v rustc &> /dev/null; then
        log "error" "Rustc не найден. Установите Rust: https://rustup.rs/"
        exit 1
    fi
    
    # Проверка наличия директории проекта
    if [ ! -f "Cargo.toml" ]; then
        log "error" "Файл Cargo.toml не найден. Запустите скрипт из корневой директории проекта."
        exit 1
    fi
    
    # Проверка наличия директории бенчмарков
    if [ ! -d "core_lib/benches" ]; then
        log "error" "Директория бенчмарков не найдена. Убедитесь, что проект настроен правильно."
        exit 1
    fi
    
    log "info" "Все необходимые инструменты найдены."
}

# Функция для запуска бенчмарка
run_benchmark() {
    local bench=$1
    local arch=$2
    local features=$3
    local allocator=$4
    local verbose=$5
    local save_file=$6
    
    log "info" "Запуск бенчмарка: bench=$bench, arch=$arch, features=$features, allocator=$allocator"
    
    # Формирование аргументов для cargo
    local cargo_args="bench"
    
    # Добавление имени бенчмарка, если указано
    if [ -n "$bench" ] && [ "$bench" != "all" ]; then
        cargo_args="$cargo_args --bench performance_benchmarks -- $bench"
    fi
    
    # Добавление features
    local feature_args=""
    
    # Добавление аллокатора
    case $allocator in
        "jemalloc")
            feature_args="$feature_args jemalloc-allocator"
            ;;
        "mimalloc")
            feature_args="$feature_args mimalloc-allocator"
            ;;
        "system")
            feature_args="$feature_args system-allocator"
            ;;
    esac
    
    # Добавление набора инструкций
    case $features in
        "avx2")
            feature_args="$feature_args avx2"
            ;;
        "avx")
            feature_args="$feature_args avx"
            ;;
        "sse4_2")
            feature_args="$feature_args sse4_2"
            ;;
        "neon")
            feature_args="$feature_args neon"
            ;;
    esac
    
    if [ -n "$feature_args" ]; then
        cargo_args="$cargo_args --features=\"$feature_args\""
    fi
    
    if [ "$verbose" == "true" ]; then
        cargo_args="$cargo_args -v"
    fi
    
    # Создание директории для результатов
    mkdir -p "benchmark_results"
    
    # Формирование имени файла для сохранения результатов
    local result_file=""
    if [ -n "$save_file" ]; then
        result_file="$save_file"
    else
        result_file="benchmark_results/${bench}_${arch}_${features}_${allocator}.txt"
    fi
    
    # Запуск бенчмарка и сохранение результатов
    log "info" "Запуск команды: cargo $cargo_args"
    eval cargo $cargo_args | tee "$result_file"
    
    log "info" "Бенчмарк завершен. Результаты сохранены в $result_file"
    
    # Возвращаем путь к файлу с результатами
    echo "$result_file"
}

# Функция для сравнения результатов разных вариантов
compare_results() {
    local results=("$@")
    
    log "info" "Сравнение результатов бенчмарков..."
    
    # Проверка наличия результатов
    if [ ${#results[@]} -lt 2 ]; then
        log "error" "Для сравнения необходимо минимум два результата."
        return 1
    fi
    
    # Создание директории для отчетов
    mkdir -p "benchmark_reports"
    
    # Формирование имени файла отчета
    local report_file="benchmark_reports/comparison_$(date +%Y%m%d_%H%M%S).md"
    
    # Создание заголовка отчета
    echo "# Сравнение результатов бенчмарков" > "$report_file"
    echo "" >> "$report_file"
    echo "Дата: $(date)" >> "$report_file"
    echo "" >> "$report_file"
    
    # Добавление информации о системе
    echo "## Информация о системе" >> "$report_file"
    echo "" >> "$report_file"
    echo "- ОС: $(uname -s)" >> "$report_file"
    echo "- Версия ОС: $(uname -r)" >> "$report_file"
    echo "- Архитектура: $(uname -m)" >> "$report_file"
    echo "- Процессор: $(grep "model name" /proc/cpuinfo | head -n 1 | cut -d ':' -f 2 | sed 's/^[ \t]*//' 2>/dev/null || echo "Неизвестно")" >> "$report_file"
    echo "" >> "$report_file"
    
    # Добавление таблицы с результатами
    echo "## Результаты" >> "$report_file"
    echo "" >> "$report_file"
    echo "| Бенчмарк | Архитектура | Набор инструкций | Аллокатор | Время выполнения | Отношение к базовому |" >> "$report_file"
    echo "|----------|-------------|------------------|-----------|------------------|----------------------|" >> "$report_file"
    
    # Парсинг результатов и добавление в таблицу
    local base_time=""
    local base_result=""
    
    for result_file in "${results[@]}"; do
        # Извлечение информации из имени файла
        local filename=$(basename "$result_file")
        local bench=$(echo "$filename" | cut -d '_' -f 1)
        local arch=$(echo "$filename" | cut -d '_' -f 2)
        local features=$(echo "$filename" | cut -d '_' -f 3)
        local allocator=$(echo "$filename" | cut -d '_' -f 4 | cut -d '.' -f 1)
        
        # Извлечение времени выполнения из файла результатов
        local time=$(grep "time:" "$result_file" | head -n 1 | awk '{print $2}')
        
        # Если это первый результат, используем его как базовый
        if [ -z "$base_time" ]; then
            base_time="$time"
            base_result="$bench $arch $features $allocator"
            echo "| $bench | $arch | $features | $allocator | $time | 1.00 |" >> "$report_file"
        else
            # Вычисление отношения к базовому времени
            local ratio=$(echo "scale=2; $base_time / $time" | bc)
            echo "| $bench | $arch | $features | $allocator | $time | $ratio |" >> "$report_file"
        fi
    done
    
    echo "" >> "$report_file"
    echo "Базовый вариант: $base_result" >> "$report_file"
    
    log "info" "Сравнение завершено. Отчет сохранен в $report_file"
    
    # Вывод отчета
    cat "$report_file"
}

# Основная функция
main() {
    # Параметры по умолчанию
    local run_all=false
    local bench=""
    local arch=""
    local features=""
    local allocator=""
    local compare=false
    local save_file=""
    local verbose=false
    
    # Массив для хранения вариантов для сравнения
    local compare_features=()
    local compare_allocators=()
    
    # Разбор аргументов командной строки
    for arg in "$@"; do
        case $arg in
            --help)
                print_help
                exit 0
                ;;
            --all)
                run_all=true
                bench="all"
                ;;
            --bench=*)
                bench="${arg#*=}"
                ;;
            --arch=*)
                arch="${arg#*=}"
                ;;
            --features=*)
                features="${arg#*=}"
                # Проверка на список для сравнения
                if [[ "$features" == *","* ]]; then
                    IFS=',' read -ra compare_features <<< "$features"
                    features=""
                    compare=true
                fi
                ;;
            --allocator=*)
                allocator="${arg#*=}"
                # Проверка на список для сравнения
                if [[ "$allocator" == *","* ]]; then
                    IFS=',' read -ra compare_allocators <<< "$allocator"
                    allocator=""
                    compare=true
                fi
                ;;
            --compare)
                compare=true
                ;;
            --save=*)
                save_file="${arg#*=}"
                ;;
            --verbose)
                verbose=true
                ;;
            *)
                log "error" "Неизвестный аргумент: $arg"
                print_help
                exit 1
                ;;
        esac
    done
    
    # Проверка наличия необходимых инструментов
    check_requirements
    
    # Определение архитектуры, если не указана
    if [ -z "$arch" ]; then
        case "$(uname -m)" in
            "x86_64")
                arch="x86_64"
                ;;
            "aarch64"|"arm64")
                arch="aarch64"
                ;;
            *)
                log "error" "Неподдерживаемая архитектура: $(uname -m)"
                exit 1
                ;;
        esac
    fi
    
    # Массив для хранения путей к файлам с результатами
    local results=()
    
    # Запуск бенчмарков
    if [ "$compare" == "true" ]; then
        # Если указаны варианты для сравнения
        if [ ${#compare_features[@]} -gt 0 ]; then
            # Запуск бенчмарков для разных наборов инструкций
            for feature in "${compare_features[@]}"; do
                # Если аллокатор не указан, используем system
                local alloc="${allocator:-system}"
                local result_file=$(run_benchmark "$bench" "$arch" "$feature" "$alloc" "$verbose" "")
                results+=("$result_file")
            done
        elif [ ${#compare_allocators[@]} -gt 0 ]; then
            # Запуск бенчмарков для разных аллокаторов
            for alloc in "${compare_allocators[@]}"; do
                # Если набор инструкций не указан, определяем по архитектуре
                local feat=""
                if [ -z "$features" ]; then
                    if [ "$arch" == "x86_64" ]; then
                        feat="avx2"
                    else
                        feat="neon"
                    fi
                else
                    feat="$features"
                fi
                local result_file=$(run_benchmark "$bench" "$arch" "$feat" "$alloc" "$verbose" "")
                results+=("$result_file")
            done
        else
            # Запуск бенчмарков для базовых вариантов
            local base_features=()
            if [ "$arch" == "x86_64" ]; then
                base_features=("avx2" "avx" "sse4_2" "base")
            else
                base_features=("neon" "base")
            fi
            
            for feature in "${base_features[@]}"; do
                local result_file=$(run_benchmark "$bench" "$arch" "$feature" "system" "$verbose" "")
                results+=("$result_file")
            done
        fi
        
        # Сравнение результатов
        compare_results "${results[@]}"
    else
        # Запуск одного бенчмарка
        if [ "$run_all" == "true" ]; then
            # Запуск всех бенчмарков
            run_benchmark "all" "$arch" "${features:-avx2}" "${allocator:-system}" "$verbose" "$save_file"
        else
            # Запуск конкретного бенчмарка
            run_benchmark "${bench:-process_array}" "$arch" "${features:-avx2}" "${allocator:-system}" "$verbose" "$save_file"
        fi
    fi
    
    log "info" "Выполнение бенчмарков завершено."
}

# Запуск основной функции
main "$@"
