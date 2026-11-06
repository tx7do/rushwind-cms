#!/usr/bin/env python3
# gen-proxies.py — 从生成的 BFF trait（OUT_DIR 的 *_gen.rs services 模块）产出
# 瘦 BFF 代理实现：每个 trait 方法 → 同名领域 gRPC 客户端调用。
#
# 契约保证：BFF trait 方法与领域 gRPC 服务方法同名同型（BFF proto 直接引用
# 领域消息），转发是零映射 pass-through。
#
# 跳过（手工实现或保持桩）：
#   AuthenticationService  — 验证码/cookie 等 BFF 关注点（手写）
#   AdminPortalService    — BFF 聚合面，无领域对应（手写）
#   FileTransferService   — multipart 流式传输（手写/桩）
#
# 用法：
#   python3 scripts/gen-proxies.py <admin_gen.rs> admin > services/admin-api/src/services/proxies.rs
#   python3 scripts/gen-proxies.py <app_gen.rs>   app  > services/app-api/src/services/proxies.rs
import re
import sys

# BFF 服务名 → 领域包（与 api/protos 的领域服务声明一致）
PACKAGE = {
    "LoginPolicy": "authentication",
    "UserCredential": "authentication",
    "Language": "dict",
    "DictEntry": "dict",
    "DictType": "dict",
    "User": "identity",
    "UserProfile": "identity",
    "OrgUnit": "identity",
    "Position": "identity",
    "Tenant": "identity",
    "Role": "permission",
    "Api": "permission",
    "Menu": "permission",
    "Permission": "permission",
    "PermissionGroup": "permission",
    "PolicyEvaluationLog": "permission",
    "ApiAuditLog": "audit",
    "LoginAuditLog": "audit",
    "OperationAuditLog": "audit",
    "DataAccessAuditLog": "audit",
    "PermissionAuditLog": "audit",
    "File": "storage",
    "Task": "task",
    "Translator": "translator",
    "Stats": "stats",
    "Post": "content",
    "Category": "content",
    "Tag": "content",
    "Page": "content",
    "ContentModel": "content",
    "Comment": "comment",
    "Interaction": "interaction",
    "InteractionAdmin": "interaction",
    "Site": "site",
    "SiteSetting": "site",
    "Navigation": "site",
    "NavigationItem": "site",
    "MediaAsset": "media",
    "InternalMessage": "internal_message",
    "InternalMessageCategory": "internal_message",
    "InternalMessageRecipient": "internal_message",
    # ── 2026-11 契约批次新增域 ──
    "AccessKey": "access_key",
    "AiAgent": "ai",
    "AiChat": "ai",
    "AiContent": "ai",
    "AiConversation": "ai",
    "AiKnowledgeBase": "ai",
    "AiMessage": "ai",
    "AiProvider": "ai",
    "AiQuery": "ai",
    "AiUsageLog": "ai",
    "Backup": "task",
    "Config": "config",
    "EditLock": "content",
    "Experiment": "content",
    "FlowRule": "flow",
    "FormSubmission": "content",
    "MFA": "authentication",
    "MonitorAlert": "monitor_alert",
    "Notification": "notify",
    "NotificationChannel": "notify",
    "NotificationPreference": "notify",
    "NotificationRule": "notify",
    "NotificationTemplate": "notify",
    "OnlineSession": "online_session",
    "Redirect": "content",
    "RedisCacheMonitor": "redis_cache",
    "ReviewLog": "content",
    "SearchSynonym": "content",
    "ServerMonitor": "server_monitor",
    "Social": "social",
    "TenantQuota": "identity",
    "Webhook": "webhook",
}

# 领域 client 名与 BFF 服务名错位的覆写：(模块, 类型)。缺省按
# snake(service)_service_client + {service}ServiceClient 派生。
DOMAIN_CLIENT = {
    # i_backup 的领域对应是 task 域的 BackupArchiveService。
    "Backup": ("backup_archive_service_client", "BackupArchiveServiceClient"),
    # 缩略词归一：tonic 把 MFAService 的 client 命名为 MfaServiceClient。
    "MFA": ("mfa_service_client", "MfaServiceClient"),
}

# snake 派生对连续大写词（缩略词）会把每个字母断开（MFA → m_f_a），领域
# client 模块是整词 mfa——逐个钉正。
SNAKE_OVERRIDE = {"MFA": "mfa"}

