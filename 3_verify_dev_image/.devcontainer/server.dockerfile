# 変数の指定
ARG RUST_VERSION=1.73.0
ARG APP_NAME=server

# 開発環境
FROM rust:${RUST_VERSION} AS development
WORKDIR /usr/src/app
RUN cargo install cargo-watch

# ビルド環境
FROM rust:${RUST_VERSION}-slim-bullseye AS build
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# 本番環境
FROM debian:bullseye-slim AS production
WORKDIR /usr/local/bin
COPY --from=build /usr/src/app/target/release/${APP_NAME} ./
CMD ["./server"]

