# AGENTS.md — 前台快照层开发指南

> 本文件覆盖 `frontend/app/` 目录的**快照层**纪律（同步 / 品牌覆写 / 双清单门），管四套 C 端快照。各冻结树内部是源仓自带的开发规范文档（README / AGENTS.md）——读它了解各脚手架约定，但**任何改动都不落地在快照内**。monorepo 全局约定见 [../../AGENTS.md](../../AGENTS.md)；管理后台快照层见 [../admin/AGENTS.md](../admin/AGENTS.md)。

## 快照模型（四套同构）

```
sync-frontend.sh <react|vue|taro|flutter_app>           # 上游整树同步 → 品牌覆写 → 重建双清单
sync-frontend.sh <react|vue|taro|flutter_app> --check   # CI 门：快照终态与 MANIFEST 逐字节一致（退出码 1）；
                                                        # 源仓漂移报告（退出码 2，CI 无源仓自跳过）
```

- **双清单**：`<name>.MANIFEST.sha256`（终态门）+ `<name>.UPSTREAM.sha256`（漂移基线），每套一份。
- **字节保真**：无归一化、无 EOL 转换（`.gitattributes` 钉 `-text`）；清单排序钉死 `LC_ALL=C`；**脚本必须用 bash 执行**。
- **快照唯一被允许的对上游偏离是 RushWind 品牌覆写**（`brand/`）；上游变更必须以整树重同步的方式显式接受（重建双清单 + 差分回归）。

## 四套快照

| 快照 | 技术栈 | dev 端口 | 后端 |
|---|---|---|---|
| `react/` | Next.js 16（静态导出型）+ React 19 + Tailwind v4 + shadcn/ui | 5001 | app-api REST :6700 |
| `vue/` | Nuxt + shadcn-vue | — | app-api REST :6700 |
| `taro/` | Taro（微信小程序 / H5） | — | app-api REST :6700 |
| `flutter_app/` | Flutter | — | app-api REST :6700 |

各内层规范：react / vue / taro / flutter_app 的冻结树内均带源仓 AGENTS.md（脚手架开发规范，只读参考）。

## dev 启动（react 为例）

- `pnpm install && pnpm dev`（端口 5001；API 基址指向 app-api :6700）。
- **react 用 `next dev --webpack`**：turbopack 依赖 inotify 配额，本机配额常被 IDE 占满而直接 500；webpack 模式不吃配额。
- Node ≥ 20.9（Next 16 要求）；曾用低版本 Node 装过依赖的，用新版 Node `pnpm install --force`。
- 快照树内的 `.next/` 构建缓存已 gitignore、不进清单，属正常。

## 品牌覆写（唯一允许的偏离）

`brand/` 三件套：`overlay/<frontend>/`（logo / favicon 等二进制资产；flutter_app 无二进制覆写——启动器图标在原生工程里，仅做字符串替换）+ `apply-brand.sh`（文案替换表，逐条显式声明，不做全局盲替换）+ `README.md`（**偏离面登记表**，四套的已登记偏离逐行列明）。

**新增一条品牌覆写的完整动作**：

1. 改 `brand/apply-brand.sh` 对应前端的替换表（或往 `overlay/<frontend>/` 加资产）；
2. 在 `brand/README.md` 登记表加一行（前端 + 文件 + 改动）；
3. 重跑该前端的 `sync-frontend.sh <name>` 重建双清单；
4. `git diff` 复核：与上一版的差异**必须恰好**是本轮登记的文件哈希——多一个哈希即混入了未登记偏离，回退排查；
5. 跑 `sync-frontend.sh <name> --check` 确认门绿。

**i18n messages 层与源码同权重**：应用标题 / hero 文案 / 联系邮箱等用户可见品牌字样散在 `messages/*.json`（react）或各端等价物里——这些同样走替换表登记，不直接手改。

## 常见操作

| 操作 | 动作 |
|---|---|
| 接受上游新 vintage | 对应前端 `sync-frontend.sh <name>` 整树重同步 → `git diff` 审查漂移 → 与 app-api 联调 + E2E 回归 |
| `--check` 挂了 | 先看是否本地构建缓存混入清单；再 diff 快照树找未登记手改；确认是上游漂移则走整树重同步接受 |
| 改品牌文案 | 只经 `apply-brand.sh` 替换表 + 登记表 + 重钉清单，**不直接改快照内文件** |
