#!/usr/bin/env bash
# e2e-test.sh — 三服务栈的综合端到端测试：认证链 → 各模块读 → CRUD → app 面。
# 前置：core(:6602)/admin(:6600)/app(:6700) 已起，PG/Redis 可达。
set -uo pipefail

ADMIN="http://127.0.0.1:6600"
APP="http://127.0.0.1:6700"
PASS_ENC='h1EAUNtwVm48wiz7iW1sjw=='   # AES("admin")
PASS_ENC2='h1EAUNtwVm48wiz7iW1sjw=='  # AES("admin") same

PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ✓ $1"; }
fail() { FAIL=$((FAIL+1)); echo "  ✗ $1  ==> $2"; }
check() { # name expected actual
  if [[ "$3" == *"$2"* ]]; then ok "$1"; else fail "$1" "$3"; fi
}

redis_cmd() {
python3 - "$@" <<'EOF'
import socket, sys
def cmd(sock, *args):
    buf = '*' + str(len(args)) + '\r\n'
    for a in args:
        b = a.encode(); buf += '$' + str(len(b)) + '\r\n' + a + '\r\n'
    sock.sendall(buf.encode()); return sock.recv(2000).decode(errors='replace')
s = socket.create_connection(('127.0.0.1', 6379), timeout=3)
cmd(s, 'AUTH', '*Abcd123456')
print(cmd(s, *sys.argv[1:]).split('\r\n')[1] if len(sys.argv) > 1 else '')
EOF
}

echo "── 1. 验证码与登录链 ──────────────────────"
CAP=$(curl -s "$ADMIN/admin/v1/captcha")
CID=$(echo "$CAP" | python3 -c "import json,sys; print(json.load(sys.stdin)['captchaId'])")
ANS=$(redis_cmd GET "cms:captcha:$CID")
check "captcha generates (id+png)" '"imageBase64":"data:image/png;base64,' "$CAP"

RESP=$(curl -s -X POST -H "Content-Type: application/json" \
  -H "X-Captcha-Id: $CID" -H "X-Captcha-Value: $ANS" \
  -d "{\"grant_type\":0,\"username\":\"admin\",\"password\":\"$PASS_ENC\"}" \
  "$ADMIN/admin/v1/login")
TOKEN=$(echo "$RESP" | python3 -c "import json,sys; print(json.load(sys.stdin).get('access_token',''))" 2>/dev/null)
check "login returns bearer token" '"token_type":"bearer"' "$RESP"
AUTH="Authorization: Bearer $TOKEN"

# captcha 单次消费
RESP2=$(curl -s -X POST -H "Content-Type: application/json" \
  -H "X-Captcha-Id: $CID" -H "X-Captcha-Value: $ANS" \
  -d "{\"grant_type\":0,\"username\":\"admin\",\"password\":\"$PASS_ENC\"}" \
  "$ADMIN/admin/v1/login")
check "captcha is single-use" 'invalid or missing captcha' "$RESP2"

# 错误密码
CAP=$(curl -s "$ADMIN/admin/v1/captcha"); CID=$(echo "$CAP" | python3 -c "import json,sys; print(json.load(sys.stdin)['captchaId'])"); ANS=$(redis_cmd GET "cms:captcha:$CID")
BAD=$(curl -s -X POST -H "Content-Type: application/json" -H "X-Captcha-Id: $CID" -H "X-Captcha-Value: $ANS" \
  -d '{"grant_type":0,"username":"admin","password":"AAAAAAAAAAAAAAAAAAAAAA=="}' "$ADMIN/admin/v1/login")
check "wrong password → USER_NOT_FOUND/INVALID_PASSWORD" '"reason":"' "$BAD"

echo "── 2. 门控路由 ──────────────────────"
R=$(curl -s -o /dev/null -w "%{http_code}" "$ADMIN/admin/v1/posts")
check "no token → 401" "401" "$R"
R=$(curl -s -H "Authorization: Bearer faketoken" "$ADMIN/admin/v1/posts")
check "fake token → 401 envelope" '"reason":"UNAUTHORIZED"' "$R"

