# 品牌覆写（admin 前端）

本目录是 `frontend/admin/` 快照**唯一被允许的**对上游偏离：

- `overlay/admin-react/` — logo / favicon 等二进制资产，同步后原样覆盖进快照。
  来源：rushwind 仓的 RushWind 品牌资产（与 rushwind-admin 的 overlay 同源）。
- `apply-brand.sh` — 声明文件的品牌文案替换表（GoWind / 风行 → RushWind / 锐风）。

登记的偏离面（admin-react）：

| 文件 | 改动 |
|------|------|
| `public/logo.png` / `public/favicon.ico` | RushWind 标 |
| `src/core/preferences/config/default.ts` | companyName / companySiteLink |
| `src/locales/{zh-CN,en-US}/_core/auth.json` | 系统标题（zh）/ 版权行 |
| `index.html` | meta keywords / author |
| `.env` / `.env.production` | 应用标题 / 命名空间 / 演示域名 |

设计注释中的「风行」为上游设计语言表述（非用户可见文案），保持上游原样以缩小偏离面。
