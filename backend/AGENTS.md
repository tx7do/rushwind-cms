# AGENTS.md — 后端开发指南

> 本文件是 backend 子项目的 AI 编码规范单一事实源，适用于所有支持 AGENTS.md 的 AI 编码工具（ZCode、GitHub Copilot、Cursor、Codex、Gemini CLI 等）。Claude Code 通过同级 `CLAUDE.md` 中的 `@AGENTS.md` 引用加载。monorepo 全局约定见 [../AGENTS.md](../AGENTS.md)。

## 项目概览

契约驱动的 Rust 微服务后端：proto 是唯一 API 契约，构建期从契约确定性生成路由表 / 绑定计划 / 服务 trait / 挂载胶水，零手写路由。

**核心技术栈**：

| 项 | 选型 |
|---|---|
| 语言 | Rust（edition 2021，`rust-version = 1.81`，stable 工具链） |
| 框架底座 | [rushwind](https://github.com/tx7do/rushwind)（http-binding / gen-http / bootstrap / transport-axum / authn-jwt / proto-build 等，git rev 钉定） |
| 工具库 | [rust-utils](https://github.com/tx7do/rust-utils)（git rev 钉定） |
| gRPC | tonic（服务面由契约构建管线的 tonic 生成器产出，默认桩） |
| 序列化 | prost 0.14 / pbjson 0.9（protojson 兼容：64 位整数以字符串编解码、EmitUnpopulated） |
| ORM | sea-orm 2.0（PostgreSQL，sqlx-postgres + rustls；DSN 只认 URL 形式） |
| 缓存 | redis 1（connection-manager + script） |
| 契约编译 | buf（`build.rs` 依赖其在 PATH；选项字节保真） |
| 异步运行时 | tokio（multi-thread） |

修改与 rushwind / rust-utils 集成点前，先理解框架封装（git 依赖，源码在 `~/.cargo/git/checkouts` 或本机 `~/Github/rushwind`）。

## 三服务架构（关键心智模型）

```
HTTP/REST + SSE
  ├─ admin-api   管理 BFF（REST :6600 / SSE :6601）──gRPC──┐
  └─ app-api     前台 BFF（REST :6700 / SSE :6701）──gRPC──┤
                                                          ▼
                                              core-service（gRPC :6602）
                                                ├─ sea-orm → PostgreSQL
                                                ├─ redis（会话/验证码/缓存）
                                                └─ 对象存储（MinIO / 本地盘）
```

| 服务 | 角色 | 是否访问 DB |
|------|------|:---:|
| `services/admin-api` | 管理网关 BFF（接收 REST，转发 gRPC 到 core） | ❌ |
| `services/app-api` | 前台网关 BFF | ❌ |
| `services/core-service` | 核心业务实现（持久化、真实逻辑） | ✅ |

> **BFF 是"瘦"网关**：`proxies.rs` 是生成器产出的 pass-through 代理表——BFF trait 方法与领域 gRPC 方法同名同型（BFF proto 直接 import 领域消息），转发零映射。**所有数据库操作只能在 core-service 内进行。**

## 目录结构

```
backend/
├── api/                          # 契约层（无业务逻辑）
│   ├── protos/                   # ★ proto 快照（字节冻结，MANIFEST.sha256 校验门，禁止手改）
│   │   └── <domain>/service/v1/  # 29 个领域：admin/app/identity/permission/content/audit/ai/notify/flow/...
│   ├── buf.yaml / buf.lock       # buf 工作区 + 六个 BSR 依赖的 commit 钉定
│   ├── MANIFEST.sha256           # 契约清单门（防手改）
│   └── sync-protos.sh            # 契约同步 / --check 校验门
├── crates/
│   ├── proto/                    # 契约 crate
│   │   ├── build.rs              # ★ 构建管线入口（配置面；引擎在 rushwind-proto-build）
│   │   └── src/auth_free.rs      # 免鉴权端点白名单（构建期消费）
│   ├── store/                    # 共享数据层
│   │   └── src/
│   │       ├── entities.rs       # ★ entity 树（gen-entities.py 产物，一表一 mod，禁止手改）
│   │       ├── paging.rs         # PagingRequest 过滤/排序/分页 select 装配
│   │       ├── bootstrap.rs      # 空库引导（黄金 DDL + 种子嵌入）
│   │       └── auth.rs / crypto.rs
│   └── auth/                     # HS256 鉴权门 crate（四字段错误信封）
├── services/
│   ├── core-service/src/
│   │   ├── server/mod.rs         # ★ tonic registry（一服务一条 add_service）
│   │   ├── service/              # 业务逻辑层（一文件一服务，实现生成的 trait）
│   │   ├── data/                 # 仓储层（一文件一 repo；生成四件套 + 手写特有查询）
│   │   ├── state.rs              # AppState + 错误映射助手（db_status/not_found_status…）
│   │   └── token.rs / config.rs
│   ├── admin-api/src/
│   │   ├── server/rest.rs / sse.rs
│   │   └── services/             # proxies.rs（生成）+ proxy_kit.rs（宏）
│   │                             # + 手写面：authentication（验证码/Cookie）、admin_portal、
│   │                             #   file_transfer（multipart）、internal_message、behaviors
│   └── app-api/src/services/     # proxies.rs（生成）+ authentication / public（公开门禁）/ file_transfer
├── scripts/
│   ├── gen-entities.py           # 黄金 DDL → entity 树
│   ├── gen-repos.py              # 域规格 → repo 生成段
│   ├── gen-proxies.py            # BFF trait → proxies.rs
│   └── e2e-test.sh               # 三服务栈端到端测试（11 节）
└── sql/
    ├── schema.sql                # ★ 黄金 DDL（entity 的唯一事实源）
    ├── seed.sql / demo-data.sql / menus-seed.sql
```

## api 契约层组织（proto-first）

- 快照按 **领域(domain) → service → version** 三级组织；BFF 面（admin/app）的文件以 `i_` 前缀，通过 import 复用领域消息，不重复定义实体；包名 `{domain}.service.v1`。
- **字节冻结快照**：`protos/` 由 `sync-protos.sh` 从契约源同步，`MANIFEST.sha256` 逐文件哈希（BOM 归一化 + `LC_ALL=C` 排序钉死跨机稳定）。**禁止手改**；发现样式问题在契约源头修，再同步进来。
- **buf 工作区**（`api/buf.yaml`）：六个第三方注解模块（googleapis / kratos errors / pagination / PGV / gnostic / redact）由 deps 声明、`buf.lock` 钉定 commit——不 vendor 进仓。`buf breaking --against` 做演进检查；**不做 buf format**（会砸字节清单门）。
- **生成管线**：`crates/proto/build.rs` → `rushwind_proto_build::run` → buf 编译产出**注解闭包**（自定义选项字节保真——prost 再编码会丢 `google.api.http` / `errors.code` 字节，annotated 面必须原生 buf image）→ prost/pbjson 类型 + tonic server/client（全量契约服务，默认 Unimplemented 桩；
  core 侧当前注册 42 个领域服务，2026-11 新契约域随特性批次注册）+ 每 BFF 一张 gen-http 面（`admin_gen.rs` / `app_gen.rs`：路由表 / 绑定计划 / trait / mounts）。admin 与 app 的 BFF 存在同名服务，两张面按 root prefix 切片、各指各的模块（`crate::gen_admin` / `crate::gen_app`），不能共享。

## 关键约定（必须遵守）

### 契约与生成

1. **proto 是单一事实源**——接口、消息、错误码、免鉴权白名单全部先在契约里声明。
2. **禁止手改一切产物**：`api/protos/`、proto crate 构建产物、`store::entities`、`proxies.rs`。
3. **契约变更走同步链**：契约源接受 → `bash api/sync-protos.sh`（重建 MANIFEST）→ `cargo build`（重跑契约构建管线）→ 连产物提交。
4. **schema 变更走 DDL**：`sql/schema.sql` 是 entity 的唯一事实源，改后重跑 `gen-entities.py`。
5. **免鉴权端点登记在 `crates/proto/src/auth_free.rs`**（构建期并入白名单），不在服务代码里散落判断。

### 分层规约

6. **admin-api / app-api 不碰数据库**——只做协议转换、鉴权门、少量 BFF 关注点（验证码 / Cookie / multipart / SSE 扇出）。
7. **数据访问只在 core-service 的 `data/`**；repo 生成段函数对 `C: ConnectionTrait` 泛型——连接与事务通吃。**多语句一致性走 service 层事务**：`self.state.db.begin()` 开启后把 `&txn` 传入各 repo 函数，`txn.commit()` 收口；错误路径 `?` 传播，未 commit 的事务 drop 即回滚。
8. **core 的装配在 `server/mod.rs` 的 `registry()`**：每领域服务一条 `add_service(...)`；新服务漏注册即 Unimplemented——编译不报错，靠测试暴露。

### 命名与组织

9. **core 的 service / data 层一文件一服务/仓储**，与领域一一对应（`user_service.rs` / `user_repo.rs`）。
10. **repo 生成段函数名 = 表名去前缀的蛇形**：`languages_by_id` / `insert_languages` / `update_languages` / `delete_languages` 四件套；领域特有查询手写在文件尾段（或 service 侧）。
11. **proto 类型路径**：`proto::proto::<pkg>::service::v1::<Type>`；文件内用包别名（`use proto::proto::dict::service::v1 as dict_v1;`）控制行宽（rustfmt 100 列）。

### 错误处理

12. **领域服务返回 `tonic::Status`**；REST 面由 http-binding 统一转**四字段错误信封**（code / reason / message / metadata，protojson 编解码）。
13. **repo 层错误映射走 `state.rs` 助手**：`StatusResult<T>` = `Result<T, Status>`；`.map_err(db_status)`（数据库错误归并）+ `.ok_or_else(|| not_found_status("<table>"))`（缺行 404）。业务拒绝用 `bad(...)` / `Status::invalid_argument` 系。

### 操作者与租户元数据

14. **操作者经 gRPC 元数据转发**（`x-user-id`）；**租户元数据 `x-md-global-tenant-id` 优先、`x-tenant-id` 回退**。提取逻辑收拢在 `service/context.rs`（`operator_of` / `tenant_of` / `request_tenant_of` / `operator_tenant_id` / `require_admin_operator`），不要在服务里裸读 metadata。

## 分层模板

### core service 层（真实业务）

实现构建管线生成的 trait（`async_trait`；未实现的方法落默认 Unimplemented 桩，逐个补齐）：

```rust
pub struct LanguageService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::proto::dict::service::v1::language_service_server::LanguageService for LanguageService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<proto::proto::dict::service::v1::ListLanguageResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_languages::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(ListLanguageResponse {
            items: rows.into_iter().map(language_proto).collect(),
            total,
        }))
    }
}
```

entity ↔ proto message 的转换是显式字段映射函数（`fn language_proto(r: sys_languages::Model) -> proto::...::Language`），时间戳经 `state::ts_to_proto`。

### data 层（仓储）

`gen-repos.py` 按域规格生成四件套（对 `C: ConnectionTrait` 泛型，可直接收 `&DatabaseConnection` 或 service 层事务 `&txn`）；领域特有查询手写在生成段之后：

```rust
pub async fn languages_by_id(db: &DatabaseConnection, id: i64) -> StatusResult<sys_languages::Model> {
    sys_languages::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| not_found_status("languages"))
}
```

### BFF 层（薄代理）

**生成面**：`proxies.rs` 是一张 `passthrough_proxy!` 宏表（`proxy_kit.rs` 定义 pass / drop / stub / hand 四形态），由 `gen-proxies.py` 从 OUT_DIR 的 BFF trait 产出——**不手改**。**手写面**独立成模块：`authentication.rs`（验证码 / Cookie / 令牌对轮换）、`admin_portal.rs`（BFF 聚合）、`file_transfer.rs`（multipart 流式）、admin 的 `behaviors.rs` 与 app 的 `public.rs`（行为表 / 公开门禁，宏 hand 形态）。

## 配置管理

YAML 内嵌于二进制（各服务 `assets/`：`data.yaml` / `auth.yaml` / `server.yaml`；core 另有 `oss.yaml`），环境变量覆盖：

| 环境变量 | 作用 |
|---|---|
| `RUSHWIND_DATABASE_SOURCE` | PostgreSQL DSN（URL 形式） |
| `RUSHWIND_REDIS_ADDR` / `RUSHWIND_REDIS_PASSWORD` | Redis 连接；**空串也会覆盖 yaml 默认，不设该变量即用默认** |
| `jwt_signing_key` | HS256 签名密钥（`auth.yaml` 内嵌值为开发演示密钥，生产必换） |
| `RUSHWIND_MINIO_*` | MinIO endpoint / 凭据等（`oss.yaml` 的 `minio.endpoint` 非空即选 MinIO，否则本地盘） |

## 数据与引导

- `store::bootstrap` 把四份 SQL 嵌进二进制：`schema.sql`（黄金 DDL，id 列带 nextval 默认）→ `seed.sql`（系统种子）→ `demo-data.sql`（演示数据）→ `menus-seed.sql`（菜单树，对齐 admin-react 路由）。**core-service 先起、空库时自动引导**，无需手工跑 SQL。
- 种子账号：管理员 `admin / admin`；登录带图形验证码（Redis，`cms:captcha:<id>`）。
- Redis 键族：会话令牌 `gwc:at/rt/bl:`（`ct` 0=admin 1=app）、验证码 `cms:captcha:`。

## 构建与开发命令

| 命令（`backend/` 下） | 作用 |
|---|---|
| `cargo build` | 全工作区（proto crate 的 build.rs 需要 buf 在 PATH） |
| `cargo run -p core-service` | 领域层 gRPC :6602（先起；空库自动引导） |
| `cargo run -p admin-api` / `-p app-api` | 管理 / 前台 BFF（REST + SSE） |
| `cargo test --workspace` | 全部单测 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Lint 门（workspace lints：clippy all + undocumented_unsafe_blocks） |
| `cargo fmt -p proto -p auth -p store -p core-service -p admin-api -p app-api -- --check` | 格式门（**按包清单跑，不用 `--all`**；rustfmt 默认风格、100 列） |
| `bash api/sync-protos.sh` / `--check` | 契约同步 / 校验门 |
| `python3 scripts/gen-entities.py` | schema 重导出后重生成 entity 树 |
| `python3 scripts/gen-repos.py` | 域规格变更后幂等重生成仓储生成段 |
| `python3 scripts/gen-proxies.py <admin_gen.rs> admin > services/admin-api/src/services/proxies.rs`（app 同理） | 契约重同步后重生成 BFF 代理表 |
| `bash scripts/e2e-test.sh` | E2E（前置：三服务已起、PG/Redis 可达） |

**开发工作流**：契约变更 → 同步脚本 → `cargo build` → 按需重跑生成器 → 补 core 实现 / 手写面 → 门禁（fmt / clippy / test）→ e2e。

## 新增领域服务 Checklist

1. **契约**：在契约源定义领域 proto（`<domain>/service/v1/<entity>.proto`）+ BFF 面（`i_<entity>.proto`，import 领域消息）→ 接受同步：`bash api/sync-protos.sh`（先 `buf breaking` 演进检查）。
2. **构建**：`cargo build` 产出类型 / tonic 面 / gen-http 面（服务默认 Unimplemented 桩）。
3. **core service**：`service/<entity>_service.rs` 实现生成的 trait（entity ↔ proto 显式映射函数）；`server/mod.rs` 的 `registry()` 追加 `add_service(...)`。
4. **data**：新表先进 `sql/schema.sql`（黄金 DDL）→ `gen-entities.py`；`gen-repos.py` 的 SPEC 加域（或按既有 repo 手写）；特有查询写在生成段后。
5. **BFF**：`gen-proxies.py` 重跑两份 proxies.rs；若该服务需手写（BFF 关注点），登记进 gen-proxies.py 的跳过清单并在手写模块实现。
6. **免鉴权端点**（如有）：登记 `crates/proto/src/auth_free.rs`。
7. **验证**：`e2e-test.sh` 补节；全门（fmt / clippy / test / 同步门）跑绿。

## 常见错误与纠正

| 错误做法 | 正确做法 |
|---|---|
| 在 BFF 里写数据库访问 / 领域逻辑 | BFF 是 pass-through；逻辑落 core-service |
| 手改 `api/protos/`、`proxies.rs`、entity 树 | 改源头（契约源 / DDL / 规格）后重新生成，连产物提交 |
| 手写 HTTP 路由 | 路由来自 gen-http 面（`google.api.http` 注解）；确需手工挂载的走既有手挂先例并注释原因 |
| 裸读 gRPC metadata 取操作者/租户 | 用 `service/context.rs` 的提取助手 |
| repo 层手拼错误 Status | `db_status` / `not_found_status` / `bad` 助手，保持信封一致 |
| JSON 断言里把 int64 写成数字 | protojson 64 位整数是字符串（`"id":"123"`） |
| `cargo fmt --all` | 按 CI 的 `-p` 包清单跑（rustfmt 不碰框架依赖树） |
| 为过 MANIFEST 门手改清单 | 清单只能由同步脚本重建；契约演进走 `buf breaking` + 差分回归 |
| 忘了 core-service 先起 | 引导（建表/种子）在 core；BFF 依赖其 gRPC |
| 忽略 Unimplemented 桩直接联调 | 新服务必须完成 `registry()` 注册 + trait 实现，E2E 兜底 |

## 关键文件索引

**架构骨架**
- `crates/proto/build.rs` — 契约构建管线配置面（faces / 依赖数据文件 / 免鉴权白名单）
- `services/core-service/src/server/mod.rs` — tonic registry（装配拓扑看这里）
- `services/admin-api/src/server/rest.rs` / `sse.rs` — REST 路由 / SSE hub
- `services/core-service/src/state.rs` — AppState + 错误映射助手

**user 链路（完整参考）**
- `api/protos/identity/service/v1/user.proto` 与 `api/protos/admin/service/v1/i_user.proto` — 领域契约 + BFF 面
- `services/core-service/src/service/user_service.rs` — core 业务
- `services/core-service/src/data/user_repo.rs` — 仓储
- `services/admin-api/src/services/proxies.rs` — BFF 代理表（生成物形态参考）

**认证与元数据**
- `crates/auth/` — HS256 鉴权门；`services/core-service/src/token.rs` — 令牌对签发/轮换
- `services/core-service/src/service/authentication_service.rs` — 认证内核（AES + bcrypt）
- `services/core-service/src/service/context.rs` — 操作者/租户元数据提取

**生成器与门**
- `backend/scripts/gen-entities.py` / `gen-repos.py` / `gen-proxies.py`
- `backend/api/sync-protos.sh` + `MANIFEST.sha256`
- `backend/sql/schema.sql` — 黄金 DDL（entity 事实源）
