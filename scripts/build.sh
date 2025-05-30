#!/bin/bash
# Скрипт для гибкой сборки проекта
# Позволяет компилировать библиотеки по отдельности для разных архитектур, ОС и аллокаторов

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Функция для вывода справки
print_help() {
    echo -e "${BLUE}Скрипт сборки проекта CPU Optimized App${NC}"
    echo ""
    echo "Использование: $0 [опции]"
    echo ""
    echo "Опции:"
    echo "  --help                  Показать эту справку"
    echo "  --all                   Собрать все варианты библиотек"
    echo "  --release               Собрать в режиме release (по умолчанию)"
    echo "  --debug                 Собрать в режиме debug"
    echo "  --arch=ARCH             Архитектура (x86_64, aarch64)"
    echo "  --os=OS                 Операционная система (linux, windows, macos)"
    echo "  --features=FEATURES     Набор инструкций (avx2, avx, sse4_2, neon, base)"
    echo "  --allocator=ALLOCATOR   Аллокатор памяти (system, jemalloc, mimalloc)"
    echo "  --clean                 Очистить директорию сборки перед компиляцией"
    echo "  --verbose               Подробный вывод"
    echo ""
    echo "Примеры:"
    echo "  $0 --all                                # Собрать все варианты библиотек"
    echo "  $0 --arch=x86_64 --features=avx2        # Собрать только для x86_64 с AVX2"
    echo "  $0 --arch=aarch64 --allocator=jemalloc  # Собрать для ARM с jemalloc"
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
    
    # Проверка версии Rust
    local rust_version=$(rustc --version | cut -d ' ' -f 2)
    log "info" "Версия Rust: $rust_version"
    
    # Проверка наличия директории проекта
    if [ ! -f "Cargo.toml" ]; then
        log "error" "Файл Cargo.toml не найден. Запустите скрипт из корневой директории проекта."
        exit 1
    fi
    
    log "info" "Все необходимые инструменты найдены."
}

# Функция для сборки основного приложения
build_main_app() {
    local build_type=$1
    local verbose=$2
    
    log "info" "Сборка основного приложения в режиме $build_type..."
    
    local cargo_args="build"
    
    if [ "$build_type" == "release" ]; then
        cargo_args="$cargo_args --release"
    fi
    
    if [ "$verbose" == "true" ]; then
        cargo_args="$cargo_args -v"
    fi
    
    cargo $cargo_args
    
    log "info" "Основное приложение успешно собрано."
}

# Функция для сборки библиотеки с заданными параметрами
build_library() {
    local arch=$1
    local os=$2
    local features=$3
    local allocator=$4
    local build_type=$5
    local verbose=$6
    
    log "info" "Сборка библиотеки: arch=$arch, os=$os, features=$features, allocator=$allocator, build_type=$build_type"
    
    # Формирование аргументов для cargo
    local cargo_args="build -p core_lib"
    
    if [ "$build_type" == "release" ]; then
        cargo_args="$cargo_args --release"
    fi
    
    # Добавление целевой платформы
    local target=""
    case $arch in
        "x86_64")
            case $os in
                "linux")
                    target="x86_64-unknown-linux-gnu"
                    ;;
                "windows")
                    target="x86_64-pc-windows-msvc"
                    ;;
                "macos")
                    target="x86_64-apple-darwin"
                    ;;
            esac
            ;;
        "aarch64")
            case $os in
                "linux")
                    target="aarch64-unknown-linux-gnu"
                    ;;
                "windows")
                    target="aarch64-pc-windows-msvc"
                    ;;
                "macos")
                    target="aarch64-apple-darwin"
                    ;;
            esac
            ;;
    esac
    
    if [ -n "$target" ]; then
        cargo_args="$cargo_args --target $target"
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
    
    # Запуск сборки
    eval cargo $cargo_args
    
    # Копирование библиотеки в директорию lib
    copy_library "$arch" "$features" "$allocator" "$build_type" "$target"
    
    log "info" "Библиотека успешно собрана."
}

