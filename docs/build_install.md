# Руководство по сборке и установке CPU Optimized App

## Содержание

1. [Требования](#требования)
2. [Установка зависимостей](#установка-зависимостей)
3. [Сборка из исходного кода](#сборка-из-исходного-кода)
4. [Сборка с использованием Makefile](#сборка-с-использованием-makefile)
5. [Сборка с использованием скрипта build.sh](#сборка-с-использованием-скрипта-buildsh)
6. [Сборка в Docker](#сборка-в-docker)
7. [Установка](#установка)
8. [Проверка установки](#проверка-установки)
9. [Устранение проблем при сборке](#устранение-проблем-при-сборке)

## Требования

Для сборки CPU Optimized App необходимы следующие компоненты:

### Обязательные требования

- **Rust** версии 1.70 или выше
- **Cargo** (поставляется вместе с Rust)
- **Git** для клонирования репозитория
- **Компилятор C/C++** (для сборки некоторых зависимостей)
  - GCC 7+ или Clang 6+ на Linux
  - MSVC на Windows
  - Xcode Command Line Tools на macOS

### Дополнительные требования

- **CMake** версии 3.10 или выше (для некоторых зависимостей)
- **pkg-config** на Linux
- **OpenSSL** (libssl-dev на Linux)
- **Make** для использования Makefile

## Установка зависимостей

### Linux (Debian/Ubuntu)

```bash
# Установка базовых инструментов разработки
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev cmake

# Установка Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Linux (Fedora/CentOS/RHEL)

```bash
# Установка базовых инструментов разработки
sudo dnf install -y gcc gcc-c++ make cmake openssl-devel pkgconfig

# Установка Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### macOS

```bash
# Установка Homebrew (если не установлен)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Установка зависимостей
brew install cmake openssl pkg-config

# Установка Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Windows

1. Установите [Visual Studio](https://visualstudio.microsoft.com/downloads/)
   с компонентами для разработки на C++
2. Установите [Rust](https://www.rust-lang.org/tools/install)
3. Установите [CMake](https://cmake.org/download/)
4. Установите [Git](https://git-scm.com/download/win)

## Сборка из исходного кода

### Клонирование репозитория

```bash
git clone https://github.com/yourusername/cpu_optimized_app.git
cd cpu_optimized_app
```

### Базовая сборка

```bash
# Сборка в режиме debug
cargo build

# Сборка в режиме release
cargo build --release
```

После успешной сборки исполняемый файл будет находиться в директории
`target/debug/` или `target/release/` в зависимости от выбранного
режима.

### Сборка библиотек

Для полноценной работы приложения необходимо собрать оптимизированные
библиотеки:

```bash
# Создание директории для библиотек
mkdir -p lib

# Сборка библиотеки для x86_64 с AVX2
cargo build --release -p core_lib --features="avx2,system-allocator"
cp target/release/libcore_lib.* lib/libx86_64_avx2_system.*

# Сборка библиотеки для x86_64 с AVX
cargo build --release -p core_lib --features="avx,system-allocator"
cp target/release/libcore_lib.* lib/libx86_64_avx_system.*

# Сборка библиотеки для x86_64 с SSE4.2
cargo build --release -p core_lib --features="sse4_2,system-allocator"
cp target/release/libcore_lib.* lib/libx86_64_sse4_2_system.*

# Сборка базовой библиотеки
cargo build --release -p core_lib
cp target/release/libcore_lib.* lib/libx86_64_base_system.*
```

## Сборка с использованием Makefile

Проект включает Makefile для упрощения процесса сборки:

```bash
# Сборка в режиме debug
make build

# Сборка в режиме release
make release

# Сборка всех вариантов библиотек
make libs

# Сборка конкретной библиотеки
make lib ARCH=x86_64 FEATURES=avx2 ALLOCATOR=system

# Полная сборка (приложение + все библиотеки)
make all
```

## Сборка с использованием скрипта build.sh

Скрипт `scripts/build.sh` предоставляет гибкие возможности для
сборки:

```bash
# Сделать скрипт исполняемым
chmod +x scripts/build.sh

# Сборка всех вариантов библиотек
./scripts/build.sh --all

# Сборка для конкретной архитектуры и набора инструкций
./scripts/build.sh --arch=x86_64 --features=avx2 --allocator=system

# Сборка в режиме debug
./scripts/build.sh --debug --all

# Сборка с очисткой предыдущих результатов
./scripts/build.sh --clean --all

# Получение справки по использованию скрипта
./scripts/build.sh --help
```

## Сборка в Docker

Проект включает Dockerfile для сборки в контейнере:

```bash
# Сборка Docker-образа
docker build -t cpu_optimized_app .

# Запуск контейнера
docker run cpu_optimized_app

# Запуск с передачей аргументов
docker run cpu_optimized_app benchmark --iterations=1000
```

## Установка

### Установка в систему

После сборки вы можете установить приложение в систему:

```bash
# Создание директории для установки
sudo mkdir -p /opt/cpu_optimized_app/lib

# Копирование исполняемого файла
sudo cp target/release/cpu_optimized_app /opt/cpu_optimized_app/

# Копирование библиотек
sudo cp lib/* /opt/cpu_optimized_app/lib/

# Создание символической ссылки
sudo ln -s /opt/cpu_optimized_app/cpu_optimized_app /usr/local/bin/cpu_optimized_app
```

### Создание пакета для дистрибутива

#### Debian/Ubuntu (DEB)

Для создания DEB-пакета можно использовать инструмент `cargo-deb`:

```bash
# Установка cargo-deb
cargo install cargo-deb

# Создание DEB-пакета
cargo deb
```

#### Fedora/CentOS/RHEL (RPM)

Для создания RPM-пакета можно использовать инструмент `cargo-rpm`:

```bash
# Установка cargo-rpm
cargo install cargo-rpm

# Создание RPM-пакета
cargo rpm build
```

## Проверка установки

После установки вы можете проверить работоспособность приложения:

```bash
# Проверка версии
cpu_optimized_app --version

# Вывод справки
cpu_optimized_app --help

# Вывод информации о системе
cpu_optimized_app info
```

## Устранение проблем при сборке

### Ошибки компиляции

#### Отсутствующие зависимости

Если при сборке возникают ошибки из-за отсутствующих зависимостей,
установите необходимые пакеты:

```bash
# Debian/Ubuntu
sudo apt-get install -y build-essential pkg-config libssl-dev cmake

# Fedora/CentOS/RHEL
sudo dnf install -y gcc gcc-c++ make cmake openssl-devel pkgconfig
```

#### Проблемы с Rust

Если возникают проблемы с Rust, попробуйте обновить его:

```bash
rustup update
```

### Ошибки при сборке библиотек

#### Неподдерживаемые наборы инструкций

Если ваш процессор не поддерживает определенные наборы инструкций
(например, AVX2), вы можете получить ошибку при запуске. В этом
случае соберите библиотеки только с поддерживаемыми наборами
инструкций:

```bash
# Проверка поддерживаемых наборов инструкций
./scripts/build.sh --help
```

#### Проблемы с аллокаторами

Если возникают проблемы с определенными аллокаторами, попробуйте
использовать стандартный системный аллокатор:

```bash
./scripts/build.sh --arch=x86_64 --features=avx2 --allocator=system
```

### Проблемы с Docker

Если возникают проблемы при сборке в Docker, убедитесь, что у вас
установлена последняя версия Docker и достаточно ресурсов для сборки:

```bash
# Проверка версии Docker
docker --version

# Сборка с увеличенными ресурсами
docker build --memory=4g --cpus=2 -t cpu_optimized_app .
```

### Общие рекомендации

- Убедитесь, что у вас достаточно свободного места на диске
- Проверьте наличие всех необходимых зависимостей
- Используйте последние версии инструментов
- При возникновении проблем с конкретным набором инструкций,
  попробуйте использовать более базовый набор
