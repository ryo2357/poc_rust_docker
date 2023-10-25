# Docker + Rust 検証リポジトリ

Docker 環境での Rust 開発の環境構築の検証

## 想定している環境

- 開発は Windows の wsl 環境
- 開発エディタはホスト Windows の VSCode を想定
- ビルド先は Ubuntu の PC。GitHub からコードを DL して docker でビルド可能にする

## 1_minimum_setting

- docker のセッティングの最小構成
- docker-compose.yml の target で開発環境か本番環境を変更できる

### 課題・気になった点

- 本番環境と開発環境を同じ image でビルドしているので image サイズが大きい(1.5GB)
- 本番環境と開発環境の切り替えを docker-compose.yml で行うので異なるセッティングは dockerfile にすべて記載する必要がある

## 2_reduce_production_image

- 開発、ビルド、本番で仕様するイメージを変えて軽量化（85.2MB）
- ディレクトリ構成も使いまわせる形に変更
- １つのコンテナで１つのプログラムを開発するイメージ
- イメージは下記サイトをそのまま流用しているので余意味は分かってない

  [docker compose watch と rust との相性を確認してみる](https://zenn.dev/frusciante/articles/edbec9640f5a50)
