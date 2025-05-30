# Dockerfile для сборки проекта CPU Optimized App в контейнере
# Многоэтапная сборка для минимизации размера итогового образа

# Этап 1: Базовый образ для сборки
FROM rust:1.70-slim-bullseye as builder

# Установка необходимых зависимостей
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Создание пользователя для сборки
RUN useradd -m -u 1000 -U -s /bin/bash builder

# Создание директории для проекта
WORKDIR /app

# Копирование файлов проекта
COPY --chown=builder:builder . .

# Переключение на пользователя builder
USER builder

# Сборка проекта
RUN cargo build --release

# Сборка всех вариантов библиотек
RUN chmod +x scripts/build.sh && ./scripts/build.sh --all --release

# Этап 2: Минимальный образ для запуска
FROM debian:bullseye-slim

# Установка необходимых зависимостей для запуска
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Создание пользователя для запуска
RUN useradd -m -u 1000 -U -s /bin/bash appuser

# Создание директорий для приложения
WORKDIR /app
RUN mkdir -p /app/lib && chown -R appuser:appuser /app

# Копирование исполняемого файла и библиотек из этапа сборки
COPY --from=builder --chown=appuser:appuser /app/target/release/cpu_optimized_app /app/
COPY --from=builder --chown=appuser:appuser /app/lib/ /app/lib/

# Копирование документации
COPY --from=builder --chown=appuser:appuser /app/README.md /app/
COPY --from=builder --chown=appuser:appuser /app/docs/ /app/docs/

# Переключение на пользователя appuser
USER appuser

# Настройка переменных окружения
ENV RUST_LOG=info

# Проверка работоспособности
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD /app/cpu_optimized_app --version || exit 1

# Точка входа
ENTRYPOINT ["/app/cpu_optimized_app"]

# Аргументы по умолчанию
CMD ["--help"]