echo "── 3. 领域读（真实种子/演示数据） ──────────────────────"
check "languages" '"languageCode":"zh-CN"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/dict/langs?page=1&pageSize=2")"
check "dict types" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/dict/types?page=1&pageSize=2")"
check "posts (translations)" '"translations"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/posts?page=1&pageSize=2")"
check "categories" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/categories?page=1&pageSize=2")"
check "tags" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/tags?page=1&pageSize=2")"
check "pages" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/pages?page=1&pageSize=2")"
check "comments" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/comments?page=1&pageSize=2")"
check "sites" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/sites?page=1&pageSize=2")"
check "site settings" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/site-settings?page=1&pageSize=2")"
check "navigations" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/navigations?page=1&pageSize=2")"
check "users" '"username":"admin"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/users?page=1&pageSize=2")"
check "roles" '"code":"platform:admin"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/roles?page=1&pageSize=5")"
check "tenants" '"code":"super"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/tenants?page=1&pageSize=5")"

echo "── 4. CRUD 写路径 ──────────────────────"
# create post
NEW=$(curl -s -X POST -H "$AUTH" -H "Content-Type: application/json" -d '{
  "data": {"status": 1, "translations": [{"languageCode":"zh-CN","title":"Rust 端到端测试文章","slug":"e2e-rust-post","content":"hello from e2e"}]}
}' "$ADMIN/admin/v1/posts")
NEW_ID=$(echo "$NEW" | python3 -c "import json,sys; print(json.load(sys.stdin).get('id',0))" 2>/dev/null)
check "post create returns id+translation" '"title":"Rust 端到端测试文章"' "$NEW"

# read it back
GOT=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/posts/$NEW_ID")
check "post get by id" '"slug":"e2e-rust-post"' "$GOT"

# update
UP=$(curl -s -X PUT -H "$AUTH" -H "Content-Type: application/json" -d '{"data":{"isFeatured":true}}' "$ADMIN/admin/v1/posts/$NEW_ID")
check "post update" '"isFeatured":true' "$UP"

# create category
NC=$(curl -s -X POST -H "$AUTH" -H "Content-Type: application/json" -d '{
  "data": {"status": 1, "translations": [{"languageCode":"zh-CN","name":"E2E分类","slug":"e2e-cat"}]}
}' "$ADMIN/admin/v1/categories")
check "category create" '"name":"E2E分类"' "$NC"
CAT_ID=$(echo "$NC" | python3 -c "import json,sys; print(json.load(sys.stdin).get('id',0))" 2>/dev/null)

# create tag
NT=$(curl -s -X POST -H "$AUTH" -H "Content-Type: application/json" -d '{
  "data": {"translations": [{"languageCode":"zh-CN","name":"E2E标签","slug":"e2e-tag"}]}
}' "$ADMIN/admin/v1/tags")
check "tag create" '"name":"E2E标签"' "$NT"

# delete post
DEL=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE -H "$AUTH" "$ADMIN/admin/v1/posts/$NEW_ID")
check "post delete → 200" "200" "$DEL"
DEL2=$(curl -s -o /dev/null -w "%{http_code}" -H "$AUTH" "$ADMIN/admin/v1/posts/$NEW_ID")
check "deleted post get → 404" "404" "$DEL2"
[ -n "$CAT_ID" ] && [ "$CAT_ID" != "0" ] && curl -s -o /dev/null -X DELETE -H "$AUTH" "$ADMIN/admin/v1/categories/$CAT_ID"

