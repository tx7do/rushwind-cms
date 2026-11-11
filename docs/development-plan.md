# rushwind-cms 开发评估与计划

> 目标：以 Rust 构建 RushWind CMS——全栈 Headless 内容平台，
> **proto 为唯一 API 契约，五个前端（admin-react + 前台 react/vue/taro/flutter）
> 零改动可对接**。底座：`rushwind`（框架 monorepo）+ `rust-utils`（工具库）。
> 框架机制与姊妹项目 `rushwind-admin` 共用（http-binding / gen-http / authn-jwt /
> bootstrap），CMS 特有差异逐条登记。

---

## 1. 关键架构决策（决定本项目独有工作）

| # | 决策 | 处理 |
|---|------|------|
| C1 | **双 BFF 面**：admin（:6600/:6601）与 app（:6700/:6701）两个对外服务、两份白名单 | 一个契约 crate、两个服务 crate（admin-api / app-api）；生成器按 BFF 子闭包各跑一次（admin 面 42 服务 / app 面 11 服务），AUTH_FREE 取两白名单并集（路由集不相交，互不干扰） |
| C2 | 生成器 trait 按服务短名命名，两个 BFF 有同名服务（TagService 等） | 每 BFF 一个生成面（gen_admin / gen_app），自引用 `crate::gen::` 改写为各自模块名（见 §3 关键事实） |
| C3 | **JWT HS256**（共享密钥） | authn-jwt `with_key`；密钥经 `jwt_signing_key` 环境变量注入 |
| C4 | Redis 键族前缀 `gwc:`，at 键形 `gwc:at:{ct}:{uid}:{jti}`（ct=client type 0/1） | TokenStore 按客户端类型分键空间（admin=0 / app=1） |
| C5 | app BFF 有一条 proto 外的手工路由：`POST /app/v1/register` | app-api rest.rs 手工挂载（bind 层 + 白名单） |
| C6 | 内容域模型更重：内容建模（content_model）/ 富文本 section / 多语言翻译（GetTranslation）/ OpenSearch 全文搜索（SearchPosts）/ 媒体资产 / 互动（点赞/收藏/观看）/ 评论审核 / 站点与导航 | Phase 2+ 按模块流水线逐个落地；OpenSearch 客户端选型（考 rust-opensearch）在内容批次前定 |
| C7 | **三服务拓扑**：core-service（领域层，对内 gRPC :6602，独占 PG/Redis）+ admin-api / app-api 薄 BFF | 已落地（见 §6）：BFF 经 gRPC 调 core，不直连存储；core 不对外暴露 |

## 2. 硬约束（「前端零改动」）

- admin 前端（admin-react）dev :5999 → 代理 REST :6600；前台 react（Next.js）dev :5001 直连 :6700
- 前台 AES key `f51d66a73d8a0927`（NEXT_PUBLIC_AES_KEY，与 admin 侧同值）——登录口令应用层加密
- CORS origins 见各 server.yaml（admin：localhost:5999；app：5001/5011/5021/10086 + 演示域名）
- 错误信封为 Kratos Status protojson 四字段；reason→状态由各 BFF 的 `*_error.proto` 注解表锚定

## 3. 关键事实（Phase 0 会话沉淀）

- **prost 解码丢弃未知字段**：任何经 prost_types 解码再编码的 FileDescriptorSet 都会
  丢失自定义选项字节（google.api.http / errors.code）——BFF 子闭包切片因此是
  **字节级**的（顶层记录过滤 + 逐记录 name/dependency 最小解析，记录体原样复制），
  不经任何消息重编码。
- **sea-orm 2.0 的 DSN 只认 URL 形式**（`postgres://user:pass@host/db?sslmode=disable`），
  key=value 串解析失败。
- buf 工作区输出文件名不带模块路径前缀（`admin/service/v1/...`），与 import 引用一致。
- （2026-11 批次前基线——现值见下方 2026-11-06 刷新记录）admin 面 42 个 BFF 服务、
  app 面 11 个；遮蔽路由恰一条（`GET /admin/v1/apis/walk-route`
  被 `{id}` 模式吸收，与 rushwind-admin 同款注册顺序遮蔽）。
- （同上，批次前基线）app 白名单 17 个 proto 操作 + 1 条手工路由；admin 白名单 4 个操作
  （Login / GenerateCaptcha / VerifyCaptcha / RefreshToken）。
