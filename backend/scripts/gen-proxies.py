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
}

# 手工实现的面（不生成）
SKIP = {"Authentication", "AdminPortal", "FileTransfer"}
# 面特定的额外跳过（BFF 方法集超出领域面）
SKIP_BY_FACE = {"app": {"UserProfile"}, "admin": set()}

# BFF/领域签名错位的方法（非零映射转发）——生成桩，待手写适配：
#   ApiService.SyncApis：BFF Empty ↔ 领域 SyncApisRequest
STUB_METHODS = {
    ("Api", "get_walk_route_data"),
    ("Api", "sync_apis"),
    ("File", "create"),
    ("Menu", "sync_menus"),
    ("Permission", "sync_permissions"),
    ("Site", "update"),
    ("Tenant", "create"),
    ("UserProfile", "bind_contact"),
    ("UserProfile", "change_password"),
    ("UserProfile", "verify_contact"),
    ("User", "edit_user_password"),
}

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


def main():
    gen_path, face = sys.argv[1], sys.argv[2]
    traits = parse_traits(gen_path)
    gen_mod = "gen_admin" if face == "admin" else "gen_app"

    out = []
    out.append("//! Generated thin-BFF proxies — one impl per BFF trait method, each a")
    out.append("//! pass-through to the core domain service's gRPC face (the BFF and")
    out.append("//! domain methods share their message types by contract, so no mapping")
    out.append("//! rides here). DO NOT EDIT; regenerate via scripts/gen-proxies.py when")
    out.append("//! the contract re-syncs. Hand-written faces (authentication — captcha/")
    out.append("//! cookie concerns; admin-portal aggregation; file-transfer multipart)")
    out.append("//! live in their own modules.")
    out.append("#![allow(clippy::all)]")
    out.append("#![allow(missing_docs)]")
    out.append("")
    out.append("use std::sync::Arc;")
    out.append("")
    out.append("use crate::state::{AppState, StatusError};")
    out.append("use crate::services::map_status;")
    out.append("")

    emitted = 0
    for trait_name, methods in sorted(traits.items()):
        service = trait_name.removesuffix("Service")
        if service in SKIP or service in SKIP_BY_FACE.get(face, set()):
            continue
        pkg = PACKAGE.get(service)
        if pkg is None:
            continue
        client_mod = f"{snake(trait_name.removesuffix('Service'))}_service_client"
        client = f"{client_mod}::{trait_name.removesuffix('Service')}ServiceClient"
        pkg_mod = "::".join(pkg.split("."))
        struct_name = f"{trait_name.removesuffix('Service')}Proxy"
        out.append(f"/// The pass-through proxy of `{trait_name}Handlers`.")
        out.append(f"pub struct {struct_name} {{")
        out.append("    pub state: Arc<AppState>,")
        out.append("}")
        out.append("")
        out.append("#[async_trait::async_trait]")
        out.append(
            f"impl proto::{gen_mod}::services::{trait_name}Handlers for {struct_name} {{"
        )
        for method, req_ty, resp_ty in methods:
            mname = f"r#{method}" if method in RUST_KEYWORDS else method
            stubbed = (service, method) in STUB_METHODS
            out.append("    async fn %s(" % mname)
            out.append("        &self,")
            out.append("        _ctx: rushwind_http_binding::ctx::RequestContext,")
            out.append(f"        req: {req_ty},")
            out.append(f"    ) -> Result<{resp_ty}, StatusError> {{")
            if stubbed:
                out.append("        let _ = (req, &self.state);")
                out.append("        Err(crate::state::internal_error(\"not implemented\"))")
            else:
                out.append("        let mut core = proto::proto::%s::service::v1::%s::new(" % (pkg_mod, client))
                out.append("            self.state.core_channel.clone(),")
                out.append("        );")
                out.append("        Ok(core")
                out.append(f"            .{method}(tonic::Request::new(req))")
                out.append("            .await")
                out.append("            .map_err(map_status)?")
                out.append("            .into_inner())")
            out.append("    }")
            out.append("")
        out.append("}")
        out.append("")
        emitted += 1

    print("\n".join(out))
    print(f"// emitted {emitted} proxy impls", file=sys.stderr)


if __name__ == "__main__":
    main()
