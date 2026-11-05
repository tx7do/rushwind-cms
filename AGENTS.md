# AGENTS.md — RushWind CMS Monorepo 开发指南（AI Agent 入口）

本文件是 monorepo 总入口：定位布局、指路各端文档、声明全仓铁律。**深入开发前先读本文件与对应端的相关文档**。

## 仓库布局

RushWind CMS（无头内容平台）：一个 Rust 后端 + 一个管理后台前端 + 四套 C 端前端的 Headless CMS。proto 是唯一 API 契约，前端以零改动快照对接后端。

```
backend/                        Rust 工作区（crates/* + services/*，契约驱动）
├── api/                        契约层：proto 快照（MANIFEST 校验门）+ buf 工作区 + 同步脚本
├── crates/
│   ├── proto/                  契约 crate：build.rs → buf 编译 → prost/pbjson 类型
│   │                           + REST/gRPC 双生成面 + 描述符池（构建期产物，不手改）
│   ├── store/                  共享数据层：entity 树（脚本生成）+ 分页管线 + 空库引导 + 认证查询
│   └── auth/                   HS256 鉴权门 crate
├── services/
│   ├── core-service/           领域层（gRPC :6602，独占 PostgreSQL/Redis，不对外暴露）
│   ├── admin-api/              管理 BFF（REST :6600 + SSE :6601，薄代理）
│   └── app-api/                前台 BFF（REST :6700 + SSE :6701，薄代理）
├── scripts/                    代码生成器（gen-entities / gen-repos / gen-proxies）+ e2e-test.sh
└── sql/                        黄金 DDL + 系统种子 + 演示数据 + 菜单种子（构建/引导期嵌入二进制）
frontend/
├── admin/                      管理后台快照（admin-react）+ 品牌覆写层 + 双清单门
└── app/                        四套前台快照（react/vue/taro/flutter_app）+ 品牌覆写层 + 双清单门
docs/                           专题文档（development-plan 等）
```

**心智模型**：admin-api / app-api 是脚本生成的薄 BFF（pass-through 代理 + 少量手写面），领域逻辑与持久化全部集中在 core-service；五个前端均为字节冻结快照，唯一允许的对上游偏离是 RushWind 品牌覆写。

**管理后台以 admin-react（React 19 + Ant Design v6）为唯一基线**；C 端四套并列（react/vue/taro/flutter_app），一次部署取一套。

## 门禁（提交前必须全绿）

| 范围 | 命令（在 `backend/` 下，另注明者除外） |
|---|---|
| 格式 | `cargo fmt -p proto -p auth -p store -p core-service -p admin-api -p app-api -- --check` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| 测试 | `cargo test --workspace` |
| 契约门（仓根执行） | `bash backend/api/sync-protos.sh --check` |
| 快照门（仓根执行） | `bash frontend/admin/sync-frontend.sh admin-react --check`；`bash frontend/app/sync-frontend.sh <react\|vue\|taro\|flutter_app> --check`（改动落在哪套跑哪套） |
| E2E（涉及后端行为） | 先起三服务（见下），`bash backend/scripts/e2e-test.sh` |

**门禁出现任何新报错，一律当作自己引入的 bug 修复**，不存在"可忽略的既有错误"。改完代码先跑门禁再声称完成。CI（`.github/workflows/ci.yml`）在 ubuntu + windows 双平台执行同一组门。

## 全仓铁律

