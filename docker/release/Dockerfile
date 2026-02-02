# ============================
# Builder
# ============================
FROM rust:1.89 AS builder

# ---- system deps ----
RUN printf "Package: esbuild\nPin: version 0.27.2*\nPin-Priority: 1001\n" \
    > /etc/apt/preferences.d/esbuild
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    esbuild \
    && rm -rf /var/lib/apt/lists/*

# ---- Rust + WASM tooling ----
RUN rustup target add wasm32-unknown-unknown

RUN cargo install \
    trunk@0.21.14 \
    wasm-bindgen-cli@0.2.108 \
    wasm-opt@0.116.1 \
    cargo-watch@8.5.3

WORKDIR /usr/src/app

# ============================
# Dependency caching
# ============================

COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/
COPY backend_commons/Cargo.toml backend_commons/
COPY frontend_commons/Cargo.toml frontend_commons/
COPY menu_back/Cargo.toml menu_back/
COPY menu_front/Cargo.toml menu_front/
COPY test_back/Cargo.toml test_back/
COPY test_front/Cargo.toml test_front/


# Fake minimal source tree
RUN find . -name src -type d -exec rm -rf {} +
RUN find . -name Cargo.toml -exec sh -c \
    'mkdir -p "$(dirname "$1")/src" && echo "fn main() { }" > "$(dirname "$1")/src/main.rs"' sh {} \;


RUN cargo build --release

# ============================
# Real source
# ============================
WORKDIR /usr/src/app
COPY . .

# ============================
# Build artifacts
# ============================

WORKDIR /usr/src/app/test_front
RUN trunk build --release

WORKDIR /usr/src/app/menu_front
RUN trunk build --release

WORKDIR /usr/src/app/backend
RUN cargo build --release

# ============================
# Runtime
# ============================
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/backend /app/backend/backend
COPY --from=builder /usr/src/app/menu_front/dist /app/menu_front/dist
COPY --from=builder /usr/src/app/test_front/dist /app/test_front/dist

USER nonroot
EXPOSE 8080
WORKDIR /app/backend
CMD ["./backend"]