# Функция для копирования библиотеки в директорию lib
copy_library() {
    local arch=$1
    local features=$2
    local allocator=$3
    local build_type=$4
    local target=$5
    
    log "info" "Копирование библиотеки в директорию lib..."
    
    # Создание директории lib, если она не существует
    mkdir -p lib
    
    # Определение расширения библиотеки в зависимости от ОС
    local lib_ext=""
    case $target in
        *"windows"*)
            lib_ext="dll"
            ;;
        *"apple"*)
            lib_ext="dylib"
            ;;
        *)
            lib_ext="so"
            ;;
    esac
    
    # Определение префикса библиотеки в зависимости от ОС
    local lib_prefix=""
    case $target in
        *"windows"*)
            lib_prefix=""
            ;;
        *)
            lib_prefix="lib"
            ;;
    esac
    
    # Определение пути к собранной библиотеке
    local build_dir="target"
    if [ -n "$target" ]; then
        build_dir="$build_dir/$target"
    fi
    build_dir="$build_dir/$build_type"
    
    local src_lib="$build_dir/${lib_prefix}core_lib.$lib_ext"
    
    # Определение имени библиотеки
    local lib_name="${lib_prefix}${arch}_${features}_${allocator}.$lib_ext"
    local dst_lib="lib/$lib_name"
    
    # Копирование библиотеки
    cp "$src_lib" "$dst_lib"
    
    log "info" "Библиотека скопирована в $dst_lib"
}

# Функция для сборки всех вариантов библиотек
build_all() {
    local build_type=$1
    local verbose=$2
    
    log "info" "Сборка всех вариантов библиотек..."
    
    # Архитектуры
    local archs=("x86_64")
    # Операционные системы
    local oses=("linux" "windows" "macos")
    # Наборы инструкций
    local features_list=("avx2" "avx" "sse4_2" "base")
    # Аллокаторы
    local allocators=("system" "jemalloc" "mimalloc")
    
    # Определение текущей ОС
    local current_os=""
    case "$(uname)" in
        "Linux")
            current_os="linux"
            ;;
        "Darwin")
            current_os="macos"
            ;;
        "MINGW"*|"MSYS"*|"CYGWIN"*)
            current_os="windows"
            ;;
    esac
    
    # Определение текущей архитектуры
    local current_arch=""
    case "$(uname -m)" in
        "x86_64")
            current_arch="x86_64"
            ;;
        "aarch64"|"arm64")
            current_arch="aarch64"
            archs+=("aarch64")
            features_list=("neon" "base")
            ;;
    esac
    
    # Сборка только для текущей ОС
    oses=("$current_os")
    
    # Сборка библиотек
    for arch in "${archs[@]}"; do
        for os in "${oses[@]}"; do
            for features in "${features_list[@]}"; do
                for allocator in "${allocators[@]}"; do
                    build_library "$arch" "$os" "$features" "$allocator" "$build_type" "$verbose"
                done
            done
        done
    done
    
    log "info" "Все варианты библиотек успешно собраны."
}

# Основная функция
main() {
    # Параметры по умолчанию
    local build_all=false
    local build_type="release"
    local arch=""
    local os=""
    local features=""
    local allocator=""
    local clean=false
    local verbose=false
    
    # Разбор аргументов командной строки
    for arg in "$@"; do
        case $arg in
            --help)
                print_help
                exit 0
                ;;
            --all)
                build_all=true
                ;;
            --release)
                build_type="release"
                ;;
            --debug)
                build_type="debug"
                ;;
            --arch=*)
                arch="${arg#*=}"
                ;;
            --os=*)
                os="${arg#*=}"
                ;;
            --features=*)
                features="${arg#*=}"
                ;;
            --allocator=*)
                allocator="${arg#*=}"
                ;;
            --clean)
                clean=true
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
    
    # Очистка директории сборки, если указан флаг --clean
    if [ "$clean" == "true" ]; then
        log "info" "Очистка директории сборки..."
        cargo clean
        rm -rf lib
        mkdir -p lib
    fi
    
    # Сборка основного приложения
    build_main_app "$build_type" "$verbose"
    
    # Сборка библиотек
    if [ "$build_all" == "true" ]; then
        build_all "$build_type" "$verbose"
    elif [ -n "$arch" ] && [ -n "$features" ] && [ -n "$allocator" ]; then
        # Определение ОС, если не указана
        if [ -z "$os" ]; then
            case "$(uname)" in
                "Linux")
                    os="linux"
                    ;;
                "Darwin")
                    os="macos"
                    ;;
                "MINGW"*|"MSYS"*|"CYGWIN"*)
                    os="windows"
                    ;;
            esac
        fi
        
        build_library "$arch" "$os" "$features" "$allocator" "$build_type" "$verbose"
    else
        log "warn" "Не указаны параметры для сборки библиотеки. Используйте --all или укажите --arch, --features и --allocator."
    fi
    
    log "info" "Сборка завершена успешно."
}

# Запуск основной функции
main "$@"
