//! The service implementations, one module per BFF service. Modules
//! absent from this list ride the generator's null placeholders.

pub mod admin_portal;
pub mod authentication;
pub mod behaviors;
pub mod component_definition;
pub mod file_transfer;
pub mod internal_message;
pub mod preview;
pub mod proxies;
pub mod proxy_kit;

use crate::state::StatusError;

/// The tonic Status → the admin BFF's four-field envelope (the
/// error-table anchors the reason when the code maps onto one).
pub fn map_status(e: tonic::Status) -> StatusError {
    // The core's reason-carrying envelope rides the status details —
    // recover the resolved (status, reason, message) verbatim when the
    // reason resolves against the generated error tables (the recovered
    // string must anchor onto a 'static literal from that table).
    if let Some((status, reason, message)) = recover_envelope(&e) {
        return StatusError::new(status, reason, message);
    }
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

/// Recovers the core's envelope off the status details. The JSON payload
/// is `{status, reason, message}` — but only a reason that anchors onto
/// a generated error-table literal is trusted; anything else (statuses
/// built without the envelope, malformed payloads) yields `None` and the
/// caller falls back to the coarse code mapping.
fn recover_envelope(e: &tonic::Status) -> Option<(i32, &'static str, String)> {
    let v: serde_json::Value = serde_json::from_slice(e.details()).ok()?;
    let reason = v.get("reason")?.as_str()?;
    let status = i32::try_from(v.get("status")?.as_i64()?).ok()?;
    let message = v
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or_default()
        .to_string();
    let anchored = proto::gen_admin::error_tables::ERROR_STATUS
        .iter()
        .chain(proto::gen_app::error_tables::ERROR_STATUS.iter())
        .find(|(_, r, _)| *r == reason)
        .map(|(_, r, _)| *r)?;
    Some((status, anchored, message))
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

/// The claims-direct variant of [`with_operator`] — the hand-mounted
/// faces (file transfer, outside the bind layer) carry the operator
/// the same way the proxies do.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_status_recovers_the_core_envelope() {
        // The core's reason-carrying envelope rides the details — the
        // resolved pair comes back verbatim (the credential-failure
        // classification reads this).
        let details = serde_json::json!({
            "status": 401, "reason": "INVALID_PASSWORD", "message": "incorrect password"
        });
        let e = tonic::Status::with_details(
            tonic::Code::Unauthenticated,
            "incorrect password",
            details.to_string().into(),
        );
        let mapped = map_status(e);
        assert_eq!(mapped.reason, "INVALID_PASSWORD");
        assert_eq!(mapped.status, 401);
        assert_eq!(mapped.message, "incorrect password");
    }

    #[test]
    fn map_status_ignores_unanchored_reasons() {
        // A reason outside the error tables falls back to the coarse
        // code mapping — the anchor requirement is the trust boundary.
        let details = serde_json::json!({
            "status": 400, "reason": "MADE_UP", "message": "x"
        });
        let e = tonic::Status::with_details(
            tonic::Code::InvalidArgument,
            "x",
            details.to_string().into(),
        );
        let mapped = map_status(e);
        assert_eq!(mapped.reason, "BAD_REQUEST");
        assert_eq!(mapped.status, 400);
    }

    #[test]
    fn map_status_falls_back_without_details() {
        let mapped = map_status(tonic::Status::not_found("row vanished"));
        assert_eq!(mapped.reason, "NOT_FOUND");
        assert_eq!(mapped.status, 404);
        assert_eq!(mapped.message, "row vanished");
    }
}
