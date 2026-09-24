# rushwind-cms 开发评估与计划

> 目标：以 Rust 复刻 `go-wind-cms`（下称「Go 后端」，GoWind Content Hub），
> **proto 为唯一 API 契约，五个前端（admin-react + 前台 react/vue/taro/flutter）
> 零改动可对接两个后端**。底座：`rushwind`（框架 monorepo）+ `rust-utils`（工具库）。
> 移植模式与 `rushwind-admin` 同源——本计划是其开发模式的 CMS 投射，
> 已验证的框架机制（http-binding / gen-http / authn-jwt / bootstrap）直接复用，
> CMS 特有差异逐条登记。

---

## 1. 与 rushwind-admin 的关键差异（决定本项目独有工作）

| # | 差异 | 处理 |
|---|------|------|
| C1 | **双 BFF 面**：Go 侧分 admin（:6600/:6601）与 app（:6700/:6701）两个服务、两份白名单 | 一个契约 crate、两个服务 crate（admin-api / app-api）；生成器按 BFF 子闭包各跑一次（admin 面 42 服务 / app 面 11 服务），AUTH_FREE 取两白名单并集（路由集不相交，互不干扰） |
| C2 | 生成器 trait 按服务短名命名，两个 BFF 有同名服务（TagService 等） | 每 BFF 一个生成面（gen_admin / gen_app），自引用 `crate::gen::` 改写为各自模块名（见 §3 关键事实） |
| C3 | **JWT HS256**（共享秘钥），非 admin 项目的 RS256 | authn-jwt `with_key`；`jwt_signing_key` 环境变量与 Go 侧 `${jwt_signing_key:...}` 占位同名 |
| C4 | Redis 键族前缀 `gwc:`，at 键形 `gwc:at:{ct}:{uid}:{jti}`（ct=client type 0/1） | TokenStore 按客户端类型分键空间（admin=0 / app=1），与 Go `user_token_cache.go` 逐字对位 |
| C5 | app BFF 有一条 proto 外的手工路由：`POST /app/v1/register` | app-api rest.rs 手工挂载（bind 层 + 白名单），对位 Go 侧手工注册 |
| C6 | 内容域模型更重：内容建模（content_model）/ 富文本 section / 多语言翻译（GetTranslation）/ OpenSearch 全文搜索（SearchPosts）/ 媒体资产 / 互动（点赞/收藏/观看）/ 评论审核 / 站点与导航 | Phase 2+ 按模块流水线逐个落地；OpenSearch 客户端选型（考 rust-opensearch）在内容批次前定 |
| C7 | Go 侧三服务（admin BFF / app BFF / core 领域层，gRPC 互联） | Rust 侧合并为单体（BFF 直连存储），对位 rushwind-admin 的合并策略；gRPC 通道不复刻（Go 侧也未注册 gRPC server 给外部） |

## 2. 硬约束（「前端零改动」）

与 rushwind-admin 的 T1-T6 / B1-B6 / R1-R4 / SSE 契约同族，此处只登记 CMS 侧的实测差异：

- admin 前端（admin-react）dev :5999 → 代理 REST :6600；前台 react（Next.js）dev :5001 直连 :6700
- 前台 AES key `f51d66a73d8a0927`（NEXT_PUBLIC_AES_KEY，与 admin 侧同值）——登录口令应用层加密
- CORS origins 见各 server.yaml（admin：localhost:5999；app：5001/5011/5021/10086 + 演示域名）
- 错误信封同为 Kratos Status protojson 四字段；reason→状态由各 BFF 的 `*_error.proto` 注解表锚定

## 3. 关键事实（Phase 0 会话沉淀）

- **prost 解码丢弃未知字段**：任何经 prost_types 解码再编码的 FileDescriptorSet 都会
  丢失自定义选项字节（google.api.http / errors.code）——BFF 子闭包切片因此是
  **字节级**的（顶层记录过滤 + 逐记录 name/dependency 最小解析，记录体原样复制），
  不经任何消息重编码。
