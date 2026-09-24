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
SKIP_BY_FACE = {"app": set(), "admin": set()}

# BFF/领域签名错位的方法（非零映射转发）——生成桩，待手写适配：
#   ApiService.SyncApis：BFF Empty ↔ 领域 SyncApisRequest
STUB_METHODS = {
    ("Api", "get_walk_route_data"),
    ("File", "create"),
    ("Menu", "sync_menus"),
    ("Permission", "sync_permissions"),
}

# 行为模块（手写面）：生成代码按 face 路由到对应模块。
BEHAVIOR_MODULES = {"app": "crate::services::public", "admin": "crate::services::behaviors"}

# 面特定的行为覆写——方法体改为调用手写模块，行为与生成代码解耦：
#   app 公开站点语义（参照 app/service 的服务层逐方法对位）：公开读
#   状态过滤、内容写禁用、Host 租户解析、游客评论策略、档案/改密钉扎。
#   admin 侧：文件元数据创建（服务端操作者盖章）、权限派生重建触发、
#   walk-route 调试面（本二进制路由表）、改密/联系面。
BEHAVIOR_METHODS = {
    "admin": {
        ("Api", "get_walk_route_data"): "walk_route_data",
        ("File", "create"): "file_create",
        ("Permission", "sync_permissions"): "sync_permissions",
        ("UserProfile", "change_password"): "change_password",
        ("UserProfile", "bind_contact"): "bind_contact",
        ("UserProfile", "verify_contact"): "verify_contact",
    },
    "app": {
        ("Post", "list"): "post_list",
        ("Post", "get"): "post_get",
        ("Post", "search_posts"): "post_search",
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

# 响应形状错位（BFF Empty ↔ 领域实体）：转发后丢弃响应体
ADAPTER_METHODS = {("Tenant", "create"), ("Site", "update")}
# 请求形状错位：BFF 方法在领域面无同名 RPC——显式适配体
EXPLICIT_METHODS = {("User", "edit_user_password")}

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
    out.append("use crate::services::with_operator;")
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
            behavior = BEHAVIOR_METHODS.get(face, {}).get((service, method))
            out.append("    async fn %s(" % mname)
            out.append("        &self,")
            out.append("        _ctx: rushwind_http_binding::ctx::RequestContext,")
            out.append(f"        req: {req_ty},")
            out.append(f"    ) -> Result<{resp_ty}, StatusError> {{")
            if behavior:
                behavior_mod = BEHAVIOR_MODULES[face]
                out.append("        // The face's hand-written behavior (see the module docs).")
                out.append(f"        {behavior_mod}::{behavior}(&self.state, &_ctx, req).await")
            elif (service, method) == ("Api", "sync_apis"):
                out.append("        let _ = req;")
                out.append("        // Sync the generated route table into sys_apis (the route")
                out.append("        // corpus this very binary was generated from).")
                out.append("        let mut req = proto::proto::permission::service::v1::SyncApisRequest::default();")
                out.append("        for r in proto::gen_admin::routes::ROUTES {")
                out.append("            if r.shadowed { continue; }")
                out.append("            req.apis.push(proto::proto::permission::service::v1::Api {")
                out.append("                operation: Some(r.operation_id.to_string()),")
                out.append("                path: Some(r.path.to_string()),")
                out.append("                method: Some(r.method.to_string()),")
                out.append("                module: Some(r.service_fq.split('.').next().unwrap_or_default().to_string()),")
                out.append("                ..Default::default()")
                out.append("            });")
                out.append("        }")
                out.append("        let mut core = proto::proto::permission::service::v1::api_service_client::ApiServiceClient::new(")
                out.append("            self.state.core_channel.clone(),")
                out.append("        );")
                out.append("        core")
                out.append("            .sync_apis(tonic::Request::new(req))")
                out.append("            .await")
                out.append("            .map_err(map_status)?;")
                out.append("        Ok(pbjson_types::Empty {})")
            elif (service, method) in EXPLICIT_METHODS:
                out.append("        let mut core = proto::proto::identity::service::v1::user_service_client::UserServiceClient::new(")
                out.append("            self.state.core_channel.clone(),")
                out.append("        );")
                out.append("        core")
                out.append("            .update(tonic::Request::new(")
                out.append("                proto::proto::identity::service::v1::UpdateUserRequest {")
                out.append("                    id: req.user_id,")
                out.append("                    password: Some(req.new_password),")
                out.append("                    ..Default::default()")
                out.append("                },")
                out.append("            ))")
                out.append("            .await")
                out.append("            .map_err(map_status)?;")
                out.append("        Ok(pbjson_types::Empty {})")
            elif (service, method) in ADAPTER_METHODS:
                out.append("        let mut core = proto::proto::%s::service::v1::%s::new(" % (pkg_mod, client))
                out.append("            self.state.core_channel.clone(),")
                out.append("        );")
                out.append("        let _ = core")
                out.append(f"            .{method}(tonic::Request::new(req))")
                out.append("            .await")
                out.append("            .map_err(map_status)?;")
                out.append(f"        Ok(<{resp_ty}>::default())")
            elif stubbed:
                out.append("        let _ = (req, &self.state);")
                out.append("        Err(crate::state::internal_error(\"not implemented\"))")
            else:
                out.append("        let mut core = proto::proto::%s::service::v1::%s::new(" % (pkg_mod, client))
                out.append("            self.state.core_channel.clone(),")
                out.append("        );")
                out.append("        Ok(core")
                out.append(f"            .{method}(with_operator(&_ctx, req))")
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