# BFF 方法名与域 RPC 名漂移（同服务 client、消息同源）的改名转发：
# (服务, BFF 方法) → 域 RPC 方法名。生成 `pass <bff> => <domain>`。
RENAMED_METHODS = {
    # i_mfa 面方法名的 m_f_a 蛇形（gen-http 对 MFA 缩略词的切法）↔
    # tonic 域 client 的整词 mfa。verify_m_f_a_challenge 除外——响应
    # 形状错位（BFF LoginResponse ↔ 域 VerifyMFAChallengeResponse），
    # 走 behaviors 的逐字段映射（TokenType 钉 bearer，同上游 Go BFF）。
    ("MFA", "get_m_f_a_status"): "get_mfa_status",
    ("MFA", "disable_m_f_a"): "disable_mfa",
    ("MFA", "reset_user_m_f_a"): "reset_user_mfa",
    # i_online_session 面带 OnlineSession 后缀的方法名 ↔ 域短名。
    ("OnlineSession", "list_online_session"): "list",
    ("OnlineSession", "force_logout_session"): "force_logout",
    ("OnlineSession", "list_my_online_session"): "list_my",
    ("OnlineSession", "revoke_my_online_session"): "revoke_my",
}

# 手工实现的面（不生成）
SKIP = {
    "Authentication",
    "AdminPortal",
    "FileTransfer",
    # 契约仅 admin 面声明、领域尚无对应服务（随特性批次落域后转正）。
    "ComponentDefinition",
    "Preview",
}
# 面特定的额外跳过（BFF 方法集超出领域面）
SKIP_BY_FACE = {"app": set(), "admin": set()}

# BFF/领域签名错位的方法（非零映射转发）——生成桩，待手写适配：
#   ApiService.SyncApis：BFF Empty ↔ 领域 SyncApisRequest
STUB_METHODS = {
    ("Api", "get_walk_route_data"),
    ("File", "create"),
    ("Menu", "sync_menus"),
    ("Permission", "sync_permissions"),
    # ── 2026-11 契约批次：BFF 聚合方法，域无对应 RPC（随特性批次落
    #    behaviors 手写面）──
    # GetBySlug：BFF 本地消息（app 包），SEO 路由数据源在域侧尚无 RPC。
    ("Post", "get_by_slug"),
    ("Page", "get_by_slug"),
    # ListEnabled：BFF ListRedirectsRequest ↔ 域 List 只收 PagingRequest，
    # 启用位过滤语义须手写。
    ("Redirect", "list_enabled"),
    # Chat：域侧 server-streaming（ChatStreamFrame），BFF 面按 unary
    # ChatResponse 建模——流式桥接随 AI 特性批次落 behaviors/SSE。
    ("AiChat", "chat"),
}

# 行为模块（手写面）按 face 路由：admin → services/behaviors.rs，
# app → services/public.rs（每个面恒定携带一行 `behaviors:`，供 hand 形态委派）。

# 面特定的行为覆写——方法体改为调用手写模块，行为与生成代码解耦：
#   app 公开站点语义（参照 app/service 的服务层逐方法对位）：公开读
#   状态过滤、内容写禁用、Host 租户解析、游客评论策略、档案/改密钉扎。
#   admin 侧：文件元数据创建（服务端操作者盖章）、路由表同步（本二进制
#   路由语料）、walk-route 调试面、权限派生重建触发、改密/联系面、
#   管理员改密适配（BFF 路径参数 ↔ 领域 UpdateUserRequest）。
BEHAVIOR_METHODS = {
    "admin": {
        ("Api", "sync_apis"): "sync_apis",
        ("Api", "get_walk_route_data"): "walk_route_data",
        ("File", "create"): "file_create",
        ("Permission", "sync_permissions"): "sync_permissions",
        ("User", "edit_user_password"): "edit_user_password",
        ("UserProfile", "change_password"): "change_password",
        ("UserProfile", "bind_contact"): "bind_contact",
        ("UserProfile", "verify_contact"): "verify_contact",
        # 挑战校验的响应映射：BFF LoginResponse（通过即发正式令牌对）
        # ↔ 域 VerifyMFAChallengeResponse（success + 可选令牌对）。
        ("MFA", "verify_m_f_a_challenge"): "mfa_verify_challenge",
    },
    "app": {
        ("Post", "list"): "post_list",
        ("Post", "get"): "post_get",
        ("Post", "search_posts"): "post_search",
        # BFF 把模型查询/引用解析挂在 PostService 下，域对应在
        # ContentModelService——改名转发（同消息类型）。
        ("Post", "get_post_content_model"): "post_content_model",
        ("Post", "resolve_content_references"): "resolve_content_references",
        ("Post", "create"): "forbidden_mutation",
        ("Post", "update"): "forbidden_mutation",
        ("Post", "delete"): "forbidden_mutation",
        ("Category", "list"): "category_list",
        ("Category", "get"): "category_get",
        ("Category", "create"): "forbidden_mutation",
        ("Category", "update"): "forbidden_mutation",
        ("Category", "delete"): "forbidden_mutation",
        ("Page", "list"): "page_list",
        ("Page", "get"): "page_get",
        ("Page", "create"): "forbidden_mutation",
        ("Page", "update"): "forbidden_mutation",
        ("Page", "delete"): "forbidden_mutation",
        ("Tag", "create"): "forbidden_mutation",
        ("Tag", "update"): "forbidden_mutation",
        ("Tag", "delete"): "forbidden_mutation",
        ("Navigation", "create"): "forbidden_mutation",
        ("Navigation", "update"): "forbidden_mutation",
        ("Navigation", "delete"): "forbidden_mutation",
        ("Site", "list"): "forbidden_mutation",
        ("Site", "create"): "forbidden_mutation",
        ("Site", "update"): "forbidden_mutation",
        ("Site", "delete"): "forbidden_mutation",
        ("Site", "get_site_by_domain"): "site_by_domain",
        ("Comment", "list"): "comment_list",
        ("Comment", "get"): "comment_get",
        ("Comment", "create"): "comment_create",
        ("Comment", "update"): "comment_update",
        ("Comment", "delete"): "comment_delete",
        ("UserProfile", "change_password"): "change_password",
        ("UserProfile", "bind_contact"): "bind_contact",
        ("UserProfile", "verify_contact"): "verify_contact",
    },
}

