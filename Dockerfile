# Multi-stage build para perfil Kollontai (Rust + musl estatico + scratch)
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev ca-certificates

WORKDIR /app

# Cache de dependencias
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/hendu* target/release/hendu*

# Codigo-fonte completo
COPY src ./src
RUN cargo build --release && \
    strip target/release/hendu

# Container final sob perfil Kollontai
FROM scratch

# Certificados raiz para comunicacao HTTPS TLS com Gemini API
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Binario estatico
COPY --from=builder /app/target/release/hendu /hendu

# Diretorio para armazenamento persistente SQLite
VOLUME ["/data"]

USER 10001:10001

ENTRYPOINT ["/hendu"]
