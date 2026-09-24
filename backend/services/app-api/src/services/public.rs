//! The app face's site-public behaviors — the C-side BFF semantics a
//! bare pass-through cannot carry (the reference's app service layer,
//! mirrored method-for-method): the public-read status filters, the
//! content-mutation bans, the Host-driven tenant resolution, the guest
//! comment policy, and the profile/password pinning. The generated
//! proxies route here by the generator's behavior table.

use pbjson_types::Empty;
use rushwind_http_binding::ctx::RequestContext;

use crate::services::{map_status, with_operator};
use crate::state::{not_found, operator_of, status_error, AppState, StatusError};
use crate::token::UserTokenPayload;

type Ctx = RequestContext;

// ── shared plumbing ──────────────────────────────────────────────────

/// The mutation ban (the reference's per-service Create/Update/Delete
/// bodies on the public app service — RBAC is intentionally noop, so
/// the BFF refuses outright).
pub async fn forbidden_mutation<T, R>(
    _state: &AppState,
    _ctx: &Ctx,
    _req: R,
) -> Result<T, StatusError> {
    Err(forbidden(
        "content mutation is not allowed on the public app service",
    ))
}

fn forbidden(message: &'static str) -> StatusError {
    StatusError::new(403, "FORBIDDEN", message)
}

fn not_implemented(message: &'static str) -> StatusError {
    StatusError::new(501, "NOT_IMPLEMENTED", message)
}

fn bad_request(message: &'static str) -> StatusError {
    status_error("BAD_REQUEST", message)
}

/// The request Host with the port stripped (the domain form the tenant
/// rows carry — the reference's SplitHostPort fallback for the
/// port-bearing dev hosts).
fn host_of(ctx: &Ctx) -> Option<String> {
    let host = ctx.headers.get("host")?.trim().to_string();
    if host.is_empty() {
        return None;
    }
    let domain = if let Some(rest) = host.strip_prefix('[') {
        match rest.split_once(']') {
            Some((h, _tail)) => h.to_string(),
            None => host.clone(),
        }
    } else {
        match host.split_once(':') {
            Some((h, p))
                if !h.is_empty() && !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()) =>
            {
                h.to_string()
            }
            _ => host.clone(),
        }
    };
    (!domain.is_empty()).then_some(domain)
}

/// The bearer credential off the headers, when the caller supplied one.
fn bearer_of(ctx: &Ctx) -> Option<String> {
    let auth = ctx.headers.get("authorization")?;
    let token = auth
        .strip_prefix("Bearer ")
        .or_else(|| auth.strip_prefix("bearer "))?;
    (!token.is_empty()).then(|| token.to_string())
}

/// The optional identity: a valid bearer upgrades the anonymous caller
/// on the whitelisted faces (the reference's manual optional auth on
/// comment creation) — the gate's two stages, tolerant of absence.
async fn optional_operator(state: &AppState, ctx: &Ctx) -> Option<UserTokenPayload> {
    let token = bearer_of(ctx)?;
    let headers = vec![(String::from("authorization"), format!("Bearer {token}"))];
    let claims = state.authenticator.authenticate(&headers).ok()?;
    let payload = UserTokenPayload::from_claims(&claims.0)?;
    let mut core =
        proto::proto::authentication::service::v1::authentication_service_client::AuthenticationServiceClient::new(
            state.core_channel.clone(),
        );
    let valid = core
        .validate_token(tonic::Request::new(
            proto::proto::authentication::service::v1::ValidateTokenRequest {
                token,
                client_type: 1,
                token_category: proto::proto::authentication::service::v1::TokenCategory::Access
                    as i32,
                ..Default::default()
            },
        ))
        .await
        .ok()?
        .into_inner();
    valid.is_valid.then_some(payload)
}

/// The Host→tenant resolution over the core face (0 = unresolved —
/// the caller decides the fail-closed shape).
async fn tenant_by_host(state: &AppState, ctx: &Ctx) -> u32 {
    let Some(domain) = host_of(ctx) else {
        return 0;
    };
    let mut core =
        proto::proto::identity::service::v1::tenant_service_client::TenantServiceClient::new(
            state.core_channel.clone(),
        );
    match core
        .resolve_tenant_by_domain(tonic::Request::new(
            proto::proto::identity::service::v1::ResolveTenantByDomainRequest { domain },
        ))
        .await
    {
        Ok(r) => r.into_inner().tenant_id,
        Err(_) => 0,
    }
}

