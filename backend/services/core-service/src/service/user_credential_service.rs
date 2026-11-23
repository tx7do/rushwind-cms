//! The UserCredentialService — the credential admin face (the
//! reference's user_credential_service.go): the paged listing, the
//! by-id/by-identifier fetches, the CRUD with the bcrypt front, and
//! the verify/change/reset trio. Tenant scoping rides the forwarded
//! operator metadata — platform tenant 0 stays unscoped; the internal
//! VerifyCredential always checks in the platform scope like the
//! reference.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use authenticationv1::user_credential_service_server::UserCredentialService;
use proto::proto::authentication::service::v1 as authv1;
use proto::proto::authentication::service::v1 as authenticationv1;
use proto::proto::authentication::service::v1::user_credential::{
    CredentialType, IdentityType, Status as CredentialStatus,
};

use crate::data::user_credential_repo as repo;
use crate::state::{bad, ts_to_proto, AppState};

pub struct UserCredentialServiceImpl {
    pub state: Arc<AppState>,
}

/// The caller's tenant scope off the forwarded metadata — `Some` only
/// for a positive tenant (the reference's maybeTenantFromViewer).
fn scoped_tenant_of<T>(request: &Request<T>) -> Option<i64> {
    for name in ["x-md-global-tenant-id", "x-tenant-id"] {
        if let Some(v) = request.metadata().get(name).and_then(|v| v.to_str().ok()) {
            if let Ok(n) = v.parse::<i64>() {
                if n > 0 {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// The entity row → the proto message (the varchar enum values read
/// back through their string names).
fn credential_proto(r: store::entities::sys_user_credentials::Model) -> authv1::UserCredential {
    authv1::UserCredential {
        id: r.id,
        user_id: r.user_id,
        tenant_id: r.tenant_id,
        identity_type: r
            .identity_type
            .as_deref()
            .and_then(IdentityType::from_str_name)
            .map(|e| e as i32),
        identifier: r.identifier,
        credential_type: r
            .credential_type
            .as_deref()
            .and_then(CredentialType::from_str_name)
            .map(|e| e as i32),
        credential: r.credential,
        is_primary: r.is_primary,
        status: r
            .status
            .as_deref()
            .and_then(CredentialStatus::from_str_name)
            .map(|e| e as i32),
        extra_info: r.extra_info.and_then(|v| serde_json::to_string(&v).ok()),
        provider: r.provider,
        provider_account_id: r.provider_account_id,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

#[async_trait::async_trait]
impl UserCredentialService for UserCredentialServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<authv1::ListUserCredentialResponse>, Status> {
        let (rows, total) = repo::list(&self.state.db, &request.into_inner()).await?;
        Ok(Response::new(authv1::ListUserCredentialResponse {
            items: rows.into_iter().map(credential_proto).collect(),
            total: total as i64,
        }))
    }

    async fn get(
        &self,
        request: Request<authv1::GetUserCredentialRequest>,
    ) -> Result<Response<authv1::UserCredential>, Status> {
        let req = request.into_inner();
        let Some(authv1::get_user_credential_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::by_id(&self.state.db, id).await?;
        Ok(Response::new(credential_proto(row)))
    }

    async fn get_by_identifier(
        &self,
        request: Request<authv1::GetUserCredentialByIdentifierRequest>,
    ) -> Result<Response<authv1::UserCredential>, Status> {
        let tenant = scoped_tenant_of(&request);
        let req = request.into_inner();
        let row = repo::by_identifier(
            &self.state.db,
            &repo::identity_type_name(req.identity_type).unwrap_or_default(),
            &req.identifier,
            tenant,
        )
        .await?;
        Ok(Response::new(credential_proto(row)))
    }

    async fn create(
        &self,
        request: Request<authv1::CreateUserCredentialRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let Some(data) = request.into_inner().data else {
            return Err(bad("invalid parameter"));
        };
        repo::insert(&self.state.db, data).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn update(
        &self,
        request: Request<authv1::UpdateUserCredentialRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        repo::update(&self.state.db, request.into_inner()).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<authv1::DeleteUserCredentialRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        // Only the Id target deletes — any other target resolves to a
        // zero id and lands on the not-found face like the reference.
        let Some(authv1::delete_user_credential_request::QueryBy::Id(id)) =
            request.into_inner().query_by
        else {
            return Err(crate::state::not_found_status("user credential"));
        };
        repo::delete(&self.state.db, id).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn verify_credential(
        &self,
        request: Request<authv1::VerifyCredentialRequest>,
    ) -> Result<Response<authv1::VerifyCredentialResponse>, Status> {
        let req = request.into_inner();
        // The internal check runs in the platform scope (tenant 0) —
        // the request carries no tenant context.
        repo::find_user_credential(
            &self.state.db,
            0,
            &repo::identity_type_name(req.identity_type).unwrap_or_default(),
            &req.identifier,
            &req.credential,
            req.need_decrypt,
        )
        .await?;
        Ok(Response::new(authv1::VerifyCredentialResponse {
            success: true,
        }))
    }

    async fn change_credential(
        &self,
        request: Request<authv1::ChangeCredentialRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let tenant = scoped_tenant_of(&request);
        let req = request.into_inner();
        repo::change_credential(
            &self.state.db,
            tenant,
            req.identity_type,
            &req.identifier,
            &req.old_credential,
            &req.new_credential,
            req.need_decrypt,
        )
        .await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn reset_credential(
        &self,
        request: Request<authv1::ResetCredentialRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let tenant = scoped_tenant_of(&request);
        let req = request.into_inner();
        repo::reset_credential(
            &self.state.db,
            tenant,
            req.identity_type,
            &req.identifier,
            &req.new_credential,
            req.need_decrypt,
        )
        .await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_with(headers: &[(&'static str, &'static str)]) -> Request<()> {
        let mut request = Request::new(());
        for (name, value) in headers {
            request.metadata_mut().insert(*name, value.parse().unwrap());
        }
        request
    }

    #[test]
    fn tenant_scope_reads_the_trusted_stamp_then_the_operator_bag() {
        // The anonymous chain's trusted stamp leads ...
        assert_eq!(
            scoped_tenant_of(&request_with(&[
                ("x-md-global-tenant-id", "5"),
                ("x-tenant-id", "7")
            ])),
            Some(5)
        );
        // ... else the operator bag.
        assert_eq!(
            scoped_tenant_of(&request_with(&[("x-tenant-id", "7")])),
            Some(7)
        );
        // The platform scope (0) is "no scope": a stamp must be positive to lead.
        assert_eq!(
            scoped_tenant_of(&request_with(&[
                ("x-md-global-tenant-id", "0"),
                ("x-tenant-id", "7")
            ])),
            Some(7)
        );
        // Nothing named: unscoped (the platform caller) — never negative.
        let unscoped: [&[(&str, &str)]; 4] = [
            &[],
            &[("x-tenant-id", "0")],
            &[("x-tenant-id", "-3")],
            &[("x-md-global-tenant-id", "abc"), ("x-tenant-id", "")],
        ];
        for headers in unscoped {
            assert_eq!(
                scoped_tenant_of(&request_with(headers)),
                None,
                "{headers:?}"
            );
        }
    }

    /// A full credential row (the golden schema's 21 columns).
    fn row() -> store::entities::sys_user_credentials::Model {
        store::entities::sys_user_credentials::Model {
            id: 9,
            created_at: None,
            updated_at: None,
            deleted_at: None,
            tenant_id: Some(3),
            user_id: Some(42),
            identity_type: Some("EMAIL".into()),
            identifier: Some("alice@example.com".into()),
            credential_type: Some("PASSWORD_HASH".into()),
            credential: Some("hash".into()),
            is_primary: Some(true),
            status: Some("ENABLED".into()),
            extra_info: Some(serde_json::json!({"k": "v"})),
            provider: Some("local".into()),
            provider_account_id: Some("acc-1".into()),
            activate_token_hash: None,
            activate_token_expires_at: None,
            activate_token_used_at: None,
            reset_token_hash: None,
            reset_token_expires_at: None,
            reset_token_used_at: None,
        }
    }

    #[test]
    fn credential_proto_reads_the_varchar_enums_back_through_their_names() {
        let p = credential_proto(row());
        // The id is a plain int64 (never optional in the proto).
        assert_eq!(p.id, 9);
        assert_eq!(p.tenant_id, Some(3));
        assert_eq!(p.user_id, Some(42));
        assert_eq!(
            p.identity_type,
            IdentityType::from_str_name("EMAIL").map(|e| e as i32)
        );
        assert_eq!(
            p.credential_type,
            CredentialType::from_str_name("PASSWORD_HASH").map(|e| e as i32)
        );
        assert_eq!(
            p.status,
            CredentialStatus::from_str_name("ENABLED").map(|e| e as i32)
        );
        assert_eq!(p.identifier.as_deref(), Some("alice@example.com"));
        assert_eq!(p.credential.as_deref(), Some("hash"));
        assert_eq!(p.is_primary, Some(true));
        // The JSON column rides as its text form.
        assert_eq!(p.extra_info.as_deref(), Some(r#"{"k":"v"}"#));
        assert_eq!(p.provider.as_deref(), Some("local"));
        assert_eq!(p.provider_account_id.as_deref(), Some("acc-1"));

        // A varchar value outside the enum reads as absent — never a
        // guessed ordinal.
        let p = credential_proto(store::entities::sys_user_credentials::Model {
            status: Some("FROZEN".into()),
            identity_type: None,
            ..row()
        });
        assert_eq!(p.status, None);
        assert_eq!(p.identity_type, None);

        // Every declared status name lands on its own ordinal.
        for v in 0..=20 {
            if let Ok(e) = CredentialStatus::try_from(v) {
                let p = credential_proto(store::entities::sys_user_credentials::Model {
                    status: Some(e.as_str_name().into()),
                    ..row()
                });
                assert_eq!(p.status, Some(v), "{e:?}");
            }
        }
    }
}
