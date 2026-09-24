# 品牌覆写（app 前台前端）

本目录是 `frontend/app/` 快照**唯一被允许的**对上游偏离：

- `overlay/<frontend>/` — logo / favicon 等二进制资产，同步后原样覆盖进快照。
  来源：rushwind 仓的 RushWind 品牌资产（与 rushwind-admin 的 overlay 同源）。
  flutter_app 无二进制覆写——启动器图标在原生工程里，仅做字符串替换。
- `apply-brand.sh` — 声明文件的品牌文案替换表（快照默认品牌字样 → RushWind / 锐风）。

登记的偏离面：

| 前端 | 文件 | 改动 |
|------|------|------|
| react | `public/logo.png` / `public/favicon.ico` | RushWind 标 |
| react | `src/core/preferences/config/default.ts` | 应用名 / 标题 / companyName / companySiteLink |
| react | `src/config/env.ts` / `.env.*` | 标题回退 / 演示域名 |
| react | `src/app/page.tsx` / `src/app/[locale]/post/detail/client-page.tsx` | 页头与 document.title 品牌名 |
| react | `src/store/core/access/store.ts` | appNamespace |
| vue | `public/logo.png` / `public/favicon.ico` | RushWind 标 |
| vue | `app/core/preferences/config/default.ts` | 应用名 / 标题 / companyName / companySiteLink |
| vue | `app/constants/core.ts` / `app/plugins/preferences.ts` | 命名空间与存储键 |
| vue | `nuxt.config.ts` | 标题回退 |
| taro | `src/assets/images/logo.png` / `src/favicon.ico` | RushWind 标 |
| taro | `src/index.html` / `src/app.config.ts` | 标题 |
| taro | `src/core/preferences/config/default.ts` / `src/config/env.ts` | 应用名 / 标题 / companyName |
| taro | `src/store/core/access/store.ts` | storage 键 |
| flutter_app | `lib/l10n/*.arb` + `lib/generated/**` | appName / footerText（arb 源与生成物同步替换） |

设计注释中的「风行」为上游设计语言表述（非用户可见文案），保持上游原样以缩小偏离面。
