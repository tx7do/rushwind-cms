# frontend/

前端与后端是零改动兼容契约：API 基址指向本仓后端（admin 前端 → REST :6600，
前台四套 → REST :6700）即可，无需改动任何前端代码。

五个前端均随本仓同步，同步模型三者一致：

```text
sync-frontend.sh <name>           # 上游整树同步 → RushWind 品牌覆写 → 重建双清单
sync-frontend.sh <name> --check   # CI 门：快照终态与 MANIFEST 逐字节一致（防手改）
```

- **管理后台**：`frontend/admin/sync-frontend.sh admin-react`（源：
  go-wind-cms `frontend/admin-react`，`ADMIN_REACT_FRONTEND_SRC` 覆盖）
- **前台四套**：`frontend/app/sync-frontend.sh <react|vue|taro|flutter_app>`
  （源：go-wind-cms `frontend/app/<name>`，`<大写名>_FRONTEND_SRC` 覆盖）

**快照唯一被允许的对上游偏离是 RushWind 品牌覆写**（各 `brand/` 目录：
overlay 二进制资产 + apply-brand.sh 文案替换表，登记于 brand/README.md）。
上游变更必须以整树重同步的方式显式接受（重建双清单并跑差分回归）。

| 前端 | 目录 | dev 端口 | 启动 |
|------|------|---------|------|
| 管理后台 React（Vite + AntD） | `admin/admin-react` | 5999 | `pnpm install && pnpm dev` |
| 前台 React（Next.js + shadcn/ui） | `app/react` | 5001 | `pnpm install && pnpm dev` |
| 前台 Vue（Nuxt + shadcn-vue） | `app/vue` | — | `pnpm install && pnpm dev` |
| 前台 Taro（小程序/H5） | `app/taro` | — | 见目录 README |
| 前台 Flutter | `app/flutter_app` | — | `flutter run` |

各前端的详细说明见其目录内的 README / AGENTS.md。
