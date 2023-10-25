# 変数の指定
ARG RUST_VERSION=1.73.0

# 開発環境
FROM rust:${RUST_VERSION} AS development
WORKDIR /usr/src/app
RUN cargo install cargo-watch
VOLUME /usr/src/app

# ビルド環境
FROM rust:${RUST_VERSION}-slim-bullseye AS build
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release


FROM debian:bullseye-slim AS deployment

WORKDIR /usr/local/bin
COPY --from=build /usr/src/app/target/release/server ./
CMD ["./server"]