echo "── 5. 刷新轮换与登出 ──────────────────────"
JTI=$(python3 -c "
import json,base64
p='$TOKEN'.split('.')[1]; p+='='*(-len(p)%4)
print(json.loads(base64.urlsafe_b64decode(p))['jti'])")
CAP=$(curl -s "$ADMIN/admin/v1/captcha"); CID=$(echo "$CAP" | python3 -c "import json,sys; print(json.load(sys.stdin)['captchaId'])"); ANS=$(redis_cmd GET "cms:captcha:$CID")
RESP=$(curl -s -c /tmp/e2e_cookies -X POST -H "Content-Type: application/json" -H "X-Captcha-Id: $CID" -H "X-Captcha-Value: $ANS" \
  -d "{\"grant_type\":0,\"username\":\"admin\",\"password\":\"$PASS_ENC\"}" "$ADMIN/admin/v1/login")
TOKEN2=$(echo "$RESP" | python3 -c "import json,sys; print(json.load(sys.stdin)['access_token'])")
JTI2=$(python3 -c "
import json,base64
p='$TOKEN2'.split('.')[1]; p+='='*(-len(p)%4)
print(json.loads(base64.urlsafe_b64decode(p))['jti'])")
[ "$JTI" != "$JTI2" ] && ok "login mints fresh jti" || fail "login jti" "same jti"
R=$(curl -s -b /tmp/e2e_cookies -X POST -H "Content-Type: application/json" \
  -d "{\"grant_type\":3,\"user_id\":1,\"jti\":\"$JTI2\"}" "$ADMIN/admin/v1/refresh-token")
TOKEN3=$(echo "$R" | python3 -c "import json,sys; print(json.load(sys.stdin).get('access_token',''))" 2>/dev/null)
[ -n "$TOKEN3" ] && [ "$TOKEN3" != "$TOKEN2" ] && ok "refresh rotates token" || fail "refresh" "$R"
# old token revoked by rotation
R=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $TOKEN2" "$ADMIN/admin/v1/posts")
check "rotated-out token rejected" "401" "$R"
# new token works
R=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $TOKEN3" "$ADMIN/admin/v1/posts")
check "rotated token accepted" "200" "$R"
# logout revokes all
R=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Authorization: Bearer $TOKEN3" -H "Content-Type: application/json" -d '{}' "$ADMIN/admin/v1/logout")
check "logout → 200" "200" "$R"
R=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $TOKEN3" "$ADMIN/admin/v1/posts")
check "post-logout token rejected" "401" "$R"

echo "── 6. app 面（前台） ──────────────────────"
R=$(curl -s -X POST -H "Content-Type: application/json" -d "{\"username\":\"e2e_app_$$\",\"password\":\"$PASS_ENC2\"}" "$APP/app/v1/register")
check "app register" 'userId' "$R"
R=$(curl -s -X POST -H "Content-Type: application/json" -d "{\"grant_type\":0,\"username\":\"e2e_app_$$\",\"password\":\"$PASS_ENC2\"}" "$APP/app/v1/login")
APPTOKEN=$(echo "$R" | python3 -c "import json,sys; print(json.load(sys.stdin).get('access_token',''))" 2>/dev/null)
check "app login (client=1, no captcha)" '"token_type":"bearer"' "$R"
check "app token has 900s expiry" '"expires_in":"900"' "$R"
R=$(curl -s "$APP/app/v1/posts?page=1&pageSize=2")
check "app posts list (public)" '"items"' "$R"
R=$(curl -s "$APP/app/v1/categories?page=1&pageSize=2")
check "app categories (public)" '"items"' "$R"
R=$(curl -s "$APP/app/v1/tags?page=1&pageSize=2")
check "app tags (public)" '"items"' "$R"
R=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Content-Type: application/json" -d '{}' "$APP/app/v1/logout")
check "app logout gated → 401" "401" "$R"
R=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Authorization: Bearer $APPTOKEN" -H "Content-Type: application/json" -d '{}' "$APP/app/v1/logout")
check "app logout with token → 200" "200" "$R"
R=$(curl -s "$APP/app/v1/comments?page=1&pageSize=2")
check "app comments (public)" '"items"' "$R"
R=$(curl -s -X POST -H "Content-Type: application/json" -d '{"data":{"contentType":1,"objectId":1,"content":"e2e 评论","authorName":"游客"}}' "$APP/app/v1/comments")
check "app guest comment create" '"content":"e2e 评论"' "$R"

echo "── 7. 新模块（权限/审计/统计/互动写/杂项） ──────────────────────"
# 上一节登出吊销了管理员全部令牌——重新登录取新令牌
CAP=$(curl -s "$ADMIN/admin/v1/captcha"); CID=$(echo "$CAP" | python3 -c "import json,sys; print(json.load(sys.stdin)['captchaId'])"); ANS=$(redis_cmd GET "cms:captcha:$CID")
RESP=$(curl -s -X POST -H "Content-Type: application/json" -H "X-Captcha-Id: $CID" -H "X-Captcha-Value: $ANS" \
  -d "{\"grant_type\":0,\"username\":\"admin\",\"password\":\"$PASS_ENC\"}" "$ADMIN/admin/v1/login")
TOKEN=$(echo "$RESP" | python3 -c "import json,sys; print(json.load(sys.stdin).get('access_token',''))")
AUTH="Authorization: Bearer $TOKEN"
check "menus list" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/menus?page=1&pageSize=3")"
check "apis list" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/apis?page=1&pageSize=3")"
check "permission groups" '"name":"系统管理"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/permission-groups?page=1&pageSize=5")"
check "permissions" '"code":"sys:access_backend"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/permissions?page=1&pageSize=10")"
check "api audit logs" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/api-audit-logs?page=1&pageSize=2")"
check "login audit logs" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/login-audit-logs?page=1&pageSize=2")"
check "operation audit logs" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/operation-audit-logs?page=1&pageSize=2")"
check "data access audit logs" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/data-access-audit-logs?page=1&pageSize=2")"
check "org units" '"name":"XX集团总部"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/org-units?page=1&pageSize=3")"
check "positions" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/positions?page=1&pageSize=3")"
check "login policies" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/login-policies?page=1&pageSize=3")"
check "content models" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/content-models?page=1&pageSize=3")"
check "media assets" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/media-assets?page=1&pageSize=3")"
check "tasks" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/tasks?page=1&pageSize=3")"
check "internal messages" '"items"' "$(curl -s -H "$AUTH" "$ADMIN/admin/v1/internal-message/messages?page=1&pageSize=3")"

# dashboard overview (stats)
STATS=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/stats/overview")
check "dashboard overview counts" '"userCount":"' "$STATS"
check "dashboard overview has posts" '"postCount":"' "$STATS"
TREND=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/stats/content-trend?days=7")
check "content trend (7d)" '"users"' "$TREND"
ISTATS=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/stats/interactions?topN=5")
check "interaction stats" '"totalLikes"' "$ISTATS"

# interaction write path: like → unlike via app token
APPTOKEN2=$(curl -s -X POST -H "Content-Type: application/json" -d "{\"grant_type\":0,\"username\":\"e2e_app_$$\",\"password\":\"$PASS_ENC2\"}" "$APP/app/v1/login" | python3 -c "import json,sys; print(json.load(sys.stdin).get('access_token',''))")
LIKE=$(curl -s -X POST -H "Authorization: Bearer $APPTOKEN2" -H "Content-Type: application/json" -d '{"targetType":1,"targetId":1}' "$APP/app/v1/interactions/like")
check "post like (returns liked+count)" '"liked":true' "$LIKE"
LIKE2=$(curl -s -X POST -H "Authorization: Bearer $APPTOKEN2" -H "Content-Type: application/json" -d '{"targetType":1,"targetId":1}' "$APP/app/v1/interactions/like")
check "like idempotent" '"liked":true' "$LIKE2"
UNLIKE=$(curl -s -X POST -H "Authorization: Bearer $APPTOKEN2" -H "Content-Type: application/json" -d '{"targetType":1,"targetId":1}' "$APP/app/v1/interactions/unlike")
check "post unlike" '"liked":false' "$UNLIKE"
WATCH=$(curl -s -X POST -H "Authorization: Bearer $APPTOKEN2" -H "Content-Type: application/json" -d '{"postId":1}' "$APP/app/v1/interactions/watch")
check "post watch" '"watched":true' "$WATCH"
UNWATCH=$(curl -s -X POST -H "Authorization: Bearer $APPTOKEN2" -H "Content-Type: application/json" -d '{"postId":1}' "$APP/app/v1/interactions/unwatch")
check "post unwatch" '"watched":false' "$UNWATCH"
R=$(curl -s -X POST -H "Content-Type: application/json" -d '{"targetType":1,"targetId":1}' "$APP/app/v1/interactions/like")
check "like without token → 401" '401' "$(curl -s -o /dev/null -w '%{http_code}' -X POST -H 'Content-Type: application/json' -d '{"targetType":1,"targetId":1}' "$APP/app/v1/interactions/like")"

echo "── 8. 写路径与聚合面（本轮新增） ──────────────────────"
# 用户 CRUD
UNIQ=$$
NEWU=$(curl -s -X POST -H "$AUTH" -H "Content-Type: application/json" -d "{\"data\":{\"username\":\"e2e_user_$UNIQ\",\"nickname\":\"端到端用户\",\"email\":\"uc$UNIQ@example.com\"},\"password\":\"Passw0rd!123\"}" "$ADMIN/admin/v1/users")
U_ID=$(echo "$NEWU" | python3 -c "import json,sys; print(json.load(sys.stdin).get('id',0))" 2>/dev/null)
check "user create returns id" "\"username\":\"e2e_user_$UNIQ\"" "$NEWU"
GOTU=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/users/$U_ID")
check "user get by id" '"nickname":"端到端用户"' "$GOTU"
UPU=$(curl -s -X PUT -H "$AUTH" -H "Content-Type: application/json" -d '{"data":{"nickname":"端到端用户改"}}' "$ADMIN/admin/v1/users/$U_ID")
GOTU2=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/users/$U_ID")
check "user update" '"nickname":"端到端用户改"' "$GOTU2"
EXISTS=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/users:exists?id=${U_ID//[!0-9]/}")
check "user exists" '"exist":true' "$EXISTS"
# 新用户登录（凭证已建）
NULOGIN=$(curl -s -X POST -H "Content-Type: application/json" -d "{\"grant_type\":0,\"username\":\"e2e_user_$UNIQ\",\"password\":\"oIWFVq71OW82um7L6YOsxw==\"}" "$APP/app/v1/login")
check "new user can login (credential works)" '"token_type":"bearer"' "$NULOGIN"

DEL=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE -H "$AUTH" "$ADMIN/admin/v1/users/$U_ID")
check "user delete" "200" "$DEL"
GONE=$(curl -s -o /dev/null -w "%{http_code}" -H "$AUTH" "$ADMIN/admin/v1/users/$U_ID")
check "deleted user 404" "404" "$GONE"

# 角色写
NEWR=$(curl -s -X POST -H "$AUTH" -H "Content-Type: application/json" -d '{"data":{"name":"E2E角色","code":"e2e:role","permissions":[1]}}' "$ADMIN/admin/v1/roles")
R_ID=$(echo "$NEWR" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('id',0))" 2>/dev/null)
R_BODY=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/roles")
check "role create+list" '"code":"e2e:role"' "$R_BODY"
[ -n "$R_ID" ] && [ "$R_ID" != "0" ] && curl -s -o /dev/null -X DELETE -H "$AUTH" "$ADMIN/admin/v1/roles/$R_ID"

# 租户写
NEWT=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "$AUTH" -H "Content-Type: application/json" -d "{\"data\":{\"name\":\"E2E租户$UNIQ\",\"code\":\"e2e-tenant-$UNIQ\"}}" "$ADMIN/admin/v1/tenants")
check "tenant create → 200" "200" "$NEWT"
TLIST=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/tenants?page=1&pageSize=50")
check "tenant created visible" "\"code\":\"e2e-tenant-$UNIQ\"" "$TLIST"
T_ID=$(echo "$TLIST" | python3 -c "
import json,sys
d=json.load(sys.stdin)
print(next((i.get('id',0) for i in d.get('items',[]) if i.get('code')=='e2e-tenant-$UNIQ'),0))" 2>/dev/null)
[ -n "$T_ID" ] && [ "$T_ID" != "0" ] && curl -s -o /dev/null -X DELETE -H "$AUTH" "$ADMIN/admin/v1/tenants/$T_ID"

# 聚合面
NAV=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/routes")
check "portal navigation (routes)" '"items"' "$NAV"
CODES=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/perm-codes")
check "portal perm codes" '"sys:access_backend"' "$CODES"
CTX=$(curl -s -H "$AUTH" "$ADMIN/admin/v1/initial-context")
check "portal initial context" '"permissions"' "$CTX"

# app 面 user profile
APPUID=$(curl -s -H "Authorization: Bearer $APPTOKEN2" "$APP/app/v1/me" 2>/dev/null)
check "app user profile" '"username"' "$APPUID"

echo "── 9. SSE 与 CORS ──────────────────────"
R=$(curl -s -o /dev/null -w "%{http_code}" -X OPTIONS "http://127.0.0.1:6601/events")
check "SSE preflight 204" "204" "$R"
R=$(curl -s -i -X OPTIONS -H "Origin: http://localhost:5999" -H "Access-Control-Request-Method: POST" "$ADMIN/admin/v1/login" 2>&1 | grep -i "access-control-allow-origin" | head -1)
check "CORS allowed origin" "http://localhost:5999" "$R"

echo ""
echo "═══════ 结果: PASS=$PASS FAIL=$FAIL ═══════"
exit $((FAIL > 0 ? 1 : 0))
