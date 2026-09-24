//! The admin face's hand behaviors — the methods whose bodies the
//! generator's bare pass-through cannot carry: the file-metadata
//! create with its server-side operator stamp, the permission sync
//! trigger, the walk-route debug dump off this binary's own route
//! table, the credential change (the admin dialog posts cleartext —
//! the core hashes; the reference sets no NeedDecrypt here), and the
//! not-yet-built contact faces (501, the reference's status).

use pbjson_types::Empty;

use crate::services::{map_status, with_operator};
use crate::state::{operator_of, AppState, StatusError};

type Ctx = rushwind_http_binding::ctx::RequestContext;

fn not_implemented(message: &'static str) -> StatusError {
    StatusError::new(501, "NOT_IMPLEMENTED", message)
}

/// The file-metadata create: the operator stamp is server-side (the
/// reference's FileService.Create), the body rides untouched.
pub async fn file_create(
    state: &AppState,
    ctx: &Ctx,
    mut req: proto::proto::storage::service::v1::CreateFileRequest,
) -> Result<Empty, StatusError> {
    let Some(data) = req.data.as_mut() else {
        return Err(crate::state::status_error("BAD_REQUEST", "invalid request"));
    };
    let op = operator_of(ctx)?;
    data.created_by = Some(op.user_id);
    let mut core = proto::proto::storage::service::v1::file_service_client::FileServiceClient::new(
        state.core_channel.clone(),
    );
    core.create(with_operator(ctx, req))
        .await
        .map_err(map_status)?;
    Ok(Empty {})
}

/// The derived-permission rebuild trigger (the reference scans the
/// enabled menus and rebuilds the business permission tables core-side).
pub async fn sync_permissions(
    state: &AppState,
    ctx: &Ctx,
    _req: Empty,
) -> Result<Empty, StatusError> {
    let mut core =
        proto::proto::permission::service::v1::permission_service_client::PermissionServiceClient::new(
            state.core_channel.clone(),
        );
    core.sync_permissions(with_operator(
        ctx,
        proto::proto::permission::service::v1::SyncPermissionsRequest::default(),
    ))
    .await
    .map_err(map_status)?;
    Ok(Empty {})
}

/// The walk-route debug dump: this binary's own registered route table
/// (the shadowed entries the mux cannot reach are skipped — the
/// reference walks its live router the same way).
pub async fn walk_route_data(
    _state: &AppState,
    _ctx: &Ctx,
    _req: Empty,
) -> Result<proto::proto::permission::service::v1::ListApiResponse, StatusError> {
    Ok(route_table())
}

/// The route-table projection behind the walk-route dump (pure, so the
/// shadow filtering stays testable).
fn route_table() -> proto::proto::permission::service::v1::ListApiResponse {
    use proto::proto::permission::service::v1::api;
    let mut items = Vec::new();
    for (i, r) in proto::gen_admin::routes::ROUTES.iter().enumerate() {
        if r.shadowed {
            continue;
        }
        items.push(proto::proto::permission::service::v1::Api {
            id: Some((i as u32) + 1),
            path: Some(r.path.to_string()),
            method: Some(r.method.to_string()),
            status: Some(api::Status::On as i32),
            ..Default::default()
        });
    }
    let total = items.len() as u64;
    proto::proto::permission::service::v1::ListApiResponse { items, total }
}

/// The profile password change: the identifier is the operator's own
/// username; the admin dialog posts cleartext (no NeedDecrypt — the
/// core hashes both sides), mirroring the reference's admin variant.
pub async fn change_password(
    state: &AppState,
    ctx: &Ctx,
    req: proto::proto::identity::service::v1::ChangePasswordRequest,
) -> Result<Empty, StatusError> {
    let op = operator_of(ctx)?;
    let mut core = proto::proto::authentication::service::v1::user_credential_service_client::UserCredentialServiceClient::new(
        state.core_channel.clone(),
    );
    core.change_credential(with_operator(
        ctx,
        proto::proto::authentication::service::v1::ChangeCredentialRequest {
            identity_type:
                proto::proto::authentication::service::v1::user_credential::IdentityType::Username
                    as i32,
            identifier: op.username,
            old_credential: req.old_password,
            new_credential: req.new_password,
            need_decrypt: false,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_table_skips_shadowed_and_counts() {
        let table = route_table();
        let live = proto::gen_admin::routes::ROUTES
            .iter()
            .filter(|r| !r.shadowed)
            .count();
        assert_eq!(table.total as usize, live);
        // every shadowed entry is absent; ids are dense from 1
        assert!(table.items.iter().all(|i| i.status == Some(1)));
        assert_eq!(table.items.len(), table.total as usize);
        let paths: Vec<_> = table
            .items
            .iter()
            .map(|i| i.path.clone().unwrap_or_default())
            .collect();
        assert!(paths.iter().any(|p| p.contains("/admin/v1/")));
    }
}
