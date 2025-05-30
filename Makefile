# Makefile для автоматизации процесса сборки проекта CPU Optimized App

# Переменные
CARGO := cargo
RUSTC := rustc
SCRIPTS_DIR := scripts
BUILD_SCRIPT := $(SCRIPTS_DIR)/build.sh
BENCHMARK_SCRIPT := $(SCRIPTS_DIR)/benchmark.sh
TARGET_DIR := target
LIB_DIR := lib
DOCS_DIR := docs

# Определение ОС
ifeq ($(OS),Windows_NT)
	DETECTED_OS := windows
	EXE_EXT := .exe
	RM := rmdir /s /q
	MKDIR := mkdir
else
	DETECTED_OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
	EXE_EXT :=
	RM := rm -rf
	MKDIR := mkdir -p
endif

# Определение архитектуры
ARCH := $(shell uname -m)
ifeq ($(ARCH),x86_64)
	DETECTED_ARCH := x86_64
	DEFAULT_FEATURES := avx2
else ifeq ($(ARCH),aarch64)
	DETECTED_ARCH := aarch64
	DEFAULT_FEATURES := neon
else
	DETECTED_ARCH := unknown
	DEFAULT_FEATURES := base
endif

# Цели
.PHONY: all build clean test bench docs release debug libs help

# Цель по умолчанию
all: build

# Сборка проекта
build:
	@echo "Сборка проекта..."
	$(CARGO) build

# Сборка в режиме release
release:
	@echo "Сборка проекта в режиме release..."
	$(CARGO) build --release

# Сборка в режиме debug
debug:
	@echo "Сборка проекта в режиме debug..."
	$(CARGO) build

# Сборка всех вариантов библиотек
libs:
	@echo "Сборка всех вариантов библиотек..."
	@if [ -f $(BUILD_SCRIPT) ]; then \
		chmod +x $(BUILD_SCRIPT) && $(BUILD_SCRIPT) --all; \
	else \
		echo "Скрипт сборки не найден: $(BUILD_SCRIPT)"; \
		exit 1; \
	fi

# Сборка библиотеки с указанными параметрами
lib:
	@echo "Сборка библиотеки с указанными параметрами..."
	@if [ -f $(BUILD_SCRIPT) ]; then \
		chmod +x $(BUILD_SCRIPT) && $(BUILD_SCRIPT) \
			--arch=$(ARCH) \
			--features=$(FEATURES) \
			--allocator=$(ALLOCATOR); \
	else \
		echo "Скрипт сборки не найден: $(BUILD_SCRIPT)"; \
		exit 1; \
	fi

# Запуск тестов
test:
	@echo "Запуск тестов..."
	$(CARGO) test

# Запуск бенчмарков
bench:
	@echo "Запуск бенчмарков..."
	@if [ -f $(BENCHMARK_SCRIPT) ]; then \
		chmod +x $(BENCHMARK_SCRIPT) && $(BENCHMARK_SCRIPT) --all; \
	else \
		echo "Скрипт бенчмарков не найден: $(BENCHMARK_SCRIPT)"; \
		exit 1; \
	fi

# Сравнение производительности разных вариантов
bench-compare:
	@echo "Сравнение производительности разных вариантов..."
	@if [ -f $(BENCHMARK_SCRIPT) ]; then \
		chmod +x $(BENCHMARK_SCRIPT) && $(BENCHMARK_SCRIPT) --compare; \
	else \
		echo "Скрипт бенчмарков не найден: $(BENCHMARK_SCRIPT)"; \
		exit 1; \
	fi

# Генерация документации
docs:
	@echo "Генерация документации..."
	$(CARGO) doc --no-deps
	@echo "Документация сгенерирована в $(TARGET_DIR)/doc"

# Очистка проекта
clean:
	@echo "Очистка проекта..."
	$(CARGO) clean
	@if [ -d $(LIB_DIR) ]; then $(RM) $(LIB_DIR); fi
	@$(MKDIR) $(LIB_DIR)

# Полная очистка (включая сгенерированную документацию)
clean-all: clean
	@echo "Полная очистка проекта..."
	@if [ -d $(TARGET_DIR)/doc ]; then $(RM) $(TARGET_DIR)/doc; fi

# Проверка стиля кода
check:
	@echo "Проверка стиля кода..."
	$(CARGO) fmt -- --check
	$(CARGO) clippy -- -D warnings

# Форматирование кода
fmt:
	@echo "Форматирование кода..."
	$(CARGO) fmt

# Вывод справки
help:
	@echo "Makefile для проекта CPU Optimized App"
	@echo ""
	@echo "Доступные цели:"
	@echo "  all            - Сборка проекта (по умолчанию)"
	@echo "  build          - Сборка проекта"
	@echo "  release        - Сборка проекта в режиме release"
	@echo "  debug          - Сборка проекта в режиме debug"
	@echo "  libs           - Сборка всех вариантов библиотек"
	@echo "  lib            - Сборка библиотеки с указанными параметрами"
	@echo "                   ARCH=x86_64|aarch64 FEATURES=avx2|avx|sse4_2|neon|base ALLOCATOR=system|jemalloc|mimalloc"
	@echo "  test           - Запуск тестов"
	@echo "  bench          - Запуск бенчмарков"
	@echo "  bench-compare  - Сравнение производительности разных вариантов"
	@echo "  docs           - Генерация документации"
	@echo "  clean          - Очистка проекта"
	@echo "  clean-all      - Полная очистка проекта"
	@echo "  check          - Проверка стиля кода"
	@echo "  fmt            - Форматирование кода"
	@echo "  help           - Вывод этой справки"
	@echo ""
	@echo "Примеры использования:"
	@echo "  make lib ARCH=x86_64 FEATURES=avx2 ALLOCATOR=system"
	@echo "  make bench-compare"