/// The outbound relay with the operator bag plus the trusted inner
/// tenant stamp (the anonymous chain's Host resolution; inner-gRPC
/// metadata cannot be forged by external callers).
fn request_with_tenant<T>(ctx: &Ctx, msg: T, tenant: u32) -> tonic::Request<T> {
    let mut req = with_operator(ctx, msg);
    if tenant > 0 {
        if let Ok(v) = tonic::metadata::MetadataValue::try_from(tenant.to_string().as_str()) {
            req.metadata_mut().insert("x-md-global-tenant-id", v);
        }
    }
    req
}

// ── post ─────────────────────────────────────────────────────────────

const PUBLISHED: i32 = proto::proto::content::service::v1::post::PostStatus::Published as i32;
const ACTIVE: i32 = proto::proto::content::service::v1::category::CategoryStatus::Active as i32;
const APPROVED: i32 = proto::proto::comment::service::v1::comment::Status::Approved as i32;

/// 公开列表仅返回已发布文章，并按过滤后条数重记 total。
pub async fn post_list(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::pagination::PagingRequest,
) -> Result<proto::proto::content::service::v1::ListPostResponse, StatusError> {
    let mut core = proto::proto::content::service::v1::post_service_client::PostServiceClient::new(
        state.core_channel.clone(),
    );
    let mut resp = core
        .list(with_operator(ctx, req))
        .await
        .map_err(map_status)?
        .into_inner();
    resp.items.retain(|p| p.status == Some(PUBLISHED));
    resp.total = resp.items.len() as u64;
    Ok(resp)
}

/// 公开详情仅返回已发布文章，其余状态按未找到处理。
pub async fn post_get(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::content::service::v1::GetPostRequest,
) -> Result<proto::proto::content::service::v1::Post, StatusError> {
    let mut core = proto::proto::content::service::v1::post_service_client::PostServiceClient::new(
        state.core_channel.clone(),
    );
    let post = core
        .get(with_operator(ctx, req))
        .await
        .map_err(map_status)?
        .into_inner();
    if post.status != Some(PUBLISHED) {
        return Err(not_found("post"));
    }
    Ok(post)
}

/// 搜索：按请求 Host 解析租户后经可信内网元数据传给 core 兜底；解析
/// 不出则不注入，core 端维持 fail-closed（返回空）。
pub async fn post_search(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::content::service::v1::SearchPostsRequest,
) -> Result<proto::proto::content::service::v1::SearchPostsResponse, StatusError> {
    let tenant = tenant_by_host(state, ctx).await;
    let mut core = proto::proto::content::service::v1::post_service_client::PostServiceClient::new(
        state.core_channel.clone(),
    );
    Ok(core
        .search_posts(request_with_tenant(ctx, req, tenant))
        .await
        .map_err(map_status)?
        .into_inner())
}

// ── category ─────────────────────────────────────────────────────────

/// 公开列表仅返回启用（前台可见）分类。
pub async fn category_list(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::pagination::PagingRequest,
) -> Result<proto::proto::content::service::v1::ListCategoryResponse, StatusError> {
    let mut core =
        proto::proto::content::service::v1::category_service_client::CategoryServiceClient::new(
            state.core_channel.clone(),
        );
    let mut resp = core
        .list(with_operator(ctx, req))
        .await
        .map_err(map_status)?
        .into_inner();
    resp.items.retain(|c| c.status == Some(ACTIVE));
    resp.total = resp.items.len() as u64;
    Ok(resp)
}

pub async fn category_get(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::content::service::v1::GetCategoryRequest,
) -> Result<proto::proto::content::service::v1::Category, StatusError> {
    let mut core =
        proto::proto::content::service::v1::category_service_client::CategoryServiceClient::new(
            state.core_channel.clone(),
        );
    let category = core
        .get(with_operator(ctx, req))
        .await
        .map_err(map_status)?
        .into_inner();
    if category.status != Some(ACTIVE) {
        return Err(not_found("category"));
    }
    Ok(category)
}

