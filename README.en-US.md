<div align="center">

<img src="docs/brand/rushwind-icon.svg" alt="RushWind CMS" width="128">

# RushWind CMS

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.81+-DEA584?logo=rust)](https://www.rust-lang.org/)
[![CI](https://github.com/tx7do/rushwind-cms/actions/workflows/ci.yml/badge.svg)](https://github.com/tx7do/rushwind-cms/actions/workflows/ci.yml)

**English** | [中文](./README.md) | [日本語](./README.ja-JP.md)

</div>

---

RushWind CMS is a **full-stack Headless content platform written in Rust**: built on the [rushwind](https://github.com/tx7do/rushwind) framework + [rust-utils](https://github.com/tx7do/rust-utils), with proto as the single API contract — five frontends (an admin console plus four site frontends) connect to the backend with zero modifications. Domain logic lives in core-service; the admin/app BFFs expose REST + SSE through generated code.

## Highlights

- **Contract-driven**: proto contracts are byte-verified by a MANIFEST gate; the build deterministically generates routing tables / binding plans / service traits / error-status tables / mount glue — zero hand-written routes
- **Three-service topology**: core-service (internal domain layer, gRPC :6602, sole owner of PostgreSQL/Redis, never exposed) + admin BFF (REST :6600 / SSE :6601) + app BFF (REST :6700 / SSE :6701); BFFs are thin proxies (script-generated pass-throughs), all domain logic stays in core
- **Dual contract surfaces**: a single contract crate yields both the REST surface (rushwind-gen-http: routes/bindings/traits/mounts) and the gRPC surface (tonic-prost: server/client for 38 domain services) — one type tree, zero duplication
- **Zero-modification frontends**: five frontend snapshots are kept in sync in-repo (sync scripts + RushWind brand overlay + dual-manifest check gate against hand edits); just point the API base URL at this repo's backends
- **Wire-protocol compatibility**: four-field error envelope (code/reason/message/metadata, protojson codec — 64-bit integers as strings, EmitUnpopulated), gorilla-compatible CORS, HS256 JWT auth gate, HttpOnly cookie sessions

## Quick Start

### Prerequisites

| Tool | Version |
|------|---------|
| Rust | stable (workspace `rust-version = 1.81`) |
| buf | latest (must be on PATH for `cargo build` — annotation-closure compilation, byte-exact options) |
| bash | for the sync scripts |
| PostgreSQL / Redis | runtime dependencies |
| Node.js + pnpm | per each frontend's `package.json` `engines` |

### Start the backend

```shell
cd backend
cargo run -p core-service # domain layer (internal): gRPC :6602 (start first — an empty DB is auto-bootstrapped with schema + seed + demo data)
cargo run -p admin-api   # admin BFF: REST :6600 + SSE :6601
cargo run -p app-api     # site BFF: REST :6700 + SSE :6701
```

- Configuration is embedded in the binaries (`services/*/assets/`: `data.yaml` / `auth.yaml` / `server.yaml`), overridable via environment variables: `RUSHWIND_DATABASE_SOURCE` / `RUSHWIND_REDIS_ADDR` / `RUSHWIND_REDIS_PASSWORD` / `jwt_signing_key`
- The key embedded in `auth.yaml` is a **development demo key**; production deployments must replace it via the `jwt_signing_key` environment variable
- Note: sea-orm 2.0 only accepts URL-form DSNs (`postgres://user:pass@host/db?sslmode=disable`)

### Contract sync

Proto contract snapshots are synced into this repo by a script, guarded by a MANIFEST check gate against hand edits:

```shell
bash backend/api/sync-protos.sh          # sync protos and rebuild the MANIFEST
bash backend/api/sync-protos.sh --check  # check gate (same as CI)
```

Source paths and override variables are documented in the script header. **Never hand-edit** `backend/api/protos/`.

### Frontend sync

```shell
bash frontend/admin/sync-frontend.sh admin-react           # admin console
bash frontend/app/sync-frontend.sh react                   # site React (Next.js)
bash frontend/app/sync-frontend.sh vue                     # site Vue (Nuxt)
bash frontend/app/sync-frontend.sh taro                    # site Taro (mini programs)
bash frontend/app/sync-frontend.sh flutter_app             # site Flutter
```

After syncing, the RushWind brand overlay is applied (logo / favicon / copy, see each `brand/README.md`), and the snapshot end-state is pinned by dual manifests (MANIFEST + UPSTREAM). **The brand overlay is the only permitted deviation from the source tree.**

### Start a frontend

| Frontend | Directory | Port | Backend |
|----------|-----------|------|---------|
| Admin React | `frontend/admin/admin-react` | 5999 | admin-api :6600 |
| Site React | `frontend/app/react` | 5001 | app-api :6700 |
| Site Vue | `frontend/app/vue` | — | app-api :6700 |
| Site Taro | `frontend/app/taro` | — | app-api :6700 |
| Site Flutter | `frontend/app/flutter_app` | — | app-api :6700 |

### Quality gates

```shell
cd backend
cargo fmt -p proto -p auth -p admin-api -p app-api -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI (see [.github/workflows/ci.yml](./.github/workflows/ci.yml)) runs the same gates: fmt / clippy / test / proto sync check / frontend snapshot check.

## Current Progress

The project advances in phases per [docs/development-plan.md](./docs/development-plan.md).

**Shipped**

- **Contract surface**: proto sync (MANIFEST gate) → buf annotation closure → prost/pbjson types + REST dual-BFF generated surfaces (42 admin / 11 app services) + tonic gRPC surface (default stubs, landing service by service)
- **Data layer**: golden DDL (id-column sequence defaults filled in) + system seed + demo data, auto-bootstrapped on an empty DB; sea-orm entities for 64 tables generated from the DDL by script; PagingRequest filter/sort/paging pipeline
- **core-service (internal gRPC :6602)**: auth kernel (login/register/logout/refresh rotation/ValidateToken, AES + bcrypt + permission gate + `gwc:` Redis key family + HS256 token pair) + landed domain services: dict×3 / post (with translations and category/tag relations) / category / tag / page / comment / interaction (counters) / site×4 / user / role / tenant (read)
- **BFF layer**: script-generated pass-through proxies (39 admin / 8 app) + hand-written auth surface (captcha/cookie) + manually mounted routes; bind pre-binding layer + HS256 auth gate (four-field envelope) + gorilla-compatible CORS + SSE
- **Frontends**: five snapshots synced + RushWind brand overlay + dual-manifest gates

**End-to-end test**: `bash backend/scripts/e2e-test.sh` (44 assertions green on the three-service stack — auth chain/rotation/revocation, real-data reads across 13 modules, CRUD writes, app registration/login and public reads, guest comments, SSE/CORS)

**In progress / planned**

- Interaction write paths (like/unlike/watch ledger), the five audit modules, RBAC tenant gate, menu/permission-point management, files/OSS, stats, internal_message, tasks
- Replay regression rig (request/response replay comparison, normalizing comparator + exemption set)

## Project Structure

```text
rushwind-cms/
├── backend/
│   ├── api/                        # API contracts (the single source of truth)
│   │   ├── protos/                 # proto contract copies (MANIFEST.sha256 gate)
│   │   ├── third_party/            # third-party protos (google.api etc., see PROVENANCE)
│   │   └── sync-protos.sh          # contract sync & check script
│   ├── crates/
│   │   ├── proto/                  # contract crate (prost/pbjson types + descriptor pool + REST/gRPC generated surfaces)
│   │   ├── store/                  # shared data layer (entity tree + paging pipeline + bootstrap + auth queries)
│   │   └── auth/                   # auth gate crate
│   └── services/
│       ├── core-service/           # domain layer (internal gRPC :6602, owns PG/Redis)
│       ├── admin-api/              # admin BFF (REST :6600 + SSE :6601, thin proxy)
│       └── app-api/                # site BFF (REST :6700 + SSE :6701, thin proxy)
├── frontend/
│   ├── admin/                      # admin snapshot (admin-react) + brand overlay + dual-manifest gate
│   └── app/                        # four site snapshots (react/vue/taro/flutter_app) + brand overlay + dual-manifest gate
├── docs/                           # project docs (development-plan …)
└── .github/workflows/              # CI (fmt / clippy / test / contract sync gate / snapshot gate)
```

## Related Projects

- **[rushwind](https://github.com/tx7do/rushwind)** — the RushWind framework monorepo (http-binding / gen-http / authn-jwt / bootstrap / transport-axum, …)
- **[rust-utils](https://github.com/tx7do/rust-utils)** — Rust utility libraries
- **[rushwind-admin](https://github.com/tx7do/rushwind-admin)** — sister project (a Rust admin platform on the same framework foundation)

## Contact

- WeChat: `yang_lin_bo` (please mention `rushwind-cms`)
