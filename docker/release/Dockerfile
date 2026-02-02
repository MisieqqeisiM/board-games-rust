FROM rust:1.89

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-watch --version 8.5.3
RUN cargo install trunk --version 0.21.14
RUN cargo install wasm-bindgen-cli --version 0.2.108
RUN cargo install wasm-opt --version 0.116.1

# TODO: fix a specific version
RUN apt-get update && apt-get install -y esbuild

COPY . /usr/src/app

WORKDIR /usr/src/app/test_front
RUN trunk build --release

WORKDIR /usr/src/app/menu_front
RUN trunk build --release

WORKDIR /usr/src/app/backend
CMD ["cargo", "run", "--release"]