# 响应形状错位（BFF Empty ↔ 领域实体）：转发后丢弃响应体（`drop` 形态）
ADAPTER_METHODS = {("Tenant", "create"), ("Site", "update")}

RUST_KEYWORDS = {"type", "ref"}


def parse_traits(gen_path):
    src = open(gen_path, encoding="utf-8").read()
    traits = {}
    for m in re.finditer(
        r"#\[async_trait::async_trait\]\s*pub trait (\w+)Handlers: Send \+ Sync \{(.*?)\n    \}",
        src,
        re.S,
    ):
        name, body = m.group(1), m.group(2)
        methods = []
        for mm in re.finditer(
            r"async fn (\w+)\(\s*&self, _?ctx: rushwind_http_binding::ctx::RequestContext, req: (.+?)\)\s*-> Result<(.+)>;",
            body,
        ):
            # normalize: the generated faces address types as crate::proto
            # (they live in the proto crate); the proxies live in the
            # service crate. Result<Resp, Err> — keep only the Resp.
            norm = lambda t: t.strip().replace("crate::proto::", "proto::proto::")
            resp = mm.group(3).split(", rushwind_http_binding::envelope::StatusError")[0]
            methods.append((mm.group(1), norm(mm.group(2)), norm(resp)))
        if methods:
            traits[name] = methods
    return traits


def snake(name):
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()


def method_kind(service, method, face):
    """One contract method's proxy-kit form — (kind, target-name).

    Precedence: the hand behavior wins over everything (an admin face
    method can be both STUB-listed and behavior-overridden — the stub
    only applies where no behavior rides), then the renamed forward
    (same service client, same message types, drifted RPC name), then
    the response-dropping adapter, then the not-implemented stub, else
    the plain pass-through.
    """
    behavior = BEHAVIOR_METHODS.get(face, {}).get((service, method))
    if behavior:
        return "hand", behavior
    renamed = RENAMED_METHODS.get((service, method))
    if renamed:
        return "renamed", renamed
    if (service, method) in ADAPTER_METHODS:
        return "drop", None
    if (service, method) in STUB_METHODS:
        return "stub", None
    return "pass", None


def shorten(ty, pkgs):
    """A type path against the package-root aliases — same type, shorter
    line (the fully-qualified paths push the macro table past the
    100-column width rustfmt holds elsewhere in the crate)."""
    for pkg in pkgs:
        ty = ty.replace(f"proto::proto::{pkg}::service::v1::", f"{pkg}_v1::")
    return ty.replace("proto::proto::pagination::", "pagination::")


