# Docker + Rust 検証リポジトリ

Docker 環境での Rust 開発の環境構築の検証

## 環境

- WSL2にdockerをインストール
- 開発エディタは別のdockerコンテナ上のVSCode  
- ビルド先は Ubuntu の PC。GitHub からコードを DL して docker でビルド可能にする

## 1_minimum_setting

- docker のセッティングの最小構成
- docker-compose.yml の target で開発環境か本番環境を変更できる
- 本番環境と開発環境を同じ image でビルドしているので image サイズが大きい(1.5GB)
- 本番環境と開発環境の切り替えを docker-compose.yml で行うので異なるセッティングは dockerfile にすべて記載する必要がある

## 2_reduce_production_image

- 開発、ビルド、本番で仕様するイメージを変えて軽量化（85.2MB）
- ディレクトリ構成も使いまわせる形に変更
- １つのコンテナで１つのプログラムを開発するイメージ
- イメージは下記サイトをそのまま流用しているので余意味は分かってない
  [docker compose watch と rust との相性を確認してみる](https://zenn.dev/frusciante/articles/edbec9640f5a50)

## 3_verify_dev_image

- cargo や VSCode 周りを開発しやすい環境に調整
- 開発環境と本番環境の乖離が大きくなるので docker-compose.yml も整理
  - [Web アプリの本番環境と開発環境を同一の Docker Compose で管理する - シナプス技術者ブログ](https://tech.synapse.jp/entry/2023/06/15/183000)
  - 開発環境のコンテナ起動：`docker compose up -d`
  - 本番環境のコンテナ起動：`docker compose -f docker-compose.yml -f docker-compose.production.yml up -d`
- 以下の塩梅になるように docker compose を調整
  - VSCode はホストで稼働するものを使用。
  - デバッグはホスト、コンテナどちらでも可能。ただし環境は異なる
  - 生成ファイルは混じらない
- デバッグ、ビルドのパフォーマンスに課題が出たら他の方法を検討する
- VSCode を起動するのが rust プロジェクトのルートではないので rust-analyzer を動作させるため workspace の設定が必用
- API サーバーを想定しているので cargo watch も起動して入り

## 4_verify_hot_reload

- docker上でHTMLを返却するAPIを建てる
- devステージで建てるとホットリロードで開発できる
- localhost周りの設定がよく分からない。そのうち調べる必用がある
  - サーバー起動 IP を 0.0.0.0:3000 に変更
    - windows で起動。windows から接続
      - http://localhost:3000/：接続可能
      - http://0.0.0.0:3000/：接続不可
      - http://127.0.0.1:3000/：接続可能
    - WSL2 上の docker で起動(ポート設定は 8080:3000)。WSL2 から接続
      - curl localhost:8080：接続可能
      - curl 0.0.0.0:8080：接続可能
      - curl 127.0.0.1:8080：接続可能
    - WSL2 上の docker で起動(ポート設定は 8080:3000)。windows から接続
      - http://localhost:8080/：接続可能
      - http://0.0.0.0:8080/：接続不可
      - http://127.0.0.1:8080/：接続可能

## 5_verify_sql

- rust+docker+mysqlの検証
- [diesel cli を利用して migration を実施ためす #MySQL - Qiita](https://qiita.com/Gma_Gama/items/a489be2207f0b35f9282)を参考
- WSL2環境では生成ファイルのパーミッション問題が発生するのでコンテナのUID、GIDをWSL2用に設定
- マイグレーションまで問題なくできることを確認

## 6_devcontainer_setting

- docker composeとDevcontainer環境の両立方法について検証
- 開発で生成されるファイル、コンテナ内にインストールするファイルの権限の設定がうまくいかないため、WSL2をrootで運用   
- HotReloadでの開発を想定しているのでVSCodeを稼働するコンテナを別に建ている