// ── page ─────────────────────────────────────────────────────────────

/// 公开列表仅返回已发布页面。
pub async fn page_list(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::pagination::PagingRequest,
) -> Result<proto::proto::content::service::v1::ListPageResponse, StatusError> {
    let mut core = proto::proto::content::service::v1::page_service_client::PageServiceClient::new(
        state.core_channel.clone(),
    );
    let mut resp = core
        .list(with_operator(ctx, req))
        .await
        .map_err(map_status)?
        .into_inner();
    resp.items.retain(|p| p.status == Some(PUBLISHED));
    resp.total = resp.items.len() as u64;
    Ok(resp)
}

pub async fn page_get(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::content::service::v1::GetPageRequest,
) -> Result<proto::proto::content::service::v1::Page, StatusError> {
    let mut core = proto::proto::content::service::v1::page_service_client::PageServiceClient::new(
        state.core_channel.clone(),
    );
    let page = core
        .get(with_operator(ctx, req))
        .await
        .map_err(map_status)?
        .into_inner();
    if page.status != Some(PUBLISHED) {
        return Err(not_found("page"));
    }
    Ok(page)
}

// ── site ─────────────────────────────────────────────────────────────

/// 站点渲染配置：domain 不接受调用方传入，始终由请求 Host 提取；无
/// Host 直接 400（前台只能拿到"自己所在域名"的站点配置）。
pub async fn site_by_domain(
    state: &AppState,
    ctx: &Ctx,
    mut req: proto::proto::site::service::v1::GetSiteByDomainRequest,
) -> Result<proto::proto::site::service::v1::Site, StatusError> {
    let Some(domain) = host_of(ctx) else {
        return Err(bad_request("resolvable host is required"));
    };
    req.domain = domain;
    let mut core = proto::proto::site::service::v1::site_service_client::SiteServiceClient::new(
        state.core_channel.clone(),
    );
    Ok(core
        .get_site_by_domain(with_operator(ctx, req))
        .await
        .map_err(map_status)?
        .into_inner())
}

// ── comment ──────────────────────────────────────────────────────────

/// 公开列表仅返回已批准评论。
pub async fn comment_list(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::pagination::PagingRequest,
) -> Result<proto::proto::comment::service::v1::ListCommentResponse, StatusError> {
    let mut core =
        proto::proto::comment::service::v1::comment_service_client::CommentServiceClient::new(
            state.core_channel.clone(),
        );
    let mut resp = core
        .list(with_operator(ctx, req))
        .await
        .map_err(map_status)?
        .into_inner();
    resp.items.retain(|c| c.status == Some(APPROVED));
    resp.total = resp.items.len() as u64;
    Ok(resp)
}

pub async fn comment_get(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::comment::service::v1::GetCommentRequest,
) -> Result<proto::proto::comment::service::v1::Comment, StatusError> {
    let mut core =
        proto::proto::comment::service::v1::comment_service_client::CommentServiceClient::new(
            state.core_channel.clone(),
        );
    let comment = core
        .get(with_operator(ctx, req))
        .await
        .map_err(map_status)?
        .into_inner();
    if comment.status != Some(APPROVED) {
        return Err(not_found("comment"));
    }
    Ok(comment)
}

