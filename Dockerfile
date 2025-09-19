# =========================
# Builder stage
# =========================
FROM rust:1.87 AS builder
WORKDIR /app

# evita conexão com o banco durante o build
ENV SQLX_OFFLINE=true

# 1. Copia manifesto do projeto para build de dependências em cache
COPY Cargo.toml Cargo.lock ./
COPY .sqlx .sqlx

# cria dummy src para não quebrar o build
RUN mkdir src && echo "fn main() {}" > src/main.rs

# compila dependências (cacheadas)
RUN cargo build --release --locked || true

# 2. Copia o resto do código
COPY . .

# recompila agora com o código real
RUN cargo build --release --locked

# =========================
# Runtime stage
# =========================
FROM debian:bullseye-slim AS runtime
ARG APP=/usr/src/app

# instalar libs necessárias
RUN apt-get update && apt-get install -y \
    gcc \
    libffi-dev \
    libpq-dev \
    make \
    iputils-ping \
    tzdata \
    && ln -fs /usr/share/zoneinfo/America/Sao_Paulo /etc/localtime \
    && dpkg-reconfigure --frontend noninteractive tzdata \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

# copia apenas o binário final
COPY --from=builder /app/target/release/app_rust ${APP}/app_rust

RUN chown -R $APP_USER:$APP_USER ${APP}
USER $APP_USER
WORKDIR ${APP}

EXPOSE 8008
ENTRYPOINT ["./app_rust"]
