//! The request-identity helpers shared by the service faces: the
//! operator user / tenant stamps the BFF forwards as gRPC metadata
//! (`x-user-id`, `x-tenant-id`, plus the anonymous chain's trusted
//! `x-md-global-tenant-id`). One definition here — the faces used to
//! carry per-file copies.

/// The operator user id off the gRPC metadata the BFF forwards
/// (`x-user-id` from the verified claims) — required.
pub fn operator_of<T>(request: &tonic::Request<T>) -> Result<i64, tonic::Status> {
    request
        .metadata()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .ok_or_else(|| tonic::Status::unauthenticated("user identity required"))
}

/// The operator identity when present (the reference tolerates its
/// absence on some flows, defaulting to operator 0).
pub fn optional_operator_user_id<T>(request: &tonic::Request<T>) -> i64 {
    request
        .metadata()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(0)
}

/// The caller's tenant scope off the BFF-forwarded header (0 = platform,
/// which scopes to nothing — the reference's `maybeTenantFromViewer`
/// only predicate applies for a *named* tenant).
pub fn tenant_of<T>(request: &tonic::Request<T>) -> i64 {
    request
        .metadata()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0)
}

/// The trusted inner tenant stamp: the anonymous chain's explicit
/// `x-md-global-tenant-id` (the reference's metadata channel) wins,
/// else the operator bag's `x-tenant-id`.
pub fn request_tenant_of<T>(request: &tonic::Request<T>) -> i64 {
    for name in ["x-md-global-tenant-id", "x-tenant-id"] {
        if let Some(v) = request.metadata().get(name).and_then(|v| v.to_str().ok()) {
            if let Ok(n) = v.parse::<i64>() {
                if n >= 0 {
                    return n;
                }
            }
        }
    }
    0
}

/// The operator tenant scope as the write faces carry it (u32; 0 =
/// platform). The identity/role/tenant write paths compare it against
/// the column-side i64 with a cast.
pub fn operator_tenant_id<T>(request: &tonic::Request<T>) -> u32 {
    request
        .metadata()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0)
}

/// Platform-admin gate: the operator must exist and ride tenant 0.
pub fn require_admin_operator<T>(request: &tonic::Request<T>) -> Result<(i64, i64), tonic::Status> {
    let user_id = operator_of(request)?;
    let tenant_id = tenant_of(request);
    if tenant_id != 0 {
        return Err(crate::state::forbidden("platform admin only"));
    }
    Ok((tenant_id, user_id))
}

#[cfg(test)]
mod tests {
    use super::*;


    /// A request carrying the given metadata headers.
    fn req_with(headers: &[(&'static str, &'static str)]) -> tonic::Request<()> {
        let mut request = tonic::Request::new(());
        for (name, value) in headers {
            request.metadata_mut().insert(*name, value.parse().unwrap());
        }
        request
    }

    // ── request_tenant_of ───────────────────────────────────────────

    #[test]
    fn tenant_global_header_wins_over_operator_bag() {
        let request = req_with(&[("x-md-global-tenant-id", "5"), ("x-tenant-id", "7")]);
        assert_eq!(request_tenant_of(&request), 5);
    }

    #[test]
    fn tenant_falls_back_to_the_operator_bag_header() {
        let request = req_with(&[("x-tenant-id", "7")]);
        assert_eq!(request_tenant_of(&request), 7);
    }

    #[test]
    fn tenant_invalid_or_negative_global_falls_through() {
        // Non-numeric global → the bag header answers.
        let request = req_with(&[("x-md-global-tenant-id", "abc"), ("x-tenant-id", "7")]);
        assert_eq!(request_tenant_of(&request), 7);

        // Negative global is rejected (n >= 0 gate) → the bag header.
        let request = req_with(&[("x-md-global-tenant-id", "-3"), ("x-tenant-id", "7")]);
        assert_eq!(request_tenant_of(&request), 7);

        // Negative everywhere → 0.
        let request = req_with(&[("x-md-global-tenant-id", "-3"), ("x-tenant-id", "-1")]);
        assert_eq!(request_tenant_of(&request), 0);

        // Non-numeric everywhere → 0.
        let request = req_with(&[
            ("x-md-global-tenant-id", "abc"),
            ("x-tenant-id", "not-a-number"),
        ]);
        assert_eq!(request_tenant_of(&request), 0);
    }

    #[test]
    fn tenant_missing_headers_and_explicit_zero_read_zero() {
        assert_eq!(request_tenant_of(&req_with(&[])), 0);
        assert_eq!(request_tenant_of(&req_with(&[("x-tenant-id", "abc")])), 0);
        // Zero is a legal tenant value (the platform scope).
        assert_eq!(
            request_tenant_of(&req_with(&[("x-md-global-tenant-id", "0")])),
            0
        );
        assert_eq!(request_tenant_of(&req_with(&[("x-tenant-id", "0")])), 0);
    }
}
