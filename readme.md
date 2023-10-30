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

## 3_verify_dev_image

- cargo や VSCode 周りを開発しやすい環境に調整
- 開発環境と本番環境の乖離が大きくなるので docker-compose.yml も整理
  - [Web アプリの本番環境と開発環境を同一の Docker Compose で管理する - シナプス技術者ブログ](https://tech.synapse.jp/entry/2023/06/15/183000)
  - 開発環境のコンテナ起動：`docker compose up -d`
  - 本番環境のコンテナ起動：`docker compose -f docker-compose.yml -f docker-compose.production.yml up -d`
- VSCode の DevContainer で開発は設定が冗長に感じる
  - [Dev Container on WSL2 で開発環境構築](https://zenn.dev/ykdev/articles/14a108290e24f9)
  - ホスト ⇒WSL2⇒docker container と経由が多い
- 以下の塩梅になるように docker compose を調整
  - VSCode はホストで稼働するものを使用。
  - デバッグはホスト、コンテナどちらでも可能。ただし環境は異なる
  - 生成ファイルは混じらない
- デバッグ、ビルドのパフォーマンスに課題が出たら他の方法を検討する
- VSCode を起動するのが rust プロジェクトのルートではないので rust-analyzer を動作させるため workspace の設定が必用
- API サーバーを想定しているので cargo watch も起動して入り

## 4_verify_hot_reload

- [WSL2 のポートへ localhost で接続する – 192.jp](https://192.jp/2020/08/23/connect-services-on-wsl-as-localhost/)
  この設定してもうまくいかなかった
- 接続できない原因は Rust 側のコードにあった
  - [Rust+Docker で Web サーバーにアクセスできない #Docker - Qiita](https://qiita.com/Sicut_study/items/dc1f232895c9264386df)
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
- localhost 周りの原理を勉強する必要がありそう
- Windows 上の編集ではコンテナないの HotReload はできない。コンテナが動作している WSL が変更を検知出来ていないから
  WSL でマウントしたボリュームでなく WSL 上のディレクトリで編集する必要がある
- UbuntuPC 上でコンテナを建てると HotReload 可能
- UbuntuPC の IP アドレスでアクセスできる

## 5_verify_sql

- SQL とサーバーが稼働するプログラムを別々に起動する
- [diesel cli を利用して migration を実施ためす #MySQL - Qiita](https://qiita.com/Gma_Gama/items/a489be2207f0b35f9282)
- マイグレーションはコンテナ構築後主導でおこなう感じ？
- image の種類によっては mysql-client の名前が違う
  [【Docker】「E: Package 'mysql-client' has no installation candidate」のエラー解決方法 #Docker - Qiita](https://qiita.com/Ryo-0131/items/df1b0072073cf80110e4)
- [Access denied for user 'root'@'172.18.0.3' (using password: YES) · Issue #486 · docker-library/mysql · GitHub](https://github.com/docker-library/mysql/issues/486)
-

## TODO

- 本番環境、開発環境の整理

- [Web アプリの本番環境と開発環境を同一の Docker Compose で管理する - シナプス技術者ブログ](https://tech.synapse.jp/entry/2023/06/15/183000)

- [API サーバを Rust で実装する　〜ローカル開発からデプロイまで〜 | OKAZAKI Shogo's Website](https://www.zakioka.net/blog/api-server-on-rust-develop-deploy)