- **2026-11-06 契约批次刷新**：接受上游 vintage 至 161d991c——proto 136 → 209 文件
  （AI 全家桶 / 通知 / 社交 / 监控 / 流程 / 表单 / 备份 / 配额 / MFA / AK-SK /
  在线会话 / 编辑锁 / 实验 / 重定向 / 审批留痕 / 搜索词库等 76 个新文件，约 341
  个新 RPC；既有 20 文件扩约 70 RPC）。admin 面 76 服务（薄代理 71 + 手写
  authentication / admin-portal / file-transfer / component-definition /
  preview）、app 面 14 服务（薄代理 12）；遮蔽路由两条（webhooks/deliveries
  随新契约加入，钉子改为 method+path 有序对——索引随契约增长漂移）；
  app 白名单 27 操作、admin 白名单 10 操作。生成器新通道：renamed（同服务
  改名转发：MFA×3、OnlineSession×4）、stub（GetBySlug×2 / ListEnabled /
  Chat——域无对应或流式错位）；MFA 挑战校验走 behaviors 逐字段映射
  （LoginResponse ↔ VerifyMfaChallengeResponse，镜像上游 Go BFF）。
  core 侧 tonic trait 自带 unimplemented 默认体——新 RPC 零 stub 编译通过，
  逐域落实现随特性批次推进；薄代理先行拨号，运行期答 UNIMPLEMENTED。
  Rust 底层同批升级：rushwind eaa813e1→a344105b、rust-utils 8c24baf4→7ff37eb1、
  protox 0.9→0.10、jsonwebtoken 9→11、rustc 1.99.1。前端五快照整树重同步
  （admin 上游切 React19+AntD v6；flutter 全量重写：token 化主题 + 四档断点
  + 18MB 可变字体入库）；品牌表扩容（上游产品名归一 GoWind Content Hub、
  新增 vue ar 语言包、flutter 原生工程显示名、taro 工程面），双清单重钉。
- **2026-11-11 契约批次刷新**：接受上游 vintage 至 faeceafd（破坏性定版，buf
  breaking 显式接受）。proto 209 文件中 88 个原位变更：ID 全链 uint32 →
  int64、分页 total uint64 → int64（946 处字段类型变更，零删字段、零新增
  RPC/消息——protojson 面所有 ID 从数字变字符串）；唯一语义新增是 role 的
  parentId/parentName（父子角色继承）。领域行为随批移植：角色权限继承全栈
  （sys_roles 加 parent_id 列 + idx_sys_roles_parent_id、Create/Update 的
  子角色权限 ⊆ 父角色绑定集校验、换父环检测深度 32、parent_id 三态/全量
  更新双口径、删父角色同事务解除子引用、List/Get 回填 parentName）、角色码/
  权限码保留字校验（platform:*/template:*/tenant:manager 与 sys: 命名空间，
  新 constants.rs 模块）、权限同步 MERGE 化（按 code 保 id 增量对齐——原位
  更新不触碰 status、缺失插入、库有派生集无的删除并同事务级联清角色授权与
  接口/菜单绑定；分组行保持重建语义）、role Update 全量语义权限关联落库 +
  Delete 不存在 404。Rust 底层同批升级：rushwind a344105b→08fbfcd（authn-gate/
  SSE 的 uid 落 i64 身份教条 + 两轮安全审计战役——constant-time 比较、bind 层
  body 限额、msgpack/cbor 深度守卫等）、rust-utils 7ff37eb1→8f3e991；仓内
  UserTokenPayload/TokenStore 的 uid/tenant_id 同步落 i64，全仓 ~260 处编译错误
  + ~200 处 clippy 残留（同型转换/恒等 map）一次性收敛（u32/u64→i64）。前端
  五快照整树重同步（admin 169 changed + 3 new——
  @rc-component/table 新 patch、角色抽屉父角色 TreeSelect + 保留码即时校验
  + i18n；react 37 / vue 35 / taro 22 / flutter 1 changed——ID 字符串化适配），
  双清单重钉；品牌表无新增行（新增面全部落在已登记豁免/既有覆盖内）。
  验证：buf breaking 946 处类型变更先记录后接受、proto 门绿、fmt/clippy
  -D warnings 全仓绿、workspace 104 测试绿（新增保留码口径 3 测）、五快照门
  绿、core-service 空库引导（schema+种子+演示+菜单）按新 DDL 落库
  （sys_roles 含 parent_id）。E2E 维持既有瘫痪基线：13 PASS / 112 FAIL——
  全部失败落在登记在案的 redact REST 瘫痪（i_user 的 redact.internal_method
  vs plan-builder 拒绝，本批 88 文件不含 i_user、crates/proto 零触碰，故障
  形态与 2026-11-10 前记录一致：401 错误路径全过、成功响应路径在 redact
  阶段 panic）；全绿仍待上游 redact 决策。

