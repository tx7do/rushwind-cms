<div align="center">

<img src="docs/brand/rushwind-icon.svg" alt="RushWind CMS" width="128">

# RushWind CMS

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.81+-DEA584?logo=rust)](https://www.rust-lang.org/)
[![CI](https://github.com/tx7do/rushwind-cms/actions/workflows/ci.yml/badge.svg)](https://github.com/tx7do/rushwind-cms/actions/workflows/ci.yml)

**中文**

</div>

---

RushWind CMS 是 [go-wind-cms](https://github.com/tx7do/go-wind-cms)（GoWind Content Hub，Go + Kratos 全栈 Headless 内容平台）的 **Rust 复刻**：以 [rushwind](https://github.com/tx7do/rushwind) 框架 + [rust-utils](https://github.com/tx7do/rust-utils) 为底座，proto 为唯一 API 契约，五个前端（管理后台 + 四套前台）零改动对接两个后端。移植模式与姊妹项目 [rushwind-admin](https://github.com/tx7do/rushwind-admin) 同源。

## 项目亮点

- **契约驱动**：proto 契约字节同步自查（MANIFEST 校验门），构建期确定性生成路由表 / 绑定计划 / 服务 trait / 错误状态表 / 挂载胶水，零手写路由
- **三服务对位**：core-service（对内领域层，gRPC :6602，独占 PostgreSQL/Redis，不对外暴露）+ admin BFF（REST :6600 / SSE :6601）+ app BFF（REST :6700 / SSE :6701）——对位 go-wind-cms 的 admin/app/core 三服务拓扑；BFF 是薄代理（脚本生成的 pass-through），领域逻辑全在 core
- **契约双面**：同一契约 crate 同时产出 REST 面（rushwind-gen-http：路由/绑定/trait/mounts）与 gRPC 面（tonic-prost：38 个领域服务的 server/client），类型一树零重复
- **前端零改动**：五个前端快照随仓同步（同步脚本 + RushWind 品牌覆写层 + 双清单校验门防手改），API 基址指向本仓后端即可
- **wire 对位**：Kratos 风格四字段错误信封（code/reason/message/metadata）、protojson 编解码（64 位整数字符串化、EmitUnpopulated）、gorilla 兼容 CORS、HS256 JWT 鉴权门、HttpOnly Cookie 会话（登录实现阶段接入）

## 快速开始

### 环境要求

| 工具 | 版本 |
|------|------|
| Rust | stable（workspace `rust-version = 1.81`） |
| buf | 最新版（`cargo build` 时需在 PATH——注解闭包编译，选项字节保真） |
| bash | 运行同步脚本 |
| PostgreSQL / Redis | 运行时依赖 |
| Node.js + pnpm | 以各前端 `package.json` 的 `engines` 为准 |

### 后端启动

```shell
cd backend
cargo run -p core-service # 领域层（对内）：gRPC :6602（先起——空库自动引导 schema+种子+演示数据）
cargo run -p admin-api   # 管理 BFF：REST :6600 + SSE :6601
cargo run -p app-api     # 前台 BFF：REST :6700 + SSE :6701
```

- 配置内嵌于二进制（`services/*/assets/`：`data.yaml` / `auth.yaml` / `server.yaml`），环境变量覆盖：`RUSHWIND_DATABASE_SOURCE` / `RUSHWIND_REDIS_ADDR` / `RUSHWIND_REDIS_PASSWORD` / `jwt_signing_key`（与 Go 侧 `${jwt_signing_key:...}` 占位同名）
- `auth.yaml` 内嵌密钥为**开发演示密钥**，生产部署必须通过 `jwt_signing_key` 环境变量更换
- 注意：sea-orm 2.0 的 DSN 只认 URL 形式（`postgres://user:pass@host/db?sslmode=disable`）

### 契约同步

proto 契约由脚本从上游契约源同步进本仓，并加 MANIFEST 校验门防止手改：

```shell
bash backend/api/sync-protos.sh          # 同步 proto 并重建 MANIFEST
bash backend/api/sync-protos.sh --check  # 校验门（与 CI 一致）
```

同步源默认 `GOWIND_CMS_API_DIR`（见脚本头部说明）。**不要手改** `backend/api/protos/`。

### 前端同步

```shell
bash frontend/admin/sync-frontend.sh admin-react           # 管理后台
bash frontend/app/sync-frontend.sh react                   # 前台 React（Next.js）
bash frontend/app/sync-frontend.sh vue                     # 前台 Vue（Nuxt）
bash frontend/app/sync-frontend.sh taro                    # 前台 Taro（小程序）
bash frontend/app/sync-frontend.sh flutter_app             # 前台 Flutter
```

同步后打 RushWind 品牌覆写（logo / favicon / 文案，见各 `brand/README.md`），并以双清单（MANIFEST + UPSTREAM）钉死快照终态。**快照唯一被允许的对上游偏离就是品牌覆写**。

### 前端启动

| 前端 | 目录 | 端口 | 后端 |
|------|------|------|------|
| 管理后台 React | `frontend/admin/admin-react` | 5999 | admin-api :6600 |
| 前台 React | `frontend/app/react` | 5001 | app-api :6700 |
| 前台 Vue | `frontend/app/vue` | — | app-api :6700 |
| 前台 Taro | `frontend/app/taro` | — | app-api :6700 |
| 前台 Flutter | `frontend/app/flutter_app` | — | app-api :6700 |

### 质量门

```shell
cd backend
cargo fmt -p proto -p auth -p admin-api -p app-api -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI（见 [.github/workflows/ci.yml](./.github/workflows/ci.yml)）执行同样的门：fmt / clippy / test / proto 同步校验 / 前端快照校验。

## 当前进度

项目按 [docs/development-plan.md](./docs/development-plan.md) 的阶段推进。

**已落地**

- **契约面**：proto 同步（MANIFEST 门）→ buf 注解闭包 → prost/pbjson 类型 + REST 双 BFF 生成面（admin 42 / app 11 服务）+ tonic gRPC 面（默认桩，逐服务落地）
- **数据层**：黄金 DDL（ent 迁移导出，id 列序列默认补齐）+ 系统种子 + 演示数据，空库自动引导；64 张表的 sea-orm entity 由脚本从 DDL 生成；PagingRequest 过滤/排序/分页管线
- **core-service（对内 gRPC :6602）**：认证内核（登录/注册/登出/刷新轮换/ValidateToken，AES+bcrypt+权限门+`gwc:` Redis 键族+HS256 令牌对）+ 已落地领域服务：dict×3 / post（含翻译+分类/标签关联）/ category / tag / page / comment / interaction（计数）/ site×4 / user / role / tenant（读）
- **BFF 层**：脚本生成的 pass-through 代理（admin 39 / app 8）+ 手写认证面（验证码/Cookie）+ 手工注册路由；bind 预绑定层 + HS256 鉴权门（四字段信封）+ gorilla 兼容 CORS + SSE
- **前端**：五个快照同步 + RushWind 品牌覆写 + 双清单门

**端到端测试**：`bash backend/scripts/e2e-test.sh`（三服务栈 44 项断言全绿——认证链/轮换/吊销、13 个模块真实数据读、CRUD 写、app 注册登录与公开读、游客评论、SSE/CORS）

**进行中 / 规划**

- 互动写路径（like/unlike/watch ledger）、审计五件套、RBAC 租户门、菜单/权限点管理、文件/OSS、统计、internal_message、任务
- 差分回归台架（Go/Rust 双后端回放比对）

## 项目结构

```text
rushwind-cms/
├── backend/
│   ├── api/                        # API 契约（唯一契约源）
│   │   ├── protos/                 # proto 契约副本（MANIFEST.sha256 校验门）
│   │   ├── third_party/            # 第三方 proto（google.api 等，PROVENANCE）
│   │   └── sync-protos.sh          # 契约同步与校验脚本
│   ├── crates/
│   │   ├── proto/                  # 契约 crate（prost/pbjson 类型 + 描述符池 + REST/gRPC 双生成面）
│   │   ├── store/                  # 共享数据层（entity 树 + 分页管线 + 引导 + 认证查询）
│   │   └── auth/                   # 鉴权门 crate
│   └── services/
│       ├── core-service/           # 领域层（对内 gRPC :6602，独占 PG/Redis）
│       ├── admin-api/              # 管理 BFF（REST :6600 + SSE :6601，薄代理）
│       └── app-api/                # 前台 BFF（REST :6700 + SSE :6701，薄代理）
├── frontend/
│   ├── admin/                      # 管理后台快照（admin-react）+ 品牌覆写 + 双清单门
│   └── app/                        # 四套前台快照（react/vue/taro/flutter_app）+ 品牌覆写 + 双清单门
├── docs/                           # 项目文档（development-plan …）
└── .github/workflows/              # CI（fmt / clippy / test / 契约同步门 / 快照门）
```

## 相关项目

- **[go-wind-cms](https://github.com/tx7do/go-wind-cms)** —— 上游 Go 实现（GoWind Content Hub）
- **[rushwind](https://github.com/tx7do/rushwind)** —— RushWind 框架 monorepo（http-binding / gen-http / authn-jwt / bootstrap / transport-axum 等）
- **[rust-utils](https://github.com/tx7do/rust-utils)** —— Rust 工具库
- **[rushwind-admin](https://github.com/tx7do/rushwind-admin)** —— 姊妹项目（go-wind-admin 的 Rust 复刻，移植模式同源）

## 联系我们

- 微信个人号：`yang_lin_bo`（备注：`rushwind-cms`）