- **sea-orm 2.0 的 DSN 只认 URL 形式**（`postgres://user:pass@host/db?sslmode=disable`），
  Go 风格 key=value 串解析失败。
- buf 工作区输出文件名不带模块路径前缀（`admin/service/v1/...`），与 import 引用一致。
- admin 面 42 个 BFF 服务、app 面 11 个；遮蔽路由恰一条（`GET /admin/v1/apis/walk-route`
  被 `{id}` 模式吸收，与 rushwind-admin 同款注册顺序遮蔽）。
- app 白名单 17 个 proto 操作 + 1 条手工路由；admin 白名单 4 个操作（Login /
  GenerateCaptcha / VerifyCaptcha / RefreshToken）。

## 4. 分阶段计划

### Phase 0 — 契约面与装配面 ✅（已完成）

- [x] 仓库脚手架：workspace（crates/proto + crates/auth + services/*）、CI（fmt / clippy /
      test / proto 门 / 五前端快照门，ubuntu + windows 矩阵）
- [x] proto 同步 + MANIFEST 门 + third_party vendor（六模块，与 rushwind-admin 同源）
- [x] 契约 crate：buf 注解闭包（选项字节保真）+ protox 过滤集 → prost/pbjson 类型 +
      双 BFF 生成面（路由 / 绑定计划 / trait / 错误表 / null 桩 / 公网-门控挂载拆分）
- [x] 白名单语料守卫（双向钉死 + 遮蔽集钉死）
- [x] admin-api / app-api 装配：bind 预绑定层 + HS256 鉴权门（四字段信封，消息文案
      missing bearer token / access token expired）+ gorilla 兼容 CORS + SSE :6601/:6701
      + 手工注册路由；配置内嵌（data/auth/server yaml + 环境覆盖）
- [x] Redis 会话键族（gwc:at/rt/bl，双客户端类型）
- [x] 五个前端快照同步 + RushWind 品牌覆写 + 双清单门
- [x] 端到端冒烟（本地 PG/Redis）：门控 401 双形态 / 公开桩 500 Unknown /
      坏 body 400 CODEC / 注册路由 bind / CORS 允许与拒绝 / SSE 预检与 401 文本形态

### Phase 1 — 认证与会话内核

- [ ] 登录链：验证码（rust-utils captcha + Redis）→ AES-CBC 口令解密 → bcrypt +
      恒时假校验 → 失败归一 INVALID_PASSWORD → 登录限流
- [ ] HS256 令牌对签发（claims 键名逐键对位 Go 侧 UserTokenPayload）+ gwc: 键族
      落库 + refresh cookie 三态 + logout 吊销
- [ ] app 面：C 端注册（RegisterUser）+ 短期 token（15min，refresh 禁用语义）
- [ ] 两前端完整登录/登出/刷新 E2E

### Phase 2 — 鉴权/数据横切

- [ ] 租户门（站点域解析 TenantResolver：app 面按 Host 解析 tenant → 匿名只读 viewer，
      fail-closed）+ 套餐/状态检查
- [ ] RBAC 策略装载 + 判定日志（sys_policy_evaluation_logs）
- [ ] SeaORM 数据层：动态 schema（50+ 表）、租户谓词注入、数据范围
- [ ] 审计中间件（api/login/operation 三类起步）+ 黄金 DDL 管道

### Phase 3 — 模块流水线（最长尾）

| 批 | 模块 | 特殊点 |
|----|------|--------|
| A 平台 | language / dict×2 / menu / permission / api / role / tenant / user / org_unit / position | 与 rushwind-admin 同族的 RBAC 基座 |
| B 内容 | content_model / post / category / tag / page / section / 翻译（GetTranslation） | 内容建模元数据驱动；多语言子表；富文本/Markdown |
| C 互动 | comment（审核/游客策略）/ interaction（点赞/收藏/观看）/ stats | 游客可选认证；公开计数 |
| D 站点 | site / site_setting / navigation / navigation_item / media_asset / file / file_transfer | 多站点独立配置；OSS（MinIO）；HMAC 图链 |
| E 运维 | task / translator / internal_message×3 / 六类审计查询 / dashboard | SSE 推送；OpenSearch（SearchPosts）选型落地 |

### Phase 4 — 收尾

- [ ] 差分回归台架（Go/Rust 双后端回放，归一比较器 + 豁免集）
- [ ] 部署对齐（docker-compose / Dockerfile / 配置 schema 等价）
- [ ] 文档：ARCHITECTURE / 模块新增指南 / 运维手册

---

## 5. 风险登记

| # | 风险 | 缓解 |
|---|------|------|
| R1 | 双 BFF 生成面的自引用改写属构建期字符串操作，生成器升级若改变自引用形态会静默失配 | 升级 rushwind-gen-http rev 后全量重建并跑语料守卫；中期把「模块路径参数化」回馈上游生成器 |
| R2 | OpenSearch 全文（SearchPosts）在 Rust 侧的等价实现（分词/评分） | 差分豁免集起步，逐步收敛；必要时经 HTTP API 而非客户端库 |
| R3 | 内容建模元数据驱动的动态字段（section/content_model）无静态 schema | 参照 rushwind-storage-seaorm 的动态 Schema 能力；金样测试钉死 |
| R4 | 多语言翻译（i18n map / GetTranslation）批量子表语义 | 逐端点差分 |

> 本计划为活文档：每阶段收口后回填实际偏差与决策修订。

---

## 6. 会话记录（2026-09-24，Phase 1 + 部分 Phase 3）

- **三服务重构（用户指示）**：对位 Go 侧拓扑落成 core-service（gRPC :6602，独占 PG/Redis，
  tonic）+ admin-api / app-api 薄 BFF。proto crate 同一 prost pass 嵌入 tonic-prost 服务
  生成（`.generate_default_stubs(true)`——trait 方法默认 Unimplemented，逐服务落地）；
  tonic 0.14 的 proto 驱动代码gen在 tonic-prost-build（tonic-build 0.14 已改手工模式）。
- **BFF 代理生成器**（scripts/gen-proxies.py）：从生成的 BFF trait 自动产出 pass-through
  代理（admin 39 / app 8）；签名错位方法登记 STUB_METHODS 桩表（sync_apis 等 7 处）；
  Authentication/AdminPortal/FileTransfer/UserProfile(app) 手写或保持桩。
- **黄金 DDL 管线**：ent 迁移经 dump-schema 程序导出（约束三遍排序：PK→UNIQUE→FK）；
  id 列补 `DEFAULT nextval`（ent 客户端取号的 Rust 侧等价）；正则表名须含数字
  （`sys_dict_entry_i18n` 教训）。种子=系统种子（对位 default_data.go）+上游演示数据；
  双逗号/无默认 NOT NULL 列逐项修复后空库一次性引导通过。
- **已落地 core 领域服务**：认证内核（登录密码授权：租户解析→AES+bcrypt+恒时假校验→
  权限门→HS256 令牌对；刷新 verify-and-revoke Lua 轮换；登出 SCAN 前缀吊销；注册事务+
  tenant:user 角色绑定；ValidateToken 会话裁决）、dict×3、post/category/tag/page
  （翻译子表+关联表+枚举名映射）、comment、interaction 计数、site×4、user/role/tenant 读。
- **E2E 套件**（scripts/e2e-test.sh）：44 断言全绿（验证码单次消费、错误口令防枚举、
  令牌门控、13 模块真实数据、CRUD 增改删、刷新轮换后旧令牌 401、登出全吊销、
  app 注册/登录（900s 无 refresh）/公开读/游客评论、SSE 预检、CORS）。
- **坑位登记**：本地 Redis 需认证（密码 `*Abcd123456`，空密码环境变量会以空串覆盖
  yaml 默认——env_string 语义）；`expires_in` 为 int64 → protojson 字符串化；
  CreateXRequest 一律 `{"data":{...}}` 包装；后台服务进程要用任务托管（nohup 子壳会被
  回收）。