/// 评论创建（白名单面，游客可达）：身份按可选认证分辨——携带有效
/// token 为登录用户（归属真实身份、昵称回落用户名），否则为游客（必须
/// 提供昵称与邮箱）；统一待审核入库；租户按 Host 解析后经可信内网
/// 元数据传给 core 落库归属。
pub async fn comment_create(
    state: &AppState,
    ctx: &Ctx,
    mut req: proto::proto::comment::service::v1::CreateCommentRequest,
) -> Result<proto::proto::comment::service::v1::Comment, StatusError> {
    use proto::proto::comment::service::v1::comment::AuthorType;
    let Some(mut data) = req.data.take() else {
        return Err(bad_request("invalid parameter"));
    };
    let operator = optional_operator(state, ctx).await;
    match &operator {
        Some(op) => {
            data.author_id = Some(op.user_id);
            data.author_type = Some(AuthorType::User as i32);
            data.created_by = Some(op.user_id);
            if data.author_name.as_deref().unwrap_or("").is_empty() {
                data.author_name = Some(op.username.clone());
            }
        }
        None => {
            if data.author_name.as_deref().unwrap_or("").is_empty()
                || data.author_email.as_deref().unwrap_or("").is_empty()
            {
                return Err(bad_request(
                    "author name and email are required for guest comments",
                ));
            }
            data.author_type = Some(AuthorType::Guest as i32);
            data.author_id = Some(0);
            data.created_by = Some(0);
        }
    }
    data.status = Some(proto::proto::comment::service::v1::comment::Status::Pending as i32);
    req.data = Some(data);
    let tenant = tenant_by_host(state, ctx).await;
    let mut core =
        proto::proto::comment::service::v1::comment_service_client::CommentServiceClient::new(
            state.core_channel.clone(),
        );
    Ok(core
        .create(request_with_tenant(ctx, req, tenant))
        .await
        .map_err(map_status)?
        .into_inner())
}

/// IDOR 防护：只能改/删自己创建的评论。
async fn ensure_comment_owner(
    state: &AppState,
    comment_id: u32,
    user_id: u32,
) -> Result<(), StatusError> {
    let mut core =
        proto::proto::comment::service::v1::comment_service_client::CommentServiceClient::new(
            state.core_channel.clone(),
        );
    let comment = core
        .get(tonic::Request::new(
            proto::proto::comment::service::v1::GetCommentRequest {
                id: comment_id,
                ..Default::default()
            },
        ))
        .await
        .map_err(map_status)?
        .into_inner();
    if comment.created_by != Some(user_id) {
        return Err(forbidden("you can only modify your own comments"));
    }
    Ok(())
}

pub async fn comment_update(
    state: &AppState,
    ctx: &Ctx,
    mut req: proto::proto::comment::service::v1::UpdateCommentRequest,
) -> Result<proto::proto::comment::service::v1::Comment, StatusError> {
    let op = operator_of(ctx)?;
    ensure_comment_owner(state, req.id, op.user_id).await?;
    if let Some(data) = req.data.as_mut() {
        data.updated_by = Some(op.user_id);
    }
    let mut core =
        proto::proto::comment::service::v1::comment_service_client::CommentServiceClient::new(
            state.core_channel.clone(),
        );
    Ok(core
        .update(with_operator(ctx, req))
        .await
        .map_err(map_status)?
        .into_inner())
}

pub async fn comment_delete(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::comment::service::v1::DeleteCommentRequest,
) -> Result<Empty, StatusError> {
    use proto::proto::comment::service::v1::delete_comment_request::QueryBy;
    let op = operator_of(ctx)?;
    let comment_id = match req.query_by {
        Some(QueryBy::Id(id)) => id,
        _ => return Err(bad_request("query_by required")),
    };
    ensure_comment_owner(state, comment_id, op.user_id).await?;
    let mut core =
        proto::proto::comment::service::v1::comment_service_client::CommentServiceClient::new(
            state.core_channel.clone(),
        );
    core.delete(with_operator(ctx, req))
        .await
        .map_err(map_status)?;
    Ok(Empty {})
}

// ── user profile ─────────────────────────────────────────────────────

/// 改密：转 core 凭证面，NeedDecrypt 与登录/注册的 AES 密文口径一致。
pub async fn change_password(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::identity::service::v1::ChangePasswordRequest,
) -> Result<Empty, StatusError> {
    let op = operator_of(ctx)?;
    let mut core = proto::proto::authentication::service::v1::user_credential_service_client::UserCredentialServiceClient::new(
        state.core_channel.clone(),
    );
    core.change_credential(tonic::Request::new(
        proto::proto::authentication::service::v1::ChangeCredentialRequest {
            identity_type:
                proto::proto::authentication::service::v1::user_credential::IdentityType::Username
                    as i32,
            identifier: op.username,
            old_credential: req.old_password,
            new_credential: req.new_password,
            need_decrypt: true,
        },
    ))
    .await
    .map_err(map_status)?;
    Ok(Empty {})
}