1. **生成物与快照不手改**：`backend/api/protos/`（MANIFEST 校验门）、五个前端快照树（MANIFEST + UPSTREAM 双清单门）、proto crate 构建产物（OUT_DIR）、`store` 的 entity 树（`scripts/gen-entities.py` 产物）、BFF 的 `proxies.rs`（`scripts/gen-proxies.py` 产物）、`gen-repos.py` 产出的仓储生成段——全部是产物。改源头（契约源 / DDL / 规格 / 脚本）后**重新生成并连同产物一并提交**。
2. **BFF 不碰数据库**：admin-api / app-api 只做协议转换与转发（生成代理 + 手写面）；数据访问只发生在 core-service。
3. **CRUD 写请求体必须包 `{"data": {...}}`**（请求消息的 `data` 字段），更新配合 `update_mask` 做部分更新。列表搜索条件一律 contains（模糊匹配），ID 类字段不进模糊搜索。**int64 字段 protojson 字符串化**——构造或断言 JSON 时写 `"expires_in":"900"` 而非 `900`。
4. **契约与快照的演进必须显式接受**：上游漂移 / 新 vintage 一律走同步脚本重建清单（proto 侧先过 `buf breaking` 演进检查，随后全量差分回归），禁止为过门而手改清单、删门或对快照做"顺手修补"。
5. **不吞错**：错误必须映射传播（`StatusResult` → `tonic::Status` 带上下文），不允许静默丢弃；合法的 best-effort 兜底仅限纯本地非关键路径且注释写明原因。

## 本地验证要点

- **buf 必须在 PATH**：proto crate 的 `build.rs` 走 `buf build` 编译契约（选项字节保真，CI 钉 1.73.0）；`cargo build` 报找不到 buf 先查 PATH（本机常装在 `~/.local/bin`）。
- **启动顺序**：core-service 先起——空库自动引导 schema + 种子 + 演示数据 + 菜单种子（PostgreSQL / Redis 须可达）；随后 admin-api、app-api。管理员账号 `admin / admin`。
- **登录带图形验证码**（Redis 存储，键 `cms:captcha:<id>`）——e2e 脚本直接读 Redis 取答案；本机 Redis 启用密码时与 `e2e-test.sh` 内的连接参数保持一致。
- **refresh 令牌对轮换**：刷新接口会轮换令牌对，手动调用会使旧 access token 立即失效，表现为"假 401"，别误判成鉴权 bug。app 面 access token 900 秒且无 refresh。
- **`RUSHWIND_REDIS_PASSWORD` 空串也会覆盖 yaml 默认**——想用配置默认值就不要设这个变量。
- **sea-orm 2.0 的 DSN 只认 URL 形式**（`postgres://user:pass@host/db?sslmode=disable`）。
- **对象存储双后端**：core 的 `assets/oss.yaml` 的 `minio` 段 `endpoint` 非空即选 MinIO，否则回落本地盘；`RUSHWIND_MINIO_*` 环境变量可覆盖。
- **前端 dev**：Vite 8 要求 Node ≥ 20.19；若曾用低版本 Node 装过依赖，须用新版 Node `pnpm install --force`（否则静默缺平台二进制）。react 快照用 `next dev --webpack`（turbopack 依赖 inotify 配额，本机配额常被 IDE 占满而 500）。
- **快照树内的构建缓存**（`.next/`、`.vite/` 等）已被 gitignore 且不进清单，属正常；同步脚本一律用 bash 执行。
- **Windows 构建**：`profile.dev` 已配 `debug = "line-tables-only"`（MSVC PDB 4GB 上限，全量调试信息链接即爆 LNK1140），勿改回全量。

## 文档索引

- 根 README（三语）：[README.md](./README.md) / [README.en-US.md](./README.en-US.md) / [README.ja-JP.md](./README.ja-JP.md)
- 后端：[backend/AGENTS.md](./backend/AGENTS.md)（后端规范单一事实源：分层模板、生成器、约定、Checklist）
- 开发计划与进度：`docs/development-plan.md`（阶段推进的唯一事实源）
- 前端快照模型：[frontend/README.md](./frontend/README.md)（同步 / 品牌覆写 / 双清单门总览）
- 快照层指南：[frontend/admin/AGENTS.md](./frontend/admin/AGENTS.md) / [frontend/app/AGENTS.md](./frontend/app/AGENTS.md)
- 品牌覆写登记：`frontend/admin/brand/README.md` / `frontend/app/brand/README.md`（偏离面唯一登记处）
- CI 门定义：`.github/workflows/ci.yml`
- 快照内层文档：各快照自带源仓的 README / AGENTS.md（脚手架开发规范参考——**只读，改动不落地**）
