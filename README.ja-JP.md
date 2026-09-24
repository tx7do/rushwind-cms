<div align="center">

<img src="docs/brand/rushwind-icon.svg" alt="RushWind CMS" width="128">

# RushWind CMS

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.81+-DEA584?logo=rust)](https://www.rust-lang.org/)
[![CI](https://github.com/tx7do/rushwind-cms/actions/workflows/ci.yml/badge.svg)](https://github.com/tx7do/rushwind-cms/actions/workflows/ci.yml)

[English](./README.en-US.md) | [中文](./README.md) | **日本語**

</div>

---

RushWind CMS は、[rushwind](https://github.com/tx7do/rushwind) フレームワーク + [rust-utils](https://github.com/tx7do/rust-utils) を土台とした **Rust 製フルスタック Headless コンテンツプラットフォーム**です。proto を唯一の API 契約とし、5 つのフロントエンド（管理コンソール + サイトフロントエンド 4 種）がゼロ改修でバックエンドに接続します。ドメインロジックは core-service に集約され、admin / app の 2 つの BFF が生成コードによって REST + SSE を公開します。

## ハイライト

- **契約駆動**：proto 契約は MANIFEST ゲートでバイト検証。ビルド時にルーティングテーブル / バインディングプラン / サービス trait / エラーステータス表 / マウントグルーを決定論的に生成し、手書きルートはゼロ
- **3 サービストポロジ**：core-service（内部ドメイン層、gRPC :6602、PostgreSQL/Redis を独占、外部非公開）+ admin BFF（REST :6600 / SSE :6601）+ app BFF（REST :6700 / SSE :6701）。BFF は薄いプロキシ（スクリプト生成の pass-through）、ドメインロジックはすべて core に集約
- **契約の両面**：同一の契約 crate から REST 面（rushwind-gen-http：ルート / バインディング / trait / mounts）と gRPC 面（tonic-prost：38 ドメインサービスの server/client）を生成。型は一ツリーで重複ゼロ
- **フロントエンドゼロ改修**：5 つのフロントエンドスナップショットをリポジトリ内で同期（同期スクリプト + RushWind ブランドオーバーレイ + 手改変防止の二重マニフェスト検証ゲート）。API ベース URL を本リポジトリのバックエンドに向けるだけ
- **ワイヤープロトコル互換**：4 フィールドエラーエンベロープ（code/reason/message/metadata、protojson コーデック — 64 ビット整数の文字列化、EmitUnpopulated）、gorilla 互換 CORS、HS256 JWT 認証ゲート、HttpOnly Cookie セッション

## クイックスタート

### 前提条件

| ツール | バージョン |
|--------|-----------|
| Rust | stable（workspace `rust-version = 1.81`） |
| buf | 最新版（`cargo build` 時に PATH 上に必要 — アノテーションクロージャのコンパイル、オプションのバイト忠実性） |
| bash | 同期スクリプトの実行 |
| PostgreSQL / Redis | 実行時依存 |
| Node.js + pnpm | 各フロントエンドの `package.json` の `engines` に従う |

### バックエンドの起動

```shell
cd backend
cargo run -p core-service # ドメイン層（内部）：gRPC :6602（先に起動 — 空 DB は schema + シード + デモデータを自動ブートストラップ）
cargo run -p admin-api   # 管理 BFF：REST :6600 + SSE :6601
cargo run -p app-api     # サイト BFF：REST :6700 + SSE :6701
```

- 設定はバイナリに埋め込み（`services/*/assets/`：`data.yaml` / `auth.yaml` / `server.yaml`）。環境変数で上書き：`RUSHWIND_DATABASE_SOURCE` / `RUSHWIND_REDIS_ADDR` / `RUSHWIND_REDIS_PASSWORD` / `jwt_signing_key`
- `auth.yaml` の埋め込みキーは**開発用デモキー**です。本番デプロイでは必ず `jwt_signing_key` 環境変数で置き換えてください
- 注意：sea-orm 2.0 の DSN は URL 形式のみ対応（`postgres://user:pass@host/db?sslmode=disable`）

### 契約同期

proto 契約スナップショットはスクリプトによって契約ソースから同期され、手改変防止の MANIFEST 検証ゲートで守られます:

```shell
bash backend/api/sync-protos.sh          # proto を同期して MANIFEST を再構築
bash backend/api/sync-protos.sh --check  # 検証ゲート（CI と同一）
```

同期元パスと上書き変数はスクリプトヘッダーを参照してください。`backend/api/protos/` は**手改変禁止**です。

### フロントエンド同期

```shell
bash frontend/admin/sync-frontend.sh admin-react           # 管理コンソール
bash frontend/app/sync-frontend.sh react                   # サイト React（Next.js）
bash frontend/app/sync-frontend.sh vue                     # サイト Vue（Nuxt）
bash frontend/app/sync-frontend.sh taro                    # サイト Taro（ミニプログラム）
bash frontend/app/sync-frontend.sh flutter_app             # サイト Flutter
```

同期後、RushWind ブランドオーバーレイを適用（logo / favicon / 文言、各 `brand/README.md` 参照）。二重マニフェスト（MANIFEST + UPSTREAM）でスナップショットの最終状態を固定します。**スナップショットに許されるソースツリーからの逸脱はブランドオーバーレイのみ**です。

### フロントエンドの起動

| フロントエンド | ディレクトリ | ポート | バックエンド |
|----------------|--------------|--------|--------------|
| 管理コンソール React | `frontend/admin/admin-react` | 5999 | admin-api :6600 |
| サイト React | `frontend/app/react` | 5001 | app-api :6700 |
| サイト Vue | `frontend/app/vue` | — | app-api :6700 |
| サイト Taro | `frontend/app/taro` | — | app-api :6700 |
| サイト Flutter | `frontend/app/flutter_app` | — | app-api :6700 |

### 品質ゲート

```shell
cd backend
cargo fmt -p proto -p auth -p admin-api -p app-api -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI（[.github/workflows/ci.yml](./.github/workflows/ci.yml) 参照）は同じゲートを実行します：fmt / clippy / test / proto 同期検証 / フロントエンドスナップショット検証。

## 現在の進捗

プロジェクトは [docs/development-plan.md](./docs/development-plan.md) のフェーズに沿って進行します。

**実装済み**

- **契約面**：proto 同期（MANIFEST ゲート）→ buf アノテーションクロージャ → prost/pbjson 型 + REST デュアル BFF 生成面（admin 42 / app 11 サービス）+ tonic gRPC 面（デフォルトスタブ、サービスごとに順次実装）
- **データ層**：ゴールデン DDL（id 列のシーケンスデフォルト補完）+ システムシード + デモデータ、空 DB を自動ブートストラップ。64 テーブルの sea-orm エンティティはスクリプトが DDL から生成。PagingRequest のフィルター / ソート / ページングパイプライン
- **core-service（内部 gRPC :6602）**：認証カーネル（ログイン / 登録 / ログアウト / リフレッシュローテーション / ValidateToken、AES + bcrypt + 権限ゲート + `gwc:` Redis キーファミリー + HS256 トークンペア）+ 実装済みドメインサービス：dict×3 / post（翻訳 + カテゴリー / タグ関連付き）/ category / tag / page / comment / interaction（カウント）/ site×4 / user / role / tenant（読み取り）
- **BFF 層**：スクリプト生成の pass-through プロキシ（admin 39 / app 8）+ 手書き認証面（captcha / Cookie）+ 手動登録ルート。bind 事前バインディング層 + HS256 認証ゲート（4 フィールドエンベロープ）+ gorilla 互換 CORS + SSE
- **フロントエンド**：5 スナップショットの同期 + RushWind ブランドオーバーレイ + 二重マニフェストゲート

**エンドツーエンドテスト**：`bash backend/scripts/e2e-test.sh`（3 サービススタックで 44 アサーションすべてグリーン — 認証チェーン / ローテーション / 失効、13 モジュールの実データ読み取り、CRUD 書き込み、app 登録・ログインと公開読み取り、ゲストコメント、SSE / CORS）

**進行中 / 計画**

- インタラクション書き込みパス（like/unlike/watch ledger）、監査 5 モジュール、RBAC テナントゲート、メニュー / 権限ポイント管理、ファイル / OSS、統計、internal_message、タスク
- リプレイ回帰リグ（リクエスト / レスポンスのリプレイ比較、正規化コンパレーター + 除外セット）

## プロジェクト構成

```text
rushwind-cms/
├── backend/
│   ├── api/                        # API 契約（唯一の契約ソース）
│   │   ├── protos/                 # proto 契約コピー（MANIFEST.sha256 検証ゲート）
│   │   ├── third_party/            # サードパーティ proto（google.api など、PROVENANCE 参照）
│   │   └── sync-protos.sh          # 契約同期・検証スクリプト
│   ├── crates/
│   │   ├── proto/                  # 契約 crate（prost/pbjson 型 + ディスクリプタプール + REST/gRPC 二重生成面）
│   │   ├── store/                  # 共有データ層（エンティティツリー + ページングパイプライン + ブートストラップ + 認証クエリ）
│   │   └── auth/                   # 認証ゲート crate
│   └── services/
│       ├── core-service/           # ドメイン層（内部 gRPC :6602、PG/Redis を独占）
│       ├── admin-api/              # 管理 BFF（REST :6600 + SSE :6601、薄いプロキシ）
│       └── app-api/                # サイト BFF（REST :6700 + SSE :6701、薄いプロキシ）
├── frontend/
│   ├── admin/                      # 管理コンソールスナップショット（admin-react）+ ブランドオーバーレイ + 二重マニフェストゲート
│   └── app/                        # サイト 4 スナップショット（react/vue/taro/flutter_app）+ ブランドオーバーレイ + 二重マニフェストゲート
├── docs/                           # プロジェクトドキュメント（development-plan …）
└── .github/workflows/              # CI（fmt / clippy / test / 契約同期ゲート / スナップショットゲート）
```

## 関連プロジェクト

- **[rushwind](https://github.com/tx7do/rushwind)** — RushWind フレームワーク monorepo（http-binding / gen-http / authn-jwt / bootstrap / transport-axum など）
- **[rust-utils](https://github.com/tx7do/rust-utils)** — Rust ユーティリティライブラリ
- **[rushwind-admin](https://github.com/tx7do/rushwind-admin)** — 姉妹プロジェクト（同一フレームワーク基盤の Rust 管理プラットフォーム）

## お問い合わせ

- WeChat 個人 ID：`yang_lin_bo`（備考：`rushwind-cms`）
