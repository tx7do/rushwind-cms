#!/usr/bin/env bash
# apply-brand.sh — 在 sync-frontend.sh 同步完上游后，为快照打 RushWind 品牌覆写：
#   1. overlay/<frontend>/ 树原样覆盖到快照（logo / favicon）
#   2. 对声明文件做品牌文案替换（GoWind / 风行 → RushWind / 锐风）
# 品牌覆写是快照**唯一被允许的**对上游偏离：全部改动必须收在本目录内，
# 新增偏离时同步更新 overlay/<frontend>/ 与下方替换表，并在 brand/README.md 登记。
set -euo pipefail

FRONTEND="${1:?usage: apply-brand.sh <admin-react> <snapshot-dir>}"
DST="${2:?usage: apply-brand.sh <admin-react> <snapshot-dir>}"
HERE="$(cd "$(dirname "$0")" && pwd)"
OVERLAY="$HERE/overlay/$FRONTEND"

# 1) overlay 覆盖
if [[ ! -d "$OVERLAY" ]]; then
  echo "ERROR: no brand overlay for frontend '$FRONTEND': $OVERLAY" >&2
  exit 1
fi
(cd "$OVERLAY" && find . -type f -print0) | while IFS= read -r -d '' f; do
  rel="${f#./}"
  mkdir -p "$DST/$(dirname "$rel")"
  cp "$OVERLAY/$rel" "$DST/$rel"
done

# 2) 品牌文案替换（逐条显式声明，不做全局盲替换；设计注释中的「风行」为
#    上游设计语言表述，非用户可见文案，保持上游原样以缩小偏离面）
replace() {
  local file="$1"
  shift
  sed -i "$@" "$DST/$file"
}

case "$FRONTEND" in
  admin-react)
    replace src/core/preferences/config/default.ts \
      -e 's/companyName: "GoWind"/companyName: "RushWind"/' \
      -e 's|companySiteLink: "https://www.gowind.cloud"|companySiteLink: "https://github.com/tx7do/rushwind"|'

    replace src/locales/zh-CN/_core/auth.json \
      -e 's/风行CMS后台管理系统/锐风CMS后台管理系统/' \
      -e 's/Copyright © {year} GoWind/Copyright © {year} RushWind/'

    replace src/locales/en-US/_core/auth.json \
      -e 's/Copyright © {year} GoWind/Copyright © {year} RushWind/'

    replace index.html \
      -e 's/content="GoWind CMS React AntD Vite"/content="RushWind CMS React AntD Vite"/' \
      -e 's/name="author" content="GoWind"/name="author" content="RushWind"/'

    replace .env \
      -e 's/VITE_APP_TITLE="GoWind CMS Admin"/VITE_APP_TITLE="RushWind CMS Admin"/' \
      -e 's/VITE_APP_NAMESPACE="gowind-cms"/VITE_APP_NAMESPACE="rushwind-cms"/'

    replace .env.production \
      -e 's|api.demo.admin.gowind.cloud|api.demo.admin.rushwind.cloud|' \
      -e 's|sse.demo.admin.gowind.cloud|sse.demo.admin.rushwind.cloud|'
    ;;
  *)
    echo "ERROR: no brand rules for frontend '$FRONTEND'" >&2
    exit 1
    ;;
esac

echo "brand overlay applied ($FRONTEND) -> $DST"