## 4. 分阶段计划

### Phase 0 — 契约面与装配面 ✅（已完成）

- [x] 仓库脚手架：workspace（crates/proto + crates/auth + services/*）、CI（fmt / clippy /
      test / proto 门 / 五前端快照门，ubuntu + windows 矩阵）
- [x] proto 同步 + MANIFEST 门；第三方注解转 buf.lock 钉定（六 BSR 模块，与上游 lock
      同源；原 third_party vendor 已退役）
- [x] 契约 crate：buf 注解闭包（选项字节保真）+ 过滤解码 → prost/pbjson 类型 +
      双 BFF 生成面（路由 / 绑定计划 / trait / 错误表 / null 桩 / 公网-门控挂载拆分）
- [x] 白名单语料守卫（双向钉死 + 遮蔽集钉死）
- [x] admin-api / app-api 装配：bind 预绑定层 + HS256 鉴权门（四字段信封，消息文案
      missing bearer token / access token expired）+ gorilla 兼容 CORS + SSE :6601/:6701
      + 手工注册路由；配置内嵌（data/auth/server yaml + 环境覆盖）
- [x] Redis 会话键族（gwc:at/rt/bl，双客户端类型）
- [x] 五个前端快照同步 + RushWind 品牌覆写 + 双清单门
- [x] 端到端冒烟（本地 PG/Redis）：门控 401 双形态 / 公开桩 500 Unknown /
      坏 body 400 CODEC / 注册路由 bind / CORS 允许与拒绝 / SSE 预检与 401 文本形态

### Phase 1 — 认证与会话内核 ✅（已完成，见 §6）

- [x] 登录链：验证码（rust-utils captcha + Redis）→ AES-CBC 口令解密 → bcrypt +
      恒时假校验 → 失败归一 INVALID_PASSWORD → 登录限流
- [x] HS256 令牌对签发（claims 键名与前端令牌解析约定逐键一致）+ gwc: 键族
      落库 + refresh cookie 三态 + logout 吊销
- [x] app 面：C 端注册（RegisterUser）+ 短期 token（15min，refresh 禁用语义）
- [x] 两前端完整登录/登出/刷新 E2E

### Phase 2 — 鉴权/数据横切 ✅（已完成，2026-11-06 盘点确认）

- [x] 租户门（站点域解析 TenantResolver：app 面按 Host 解析 tenant → 匿名只读 viewer，
      fail-closed）+ 套餐/状态检查（登录链 `tenant.status == "ON"` 门 + 用户状态门）
- [x] RBAC 策略装载 + 判定日志（sys_policy_evaluation_logs）
- [x] SeaORM 数据层：动态 schema（50+ 表）、租户谓词注入、数据范围
- [x] 审计中间件（api/login/operation 起步 → 落地五类：api / login / operation /
      data_access / permission）+ 黄金 DDL 管道

### Phase 3 — 模块流水线（最长尾）

| 批 | 模块 | 特殊点 | 进度（2026-11-06） |
|----|------|--------|--------------------|
| A 平台 | language / dict×2 / menu / permission / api / role / tenant / user / org_unit / position | 与 rushwind-admin 同族的 RBAC 基座 | ✅ 全落 |
| B 内容 | content_model / post / category / tag / page / section / 翻译（GetTranslation） | 内容建模元数据驱动；多语言子表；富文本/Markdown | ✅ 除 section（未落） |
| C 互动 | comment（审核/游客策略）/ interaction（点赞/收藏/观看）/ stats | 游客可选认证；公开计数 | ✅ 全落 |
| D 站点 | site / site_setting / navigation / navigation_item / media_asset / file / file_transfer | 多站点独立配置；OSS（MinIO）；HMAC 图链 | ✅ 全落 |
| E 运维 | task / translator / internal_message×3 / 六类审计查询 / dashboard | SSE 推送；OpenSearch（SearchPosts）选型落地 | ✅ 除 OpenSearch 桥接（SearchPosts 走 SQL 兜底，highlight_fragments 恒空） |
| F 新契约域 | 2026-11 契约批次新增域：AI 全家桶（agent/chat/content/conversation/knowledge/message/provider/query/usage-log）/ notify×5 / flow / webhook / social / monitor-alert / server-monitor / online-session / redis-cache-monitor / config / access-key / backup / task-monitor / tenant-quota / edit-lock / experiment / form / redirect / review-log / search-synonym / email-template / collab-doc-state / mfa | 契约与 BFF 薄代理已就位；运行期 core 答 UNIMPLEMENTED | ⬜ 待逐域落地（当前最长尾） |

### Phase 4 — 收尾

- [ ] 回放回归台架（请求/响应回放比对，归一比较器 + 豁免集）
- [ ] 部署对齐（docker-compose / Dockerfile / 配置 schema 等价）
- [ ] 文档：ARCHITECTURE / 模块新增指南 / 运维手册

---

## 5. 风险登记

| # | 风险 | 缓解 |
|---|------|------|
| R1 | 双 BFF 生成面的自引用改写属构建期字符串操作，生成器升级若改变自引用形态会静默失配 | 升级 rushwind-gen-http rev 后全量重建并跑语料守卫；中期把「模块路径参数化」回馈上游生成器 |
| R2 | OpenSearch 全文（SearchPosts）的等价实现（分词/评分） | 豁免集起步，逐步收敛；必要时经 HTTP API 而非客户端库 |
| R3 | 内容建模元数据驱动的动态字段（section/content_model）无静态 schema | 参照 rushwind-storage-seaorm 的动态 Schema 能力；金样测试钉死 |
| R4 | 多语言翻译（i18n map / GetTranslation）批量子表语义 | 逐端点差分 |

> 本计划为活文档：每阶段收口后回填实际偏差与决策修订。

---

## 6. 会话记录（2026-09-24，Phase 1 + 部分 Phase 3）

- **三服务重构（用户指示）**：落成 core-service（gRPC :6602，独占 PG/Redis，
  tonic）+ admin-api / app-api 薄 BFF。proto crate 同一 prost pass 嵌入 tonic-prost 服务
  生成（`.generate_default_stubs(true)`——trait 方法默认 Unimplemented，逐服务落地）；
  tonic 0.14 的 proto 驱动代码gen在 tonic-prost-build（tonic-build 0.14 已改手工模式）。
- **BFF 代理生成器**（scripts/gen-proxies.py）：从生成的 BFF trait 自动产出 pass-through
  代理（admin 39 / app 8）；签名错位方法登记 STUB_METHODS 桩表（sync_apis 等 7 处）；
  Authentication/AdminPortal/FileTransfer/UserProfile(app) 手写或保持桩。
- **黄金 DDL 管线**：DDL 经 dump-schema 程序导出（约束三遍排序：PK→UNIQUE→FK）；
  id 列补 `DEFAULT nextval`（序列取号等价）；正则表名须含数字
  （`sys_dict_entry_i18n` 教训）。种子=系统种子+演示数据；
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

---

## 7. 状态盘点（2026-11-06，v0.1.0 发布前）

- Phase 0 / 1 / 2 ✅；Phase 3 批次 A / C / D 全落，B 缺 section，E 缺 OpenSearch 桥接；
  批次 F（2026-11 契约新增域）为当前最长尾——契约与 BFF 薄代理就位，core 答 UNIMPLEMENTED。
- 面上规模：core gRPC 注册 42 领域服务；admin BFF 76 服务（薄代理 71 + 手写 5）、
  app BFF 14 服务（薄代理 12）；白名单 admin 10 / app 27 操作；契约 209 文件 / 29 领域。
- 门禁基线：cargo fmt / clippy -D warnings / workspace 101 tests / proto 契约门 /
  五前端快照门全绿（ubuntu + windows CI）；core 空库引导（schema + 种子 + 演示 + 菜单）过。
- 已知问题：① i_user 的 redact.internal_method 与 gen-http plan-builder 的相容性未决
  （上游决策挂起）——BFF REST 面整体 500，e2e 套件在 HEAD 不可复跑（基线为 2026-09-24
  的 44 断言 / 13 模块）；② SearchPosts 的 OpenSearch 桥接未落；③ section 域未落。
