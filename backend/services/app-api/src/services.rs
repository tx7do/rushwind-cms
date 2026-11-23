//! The BFF service implementations, one module per app-face service.
//! Modules absent from this list ride the generator's null
//! placeholders.

pub mod authentication;
pub mod file_transfer;
pub mod proxies;
pub mod proxy_kit;
pub mod public;

use crate::state::StatusError;

/// The tonic Status → the app BFF's four-field envelope (the
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

    #[test]
    fn map_status_coarse_mapping_covers_every_code_family() {
        // Without the core's envelope details the gRPC code alone picks
        // the reason, and the error table the status behind it.
        let cases = [
            (tonic::Code::InvalidArgument, "BAD_REQUEST", 400),
            (tonic::Code::Unauthenticated, "UNAUTHORIZED", 401),
            (tonic::Code::PermissionDenied, "FORBIDDEN", 403),
            (tonic::Code::NotFound, "NOT_FOUND", 404),
            (tonic::Code::AlreadyExists, "CONFLICT", 409),
            (tonic::Code::Aborted, "CONFLICT", 409),
        ];
        for (code, reason, status) in cases {
            let mapped = map_status(tonic::Status::new(code, "m"));
            assert_eq!((mapped.status, mapped.reason), (status, reason), "{code:?}");
            assert_eq!(mapped.message, "m");
        }
        // Everything else is the Unknown branch: 500 with an empty reason.
        for code in [
            tonic::Code::Internal,
            tonic::Code::Unavailable,
            tonic::Code::Unimplemented,
            tonic::Code::DeadlineExceeded,
            tonic::Code::Cancelled,
            tonic::Code::Unknown,
            tonic::Code::FailedPrecondition,
            tonic::Code::ResourceExhausted,
        ] {
            let mapped = map_status(tonic::Status::new(code, "boom"));
            assert_eq!((mapped.status, mapped.reason), (500, ""), "{code:?}");
            assert_eq!(mapped.message, "boom");
        }
    }

    // ── with_operator ────────────────────────────────────────────────

    /// The claim bag in the shape the relay takes: an owned map, optional.
    fn bag(value: &serde_json::Value) -> Option<serde_json::Map<String, serde_json::Value>> {
        value.as_object().cloned()
    }

    #[test]
    fn operator_relay_stamps_the_verified_ids_as_metadata() {
        let claims =
            serde_json::json!({"uid": 5, "tid": 9, "sub": "alice", "roc": ["tenant:user"]});
        let req = with_operator_claims(&bag(&claims), "payload");
        assert_eq!(*req.get_ref(), "payload");
        let md = |name: &str| {
            req.metadata()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        assert_eq!(md("x-user-id").as_deref(), Some("5"));
        assert_eq!(md("x-tenant-id").as_deref(), Some("9"));
        // Nothing else from the bag rides the wire.
        assert!(req.metadata().get("sub").is_none());
        assert!(req.metadata().get("roc").is_none());

        // Without an identity nothing is stamped.
        let req = with_operator_claims(&None, ());
        assert!(req.metadata().get("x-user-id").is_none());
        assert!(req.metadata().get("x-tenant-id").is_none());
    }

    #[test]
    fn operator_relay_reads_only_unsigned_integral_ids() {
        // A negative, a string, a float, or null never stamps.
        for value in [
            serde_json::json!(-1),
            serde_json::json!("5"),
            serde_json::json!(5.5),
            serde_json::json!(null),
        ] {
            let claims = serde_json::json!({"uid": value, "tid": 1});
            let req = with_operator_claims(&bag(&claims), ());
            let user = req
                .metadata()
                .get("x-user-id")
                .and_then(|v| v.to_str().ok());
            assert_eq!(user, None, "uid {value}");
            // The tenant stamps independently of the user.
            let tenant = req
                .metadata()
                .get("x-tenant-id")
                .and_then(|v| v.to_str().ok());
            assert_eq!(tenant, Some("1"));
        }
        // Zero is a legal id (the platform tenant), and the full i64 range
        // of the int64 finalize rides through.
        let claims = serde_json::json!({"uid": i64::MAX, "tid": 0});
        let req = with_operator_claims(&bag(&claims), ());
        assert_eq!(
            req.metadata()
                .get("x-user-id")
                .and_then(|v| v.to_str().ok()),
            Some("9223372036854775807")
        );
        assert_eq!(
            req.metadata()
                .get("x-tenant-id")
                .and_then(|v| v.to_str().ok()),
            Some("0")
        );
    }
}
