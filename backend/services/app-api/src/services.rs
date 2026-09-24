//! The BFF service implementations, one module per app-face service.
//! Modules absent from this list ride the generator's null
//! placeholders.

pub mod authentication;
pub mod file_transfer;
pub mod proxies;
pub mod public;

use crate::state::StatusError;

/// The tonic Status → the app BFF's four-field envelope (the
/// error-table anchors the reason when the code maps onto one).
pub fn map_status(e: tonic::Status) -> StatusError {
    let reason: &'static str = match e.code() {
        tonic::Code::InvalidArgument => "BAD_REQUEST",
        tonic::Code::Unauthenticated => "UNAUTHORIZED",
        tonic::Code::PermissionDenied => "FORBIDDEN",
        tonic::Code::NotFound => "NOT_FOUND",
        tonic::Code::AlreadyExists | tonic::Code::Aborted => "CONFLICT",
        _ => "",
    };
    if reason.is_empty() {
        crate::state::internal_error(e.message())
    } else {
        crate::state::status_error(reason, e.message())
    }
}

/// Wraps the outbound message with the operator metadata (uid/tid from
/// the verified claims) — the core service reads it where the proto
/// carries no user field (the interaction ledger writes).
pub fn with_operator<T>(
    ctx: &rushwind_http_binding::ctx::RequestContext,
    msg: T,
) -> tonic::Request<T> {
    let mut req = tonic::Request::new(msg);
    if let Some(claims) = &ctx.claims {
        if let Some(uid) = claims.get("uid").and_then(|v| v.as_u64()) {
            if let Ok(v) = uid.to_string().parse() {
                req.metadata_mut().insert("x-user-id", v);
            }
        }
        if let Some(tid) = claims.get("tid").and_then(|v| v.as_u64()) {
            if let Ok(v) = tid.to_string().parse() {
                req.metadata_mut().insert("x-tenant-id", v);
            }
        }
    }
    req
}

/// Wraps the outbound message with the operator metadata carried by a
/// raw claim bag (the hand-mounted faces read the bag off the request
/// extensions, not a RequestContext).
pub fn with_operator_claims<T>(
    claims: &Option<serde_json::Map<String, serde_json::Value>>,
    msg: T,
) -> tonic::Request<T> {
    let ctx = rushwind_http_binding::ctx::RequestContext {
        claims: claims.clone(),
        ..Default::default()
    };
    with_operator(&ctx, msg)
}