pub async fn bind_contact(
    _state: &AppState,
    _ctx: &Ctx,
    _req: proto::proto::identity::service::v1::BindContactRequest,
) -> Result<Empty, StatusError> {
    Err(not_implemented("contact binding is not implemented"))
}

pub async fn verify_contact(
    _state: &AppState,
    _ctx: &Ctx,
    _req: proto::proto::identity::service::v1::VerifyContactRequest,
) -> Result<Empty, StatusError> {
    Err(not_implemented("contact verification is not implemented"))
}

// ── register ─────────────────────────────────────────────────────────

/// The C-side registration chain (the reference's Register verbatim):
/// the server-side field gates (3-20 bytes of `[A-Za-z0-9_]`, ≥6-byte
/// password), the Host-resolved tenant ownership (the caller's
/// tenant_code is dropped — the registration form carries no tenant
/// input), the tenant-ON gate, and the AES(base64) password decode
/// (the core credential path expects plaintext — it bcrypts before
/// storing, and login decrypts the same way; storing bcrypt(ciphertext)
/// would never verify).
pub async fn register(
    state: &AppState,
    ctx: &Ctx,
    mut req: proto::proto::authentication::service::v1::RegisterUserRequest,
) -> Result<u32, StatusError> {
    use proto::proto::identity::service::v1::get_tenant_request::QueryBy;
    if req.username.trim().is_empty() || req.password.is_empty() {
        return Err(bad_request("invalid username or password"));
    }
    // 口径与前端一致：3-20 位字母/数字/下划线；密码至少 6 位。
    let name_len = req.username.len();
    let name_ok = (3..=20).contains(&name_len)
        && req
            .username
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_');
    if !name_ok {
        return Err(bad_request("invalid username format"));
    }
    if req.password.len() < 6 {
        return Err(bad_request("password too weak"));
    }
    let tenant_id = tenant_by_host(state, ctx).await;
    if tenant_id == 0 {
        return Err(bad_request(
            "cannot resolve tenant, registration unavailable",
        ));
    }
    let mut tenants =
        proto::proto::identity::service::v1::tenant_service_client::TenantServiceClient::new(
            state.core_channel.clone(),
        );
    let tenant = tenants
        .get(tonic::Request::new(
            proto::proto::identity::service::v1::GetTenantRequest {
                query_by: Some(QueryBy::Id(tenant_id)),
                ..Default::default()
            },
        ))
        .await
        .map_err(map_status)?
        .into_inner();
    let code = tenant.code.clone().unwrap_or_default();
    if tenant.status != Some(proto::proto::identity::service::v1::tenant::Status::On as i32)
        || code.is_empty()
    {
        return Err(bad_request("tenant unavailable"));
    }
    // 租户归属由服务端决定，调用方不可覆盖。
    req.tenant_code = code;
    let plain = store::crypto::decrypt_login_credential(&req.password)
        .map_err(|_| bad_request("invalid password encoding"))?;
    req.password = plain;
    let mut core = proto::proto::authentication::service::v1::authentication_service_client::AuthenticationServiceClient::new(
        state.core_channel.clone(),
    );
    let resp = core
        .register_user(tonic::Request::new(req))
        .await
        .map_err(map_status)?
        .into_inner();
    Ok(resp.user_id)
}

#[cfg(test)]
mod host_tests {
    use super::*;

    fn ctx_with(host: &str) -> RequestContext {
        let mut ctx = RequestContext::default();
        ctx.headers.insert("host".into(), host.into());
        ctx
    }

    #[test]
    fn host_strips_ports_and_brackets() {
        assert_eq!(
            host_of(&ctx_with("example.com:5001")).as_deref(),
            Some("example.com")
        );
        assert_eq!(
            host_of(&ctx_with("example.com")).as_deref(),
            Some("example.com")
        );
        assert_eq!(host_of(&ctx_with("[::1]:5001")).as_deref(), Some("::1"));
        assert_eq!(
            host_of(&ctx_with("localhost")).as_deref(),
            Some("localhost")
        );
        assert_eq!(host_of(&ctx_with("")), None);
        assert_eq!(host_of(&RequestContext::default()), None);
    }
}
