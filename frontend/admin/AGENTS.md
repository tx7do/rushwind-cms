# AGENTS.md — 管理后台快照层开发指南

> 本文件覆盖 `frontend/admin/` 目录的**快照层**纪律（同步 / 品牌覆写 / 双清单门）。`admin-react/` 冻结树内部是源仓自带的开发规范文档（README / AGENTS.md）——读它了解脚手架约定，但**任何改动都不落地在快照内**。monorepo 全局约定见 [../../AGENTS.md](../../AGENTS.md)。

## 快照模型

```
sync-frontend.sh admin-react           # 上游整树同步 → RushWind 品牌覆写 → 重建双清单
sync-frontend.sh admin-react --check   # CI 门：快照终态与 MANIFEST 逐字节一致（防手改，退出码 1）；
                                       # 源仓在时另对比 UPSTREAM 清单做漂移报告（退出码 2，CI 自跳过）
```

- **双清单**：`admin-react.MANIFEST.sha256`（快照终态门，CI 逐字节比对）+ `admin-react.UPSTREAM.sha256`（源仓漂移基线）。
- **字节保真**：不做归一化、不做 EOL 转换（`.gitattributes` 对快照与清单钉了 `-text`）；清单排序钉死 `LC_ALL=C`。**脚本必须用 bash 执行。**
- **哈希宇宙** = 树上现有文件 − git 忽略路径：dev server 落在快照内的构建缓存（`.vite/` 等）不进清单，属正常，勿手删。
- **快照唯一被允许的对上游偏离是 RushWind 品牌覆写**（`brand/`）。除此之外任何手改都视为契约破坏；上游变更必须以**整树重同步**的方式显式接受（重建双清单并跑差分回归）。

## 技术栈（冻结树内，参考内层 AGENTS.md）

React 19 + TypeScript + Vite 8（SWC）+ Ant Design v6 + ProComponents + TanStack React Query + Zustand + TipTap；pnpm。

## dev 启动

| 项 | 值 |
|---|---|
| 目录 | `frontend/admin/admin-react/` |
| 端口 | 5999（`.env.development` 的 `VITE_SERVER_PORT`） |
| API | dev 代理 `VITE_PROXY='[["/admin", "http://127.0.0.1:6600/"]]'` → admin-api REST :6600 |
| 要求 | Node ≥ 20.19（Vite 8）；曾用低版本 Node 装过依赖的，用新版 Node `pnpm install --force` |
| CORS | admin-api 的 `assets/server.yaml` 默认放行 `localhost:5999` / `127.0.0.1:5999`，换端口要同步改 |

登录链路：密码 AES 加密传输（`VITE_AES_KEY`），登录带图形验证码（Redis 存储）。

## 品牌覆写（唯一允许的偏离）

`brand/` 三件套：`overlay/admin-react/`（logo / favicon 等二进制资产，同步后原样覆盖）+ `apply-brand.sh`（文案替换表，逐条显式声明，不做全局盲替换）+ `README.md`（**偏离面登记表**）。

**新增一条品牌覆写的完整动作**：

1. 改 `brand/apply-brand.sh` 的替换表（或往 `overlay/` 加资产）；
2. 在 `brand/README.md` 登记表加一行（文件 + 改动）；
3. 重跑 `sync-frontend.sh admin-react` 重建双清单；
4. `git diff` 复核：与上一版的差异**必须恰好**是本轮登记的文件哈希——多一个哈希即混入了未登记偏离，回退排查；
5. 跑 `sync-frontend.sh admin-react --check` 确认门绿。

**设计注释政策**：源码注释里的上游设计语言表述（如「风行」）非用户可见文案，保持上游原样以缩小偏离面——不在覆写范围。

## 常见操作

| 操作 | 动作 |
|---|---|
| 接受上游新 vintage | `sync-frontend.sh admin-react` 整树重同步（自动品牌覆写 + 重建双清单）→ `git diff` 审查漂移 → 前后端联调 + E2E 回归 |
| `--check` 挂了 | 先看是否本地构建缓存混入清单（哈希宇宙按 gitignore 剪枝，bash 语义）；再 diff 快照树找未登记手改；确认是上游漂移则走整树重同步接受 |
| 改品牌文案 | 只经 `apply-brand.sh` 替换表 + 登记表 + 重钉清单，**不直接改快照内文件** |