def main():
    gen_path, face = sys.argv[1], sys.argv[2]
    traits = parse_traits(gen_path)
    gen_mod = "gen_admin" if face == "admin" else "gen_app"

    # First pass — the emitted faces, so the alias block covers exactly
    # the packages the output references.
    faces = []
    used_pkgs = set()
    for trait_name, methods in sorted(traits.items()):
        service = trait_name.removesuffix("Service")
        if service in SKIP or service in SKIP_BY_FACE.get(face, set()):
            continue
        pkg = PACKAGE.get(service)
        if pkg is None:
            continue
        kinds = [(m, *method_kind(service, m, face), req_ty, resp_ty)
                 for m, req_ty, resp_ty in methods]
        faces.append((trait_name, service, pkg, kinds))
        used_pkgs.add(pkg)
    uses_pagination = any(
        "proto::proto::pagination::" in ty
        for _, _, _, kinds in faces
        for _, _, _, req_ty, resp_ty in kinds
        for ty in (req_ty, resp_ty)
    )

    out = []
    out.append("//! Generated thin-BFF proxies — a table of `passthrough_proxy!`")
    out.append("//! invocations, one per BFF face (see services/proxy_kit.rs for the")
    out.append("//! pass/drop/stub/hand method kinds; the BFF and domain methods share")
    out.append("//! their message types by contract, so no mapping rides here).")
    out.append("//! DO NOT EDIT; regenerate via scripts/gen-proxies.py when the")
    out.append("//! contract re-syncs. Hand-written faces (authentication — captcha/")
    out.append("//! cookie concerns; admin-portal aggregation; file-transfer multipart)")
    out.append("//! live in their own modules.")
    out.append("#![allow(clippy::all)]")
    out.append("#![allow(missing_docs)]")
    out.append("")
    out.append("use crate::passthrough_proxy;")
    # Package-root aliases: every type below addresses its package through
    # one of these (see `shorten`), keeping the lines at the 100-column
    # width. Sorted: rustfmt orders the block and would otherwise drift
    # the committed file away from the generator's output.
    import_lines = [
        f"use proto::proto::{pkg}::service::v1 as {pkg}_v1;" for pkg in used_pkgs
    ]
    if uses_pagination:
        import_lines.append("use proto::proto::pagination;")
    out.extend(sorted(import_lines))
    out.append("")

    emitted = 0
    for trait_name, service, pkg, kinds in faces:
        client_mod, client_ty = DOMAIN_CLIENT.get(
            service,
            (f"{SNAKE_OVERRIDE.get(service, snake(service))}_service_client",
             f"{service}ServiceClient"),
        )
        # The Channel-generic spelled out: the kit calls the client via a
        # qualified path (<Ty>::new), where inference does not apply.
        client_head = f"        client: {pkg}_v1::{client_mod}::{client_ty}"
        struct_name = f"{service}Proxy"
        behaviors = "behaviors" if face == "admin" else "public"
        out.append("passthrough_proxy! {")
        out.append(f"    /// The pass-through proxy of `{trait_name}Handlers`.")
        for_line = (
            f"    proto::{gen_mod}::services::{trait_name}Handlers for {struct_name} {{"
        )
        if len(for_line) <= 100:
            out.append(for_line)
        else:
            out.append(f"    proto::{gen_mod}::services::{trait_name}Handlers")
            out.append(f"        for {struct_name} {{")
        client_tail = "<tonic::transport::Channel>,"
        if len(client_head) + len(client_tail) <= 100:
            out.append(client_head + client_tail)
        elif len(client_head) + 1 <= 100:
            out.append(client_head + "<")
            out.append("            tonic::transport::Channel,")
            out.append("        >,")
        else:
            out.append(f"        client: {pkg}_v1::{client_mod}::")
            out.append(f"            {client_ty}<tonic::transport::Channel>,")
        out.append(f"        behaviors: {behaviors},")
        out.append("        methods: [")
        for method, kind, behavior, req_ty, resp_ty in kinds:
            mname = f"r#{method}" if method in RUST_KEYWORDS else method
            req = shorten(req_ty, used_pkgs)
            resp = shorten(resp_ty, used_pkgs)
            suffix = f" => {behavior}," if kind in ("hand", "renamed") else ","
            line = f"            {kind} {mname}({req}) -> {resp}{suffix}"
            if len(line) <= 100:
                out.append(line)
            elif len(f"            {kind} {mname}({req})") <= 100:
                out.append(f"            {kind} {mname}({req})")
                out.append(f"                -> {resp}{suffix}")
            else:
                out.append(f"            {kind} {mname}(")
                out.append(f"                {req}")
                out.append(f"            ) -> {resp}{suffix}")
        out.append("        ]")
        out.append("    }")
        out.append("}")
        out.append("")
        emitted += 1

    print("\n".join(out))
    print(f"// emitted {emitted} proxy impls", file=sys.stderr)


if __name__ == "__main__":
    main()
