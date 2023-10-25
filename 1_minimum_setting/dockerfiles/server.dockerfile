# 変数の指定
ARG RUST_VERSION=1.73.0
ARG APP_NAME=server

# ベースイメージを指定
FROM rust:1.73 AS base
WORKDIR /usr/src/app/server

# 開発環境
FROM base AS development
RUN cargo install cargo-watch
VOLUME /usr/src/app/server

# ビルド環境
FROM base AS build
COPY . .
RUN cargo build --release

# 本番環境

#  version `GLIBC_2.33' not found (required by ./server)
# FROM rust:1.73-slim-buster AS deployment
# imageサイズが1.5GB
FROM base AS deployment

WORKDIR /usr/local/bin
COPY --from=build /usr/src/app/server/target/release/server ./
CMD ["./server"]

# 103MBだがversion `GLIBC_2.33' not found 
FROM debian:bullseye-slim AS debian

WORKDIR /usr/local/bin
RUN apt-get update && apt-get install -y libgcc1 libstdc++6 bash

COPY --from=build /usr/src/app/server/target/release/server ./

# 
FROM debian:buster-slim AS debian-2

RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin
COPY --from=build /usr/src/app/server/target/release/server ./
CMD ["./server"]

# [docker compose watchとrustとの相性を確認してみる](https://zenn.dev/frusciante/articles/edbec9640f5a50)





