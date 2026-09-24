#!/usr/bin/env bash
# apply-brand.sh — 在 sync-frontend.sh 同步完上游后，为前台快照打 RushWind 品牌覆写：
#   1. overlay/<frontend>/ 树原样覆盖到快照（logo / favicon）
#   2. 对声明文件做品牌文案替换（GoWind / 风行 → RushWind / 锐风）
# 品牌覆写是快照**唯一被允许的**对上游偏离：全部改动必须收在本目录内，
# 新增偏离时同步更新 overlay/<frontend>/ 与下方替换表，并在 brand/README.md 登记。
set -euo pipefail

FRONTEND="${1:?usage: apply-brand.sh <react|vue|taro|flutter_app> <snapshot-dir>}"
DST="${2:?usage: apply-brand.sh <react|vue|taro|flutter_app> <snapshot-dir>}"
HERE="$(cd "$(dirname "$0")" && pwd)"
OVERLAY="$HERE/overlay/$FRONTEND"

# 1) overlay 覆盖（flutter_app 无二进制覆写——启动器图标在原生工程里，字符串走替换表）
if [[ -d "$OVERLAY" ]]; then
  (cd "$OVERLAY" && find . -type f -print0) | while IFS= read -r -d '' f; do
    rel="${f#./}"
    mkdir -p "$DST/$(dirname "$rel")"
    cp "$OVERLAY/$rel" "$DST/$rel"
  done
fi

# 2) 品牌文案替换（逐条显式声明，不做全局盲替换；设计注释中的「风行」为
#    上游设计语言表述，非用户可见文案，保持上游原样以缩小偏离面）
replace() {
  local file="$1"
  shift
  sed -i "$@" "$DST/$file"
}

case "$FRONTEND" in
  react)
    replace src/core/preferences/config/default.ts \
      -e "s/name: 'GoWind CMS'/name: 'RushWind CMS'/" \
      -e "s/title: 'GoWind Content Hub'/title: 'RushWind Content Hub'/" \
      -e "s/companyName: 'GoWind'/companyName: 'RushWind'/" \
      -e "s|companySiteLink: 'https://www.gowind.cloud'|companySiteLink: 'https://github.com/tx7do/rushwind'|"

    replace src/config/env.ts \
      -e "s/'GoWind Content Hub'/'RushWind Content Hub'/"

    replace src/app/page.tsx \
      -e 's/GoWind CMS/RushWind CMS/'

    replace 'src/app/[locale]/post/detail/client-page.tsx' \
      -e 's/GoWind Content Hub/RushWind Content Hub/'

    replace src/store/core/access/store.ts \
      -e "s/const appNamespace = 'gowind'/const appNamespace = 'rushwind'/"

    replace .env.development \
      -e "s/NEXT_PUBLIC_APP_TITLE='GoWind Content Hub'/NEXT_PUBLIC_APP_TITLE='RushWind Content Hub'/"

    replace .env.production \
      -e "s/NEXT_PUBLIC_APP_TITLE='GoWind Content Hub'/NEXT_PUBLIC_APP_TITLE='RushWind Content Hub'/" \
      -e 's|api.cms.gowind.cloud|api.cms.rushwind.cloud|'
    ;;

  vue)
    replace app/core/preferences/config/default.ts \
      -e 's/name: "GoWind CMS"/name: "RushWind CMS"/' \
      -e 's/title: "GoWind Content Hub"/title: "RushWind Content Hub"/' \
      -e 's/companyName: "GoWind"/companyName: "RushWind"/' \
      -e 's|companySiteLink: "https://www.gowind.cloud"|companySiteLink: "https://github.com/tx7do/rushwind"|'

    replace app/constants/core.ts \
      -e "s/export const APP_NAMESPACE = 'gowind'/export const APP_NAMESPACE = 'rushwind'/"

    replace app/plugins/preferences.ts \
      -e "s/'gowind-cms:preferences'/'rushwind-cms:preferences'/" \
      -e "s/'gowind-cms:app-locale'/'rushwind-cms:app-locale'/" \
      -e "s/namespace: 'gowind-cms'/namespace: 'rushwind-cms'/"

    replace nuxt.config.ts \
      -e "s/'GoWind Content Hub'/'RushWind Content Hub'/"
    ;;

  taro)
    replace src/index.html \
      -e 's/<title>GoWind CMS<\/title>/<title>RushWind CMS<\/title>/'

    replace src/app.config.ts \
      -e "s/navigationBarTitleText: 'GoWind CMS'/navigationBarTitleText: 'RushWind CMS'/"

    replace src/core/preferences/config/default.ts \
      -e "s/name: 'GoWind CMS'/name: 'RushWind CMS'/" \
      -e "s/title: 'GoWind Content Hub'/title: 'RushWind Content Hub'/" \
      -e "s/companyName: 'GoWind'/companyName: 'RushWind'/" \
      -e "s|companySiteLink: 'https://www.gowind.cloud'|companySiteLink: 'https://github.com/tx7do/rushwind'|"

    replace src/config/env.ts \
      -e "s/'GoWind Content Hub'/'RushWind Content Hub'/"

    replace src/store/core/access/store.ts \
      -e "s/name: 'gowind-access-storage'/name: 'rushwind-access-storage'/"
    ;;

  flutter_app)
    # l10n 源（arb）与生成物（generated/）都随快照字节冻结——两边同步替换，
    # 保持 flutter gen-l10n 语义一致。
    replace lib/l10n/intl_zh_CN.arb \
      -e 's/"appName": "GoWind CMS"/"appName": "RushWind CMS"/' \
      -e 's/© 2026 GoWind CMS/© 2026 RushWind CMS/'

    replace lib/l10n/intl_en_US.arb \
      -e 's/"appName": "GoWind CMS"/"appName": "RushWind CMS"/' \
      -e 's/© 2026 GoWind CMS/© 2026 RushWind CMS/'

    replace lib/generated/l10n.dart \
      -e 's/GoWind CMS/RushWind CMS/'

    replace lib/generated/intl/messages_en_US.dart \
      -e 's/GoWind CMS/RushWind CMS/'

    replace lib/generated/intl/messages_zh_CN.dart \
      -e 's/GoWind CMS/RushWind CMS/'
    ;;

  *)
    echo "ERROR: no brand rules for frontend '$FRONTEND'" >&2
    exit 1
    ;;
esac

echo "brand overlay applied ($FRONTEND) -> $DST"
