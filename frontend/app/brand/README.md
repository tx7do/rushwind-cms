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
| react | `src/app/page.tsx` / `src/app/[locale]/post/detail/client-page.tsx` / `src/app/[locale]/page/detail/client-page.tsx` | 页头与 document.title 品牌名 |
| react | `src/store/core/access/store.ts` | appNamespace |
| react | `messages/{zh-CN,en-US}/app.json` / `authentication.json` / `ui.json` | 应用标题 / 认证页品牌名 / 版权行 |
| react | `messages/{zh-CN,en-US}/page.json` | hero 副标题（Go + React → Rust + React）/ 关于与特性文案 / 联系邮箱域名 |
| vue | `public/logo.png` / `public/favicon.ico` | RushWind 标 |
| vue | `app/core/preferences/config/default.ts` | 应用名 / 标题 / companyName / companySiteLink |
| vue | `app/constants/core.ts` / `app/plugins/preferences.ts` | 命名空间与存储键 |
| vue | `nuxt.config.ts` | 标题回退 |
| vue | `locales/{zh-CN,en-US,ar}/{app,authentication,ui,page}.json` | 应用标题 / 认证页品牌名 / 版权行 / 关于与特性文案 / 联系邮箱域名（2026-11 批次扩张，含新增 ar 语言包） |
| taro | `src/assets/images/logo.png` / `src/favicon.ico` | RushWind 标 |
| taro | `src/index.html` / `src/app.config.ts` | 标题 |
| taro | `src/core/preferences/config/default.ts` / `src/config/env.ts` | 应用名 / 标题 / companyName（含演示配置「风行教育」→「锐风教育」） |
| taro | `src/store/core/access/store.ts` / `config/index.ts` | storage 键 / namespace 回退 |
| taro | `.env.development` / `.env.production` / `project.config.json` / `package.json` | namespace / 标题 / 站点域名 / 工程描述（2026-11 批次） |
| taro | `messages/{zh-CN,en-US}/{app,authentication,ui,page}.json` | 同 react messages 口径（2026-11 批次扩张） |
| flutter_app | `lib/l10n/*.arb` + `lib/generated/**` | appName / footerText / 联系邮箱与官网域名（gowind.dev → rushwind.dev；arb 源与生成物同步替换） |
| flutter_app | `.env` / `web/index.html` / `android/app/src/main/AndroidManifest.xml` / `ios/Runner/Info.plist` | API 站点域 / Web 元信息与标题 / 原生工程显示名（2026-11 批次：上游 27310913 品牌化 Flutter 原生面） |

设计注释中的「风行」为上游设计语言表述（非用户可见文案），保持上游原样以缩小偏离面。
树内冻结文档（各快照的 README / AGENTS.md / docs/*）、部署模板注释与 CSS 设计
token 名为上游工程面，保持上游原样。
