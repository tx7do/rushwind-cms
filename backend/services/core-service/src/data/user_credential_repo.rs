//! userCredentialRepo — the data layer of the credential domain,
//! mirroring the reference's `user_credential_repo.go`: the paged
//! listing, the by-id/by-identifier fetches (tenant-scoped — platform
//! tenant 0 stays unscoped), the insert/update with the bcrypt
//! `prepareCredential` front, and the verify/change/reset trio with
//! the AES unwrap, the dummy-verify timing equalizer and the
//! ENABLED-only gate.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use tonic::Status;

use proto::proto::authentication::service::v1 as authv1;
use proto::proto::pagination::PagingRequest;
use store::entities::sys_user_credentials;
use store::entities::sys_user_credentials as entity;
use store::paging::fetch_paged;

use crate::state::{db_status, not_found_status, StatusResult};

/// The proto enum number → the varchar value the golden schema stores
/// (the proto string names ARE the stored values); unknown numbers
/// collapse to the zero value like the reference's enum converter.
pub fn identity_type_name(v: i32) -> Option<String> {
    use proto::proto::authentication::service::v1::user_credential::IdentityType;
    Some(
        IdentityType::try_from(v)
            .map(|e| e.as_str_name())
            .unwrap_or("USERNAME")
            .to_string(),
    )
}

pub fn credential_type_name(v: i32) -> Option<String> {
    use proto::proto::authentication::service::v1::user_credential::CredentialType;
    Some(
        CredentialType::try_from(v)
            .map(|e| e.as_str_name())
            .unwrap_or("TYPE_UNSPECIFIED")
            .to_string(),
    )
}

pub fn status_name(v: i32) -> Option<String> {
    use proto::proto::authentication::service::v1::user_credential::Status;
    Some(
        Status::try_from(v)
            .map(|e| e.as_str_name())
            .unwrap_or("DISABLED")
            .to_string(),
    )
}

fn bad(message: &str) -> Status {
    Status::invalid_argument(message.to_string())
}

/// The stored form of a new credential — bcrypt for PASSWORD_HASH,
/// verbatim otherwise (the reference's prepareCredential).
pub fn prepare_credential(credential_type: Option<&str>, plain: &str) -> Result<String, Status> {
    match credential_type {
        Some("PASSWORD_HASH") => {
            store::crypto::hash_password(plain).map_err(|_| bad("hash new password failed"))
        }
        _ => Ok(plain.to_string()),
    }
}

/// The stored-credential check — bcrypt for PASSWORD_HASH, plain
/// equality otherwise; an empty presented credential never matches
/// (the reference's verifyCredential).
fn verify_credential(credential_type: Option<&str>, plain: &str, stored: &str) -> bool {
    if plain.is_empty() {
        return false;
    }
    match credential_type {
        Some("PASSWORD_HASH") => store::crypto::verify_password(plain, stored),
        _ => plain == stored,
    }
}

/// The tenant scope of the caller — `Some` when a positive tenant id
/// rides the request (the viewer's tenant); the platform (tenant 0)
/// stays unscoped (the reference's maybeTenantFromViewer).
fn tenant_condition(cond: Condition, tenant: Option<i64>) -> Condition {
    match tenant.filter(|t| *t > 0) {
        Some(tid) => cond.add(entity::Column::TenantId.eq(tid)),
        None => cond,
    }
}

fn identifier_condition(identity_type: &str, identifier: &str, tenant: Option<i64>) -> Condition {
    tenant_condition(
        Condition::all()
            .add(entity::Column::IdentityType.eq(identity_type))
            .add(entity::Column::Identifier.eq(identifier)),
        tenant,
    )
}

/// The paged listing (the reference's List).
pub async fn list(
    db: &DatabaseConnection,
    req: &PagingRequest,
) -> StatusResult<(Vec<entity::Model>, u64)> {
    fetch_paged(db, entity::Entity::find(), req)
        .await
        .map_err(|e| Status::internal(e.message))
}

/// The by-id fetch (the reference's Get with the Id query).
pub async fn by_id(db: &DatabaseConnection, id: i64) -> StatusResult<entity::Model> {
    entity::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| crate::state::not_found_status("user credential"))
}

/// The by-identifier fetch with the tenant scope (the reference's
/// GetByIdentifier).
pub async fn by_identifier(
    db: &DatabaseConnection,
    identity_type: &str,
    identifier: &str,
    tenant: Option<i64>,
) -> StatusResult<entity::Model> {
    entity::Entity::find()
        .filter(identifier_condition(identity_type, identifier, tenant))
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| Status::not_found("user credential not found"))
}

/// The jsonb form of the extra-info string.
fn extra_info_json(value: &Option<String>) -> Option<sea_orm::prelude::Json> {
    value
        .as_ref()
        .map(|s| serde_json::from_str(s).unwrap_or(serde_json::Value::String(s.clone())))
}

/// The insert (the reference's CreateWithTx): the credential is
/// bcrypt-hashed up front when the type is PASSWORD_HASH.
pub async fn insert(
    db: &DatabaseConnection,
    data: authv1::UserCredential,
) -> StatusResult<entity::Model> {
    let credential = match data.credential.as_deref() {
        Some(c) => {
            let kind = data.credential_type.and_then(credential_type_name);
            Some(prepare_credential(kind.as_deref(), c)?)
        }
        None => None,
    };
    let row = entity::ActiveModel {
        tenant_id: Set(data.tenant_id.map(|v| v as i64)),
        user_id: Set(data.user_id.map(|v| v as i64)),
        identity_type: Set(data.identity_type.and_then(identity_type_name)),
        identifier: Set(data.identifier),
        credential_type: Set(data.credential_type.and_then(credential_type_name)),
        credential: Set(credential),
        is_primary: Set(data.is_primary),
        status: Set(data.status.and_then(status_name)),
        extra_info: Set(extra_info_json(&data.extra_info)),
        provider: Set(data.provider),
        provider_account_id: Set(data.provider_account_id),
        created_at: Set(Some(store::now())),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_status)?;
    Ok(row)
}

/// Whether the update-mask (json or proto names) selects a field; no
/// mask selects everything.
fn in_mask(mask: Option<&pbjson_types::FieldMask>, field: &str) -> bool {
    let Some(mask) = mask else {
        return true;
    };
    let mut camel = String::with_capacity(field.len());
    for (i, part) in field.split('_').enumerate() {
        if i == 0 {
            camel.push_str(part);
        } else {
            let mut chars = part.chars();
            if let Some(head) = chars.next() {
                camel.extend(head.to_uppercase());
                camel.push_str(chars.as_str());
            }
        }
    }
    mask.paths.iter().any(|p| p == field || p == camel.as_str())
}

/// The update (the reference's Update): allow_missing creates the row
/// when absent; the credential re-runs prepareCredential; the update
/// mask (when present) selects the applied fields; user_id/tenant_id
/// never move.
pub async fn update(
    db: &DatabaseConnection,
    req: authv1::UpdateUserCredentialRequest,
) -> StatusResult<()> {
    let Some(data) = req.data else {
        return Err(bad("invalid request"));
    };
    let exists = entity::Entity::find_by_id(req.id as i64)
        .one(db)
        .await
        .map_err(db_status)?
        .is_some();
    if req.allow_missing == Some(true) && !exists {
        insert(db, data).await?;
        return Ok(());
    }
    let row = by_id(db, req.id as i64).await?;

    let credential = if in_mask(req.update_mask.as_ref(), "credential") {
        match data.credential.as_deref() {
            Some(c) => {
                let kind = data.credential_type.and_then(credential_type_name);
                Some(prepare_credential(kind.as_deref(), c)?)
            }
            None => None,
        }
    } else {
        None
    };

    let mut a: entity::ActiveModel = row.into();
    if in_mask(req.update_mask.as_ref(), "identity_type") {
        if let Some(v) = data.identity_type {
            a.identity_type = Set(identity_type_name(v));
        }
    }
    if in_mask(req.update_mask.as_ref(), "identifier") {
        if let Some(v) = data.identifier {
            a.identifier = Set(Some(v));
        }
    }
    if in_mask(req.update_mask.as_ref(), "credential_type") {
        if let Some(v) = data.credential_type {
            a.credential_type = Set(credential_type_name(v));
        }
    }
    if let Some(credential) = credential {
        a.credential = Set(Some(credential));
    }
    if in_mask(req.update_mask.as_ref(), "is_primary") {
        if let Some(v) = data.is_primary {
            a.is_primary = Set(Some(v));
        }
    }
    if in_mask(req.update_mask.as_ref(), "status") {
        if let Some(v) = data.status {
            a.status = Set(status_name(v));
        }
    }
    if in_mask(req.update_mask.as_ref(), "extra_info") {
        a.extra_info = Set(extra_info_json(&data.extra_info));
    }
    if in_mask(req.update_mask.as_ref(), "provider") {
        if let Some(v) = data.provider {
            a.provider = Set(Some(v));
        }
    }
    if in_mask(req.update_mask.as_ref(), "provider_account_id") {
        if let Some(v) = data.provider_account_id {
            a.provider_account_id = Set(Some(v));
        }
    }
    a.updated_at = Set(Some(store::now()));
    a.update(db).await.map_err(db_status)?;
    Ok(())
}

/// The delete (the reference's Delete) — zero affected rows is a miss.
pub async fn delete(db: &DatabaseConnection, id: i64) -> StatusResult<()> {
    let res = entity::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(db_status)?;
    if res.rows_affected == 0 {
        return Err(crate::state::not_found_status("user credential"));
    }
    Ok(())
}

/// The credential check of the reference's FindUserCredential: locate
/// the row inside the tenant scope, gate on ENABLED, bcrypt-compare —
/// with the dummy verify on every miss so response times do not leak
/// existence. Returns the matched user id.
pub async fn find_user_credential(
    db: &DatabaseConnection,
    tenant_id: i64,
    identity_type: &str,
    identifier: &str,
    credential: &str,
    need_decrypt: bool,
) -> Result<i64, Status> {
    let plain = if need_decrypt {
        store::crypto::decrypt_login_credential(credential)
            .map_err(|_| bad("invalid credential format"))?
    } else {
        credential.to_string()
    };

    let row = entity::Entity::find()
        .filter(identifier_condition(
            identity_type,
            identifier,
            Some(tenant_id),
        ))
        .one(db)
        .await
        .map_err(|_| Status::unavailable("db error"))?;

    let miss = || Status::not_found("user not found");
    let Some(row) = row else {
        store::crypto::dummy_verify();
        return Err(miss());
    };
    if row.status.as_deref() != Some("ENABLED")
        || row.credential_type.is_none()
        || row.credential.is_none()
    {
        store::crypto::dummy_verify();
        return Err(miss());
    }
    let user_id = row.user_id.unwrap_or(0);
    if user_id == 0 {
        store::crypto::dummy_verify();
        return Err(miss());
    }

    if verify_credential(
        row.credential_type.as_deref(),
        &plain,
        row.credential.as_deref().unwrap_or_default(),
    ) {
        return Ok(user_id);
    }
    Err(bad("incorrect password"))
}

/// The change (the reference's ChangeCredential): both credentials
/// unwrap when need_decrypt, the old one verifies against the stored
/// row, then the new one hashes in — all inside the caller's tenant
/// scope.
pub async fn change_credential(
    db: &DatabaseConnection,
    tenant: Option<i64>,
    identity_type: i32,
    identifier: &str,
    old_credential: &str,
    new_credential: &str,
    need_decrypt: bool,
) -> StatusResult<()> {
    let decrypt = |value: &str, what: &str| -> Result<String, Status> {
        if !need_decrypt {
            return Ok(value.to_string());
        }
        store::crypto::decrypt_login_credential(value)
            .map_err(|_| bad(&format!("invalid {what} credential format")))
    };
    let old_plain = decrypt(old_credential, "old")?;
    let new_plain = decrypt(new_credential, "new")?;

    let type_name = identity_type_name(identity_type).unwrap_or_else(|| "USERNAME".to_string());
    let row = entity::Entity::find()
        .filter(identifier_condition(&type_name, identifier, tenant))
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| Status::not_found("user credential not found"))?;

    if !verify_credential(
        row.credential_type.as_deref(),
        &old_plain,
        row.credential.as_deref().unwrap_or_default(),
    ) {
        return Err(bad("invalid old password"));
    }

    let prepared = prepare_credential(row.credential_type.as_deref(), &new_plain)?;
    if prepared.is_empty() {
        return Err(bad("new credential cannot be empty"));
    }

    let mut a: entity::ActiveModel = row.into();
    a.credential = Set(Some(prepared));
    a.updated_at = Set(Some(store::now()));
    a.update(db).await.map_err(db_status)?;
    Ok(())
}

/// The reset (the reference's ResetCredential): the change without the
/// old-credential check (the forgot-password face).
pub async fn reset_credential(
    db: &DatabaseConnection,
    tenant: Option<i64>,
    identity_type: i32,
    identifier: &str,
    new_credential: &str,
    need_decrypt: bool,
) -> StatusResult<()> {
    let new_plain = if need_decrypt {
        store::crypto::decrypt_login_credential(new_credential)
            .map_err(|_| bad("invalid new credential format"))?
    } else {
        new_credential.to_string()
    };

    let type_name = identity_type_name(identity_type).unwrap_or_else(|| "USERNAME".to_string());
    let row = entity::Entity::find()
        .filter(identifier_condition(&type_name, identifier, tenant))
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| Status::not_found("user credential not found"))?;

    let prepared = prepare_credential(row.credential_type.as_deref(), &new_plain)?;
    if prepared.is_empty() {
        return Err(bad("new credential cannot be empty"));
    }

    let mut a: entity::ActiveModel = row.into();
    a.credential = Set(Some(prepared));
    a.updated_at = Set(Some(store::now()));
    a.update(db).await.map_err(db_status)?;
    Ok(())
}
pub async fn user_credentials_by_id(
    db: &DatabaseConnection,
    id: i64,
) -> StatusResult<sys_user_credentials::Model> {
    sys_user_credentials::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| not_found_status("user credentials"))
}
pub async fn insert_user_credentials(
    db: &DatabaseConnection,
    a: sys_user_credentials::ActiveModel,
) -> StatusResult<sys_user_credentials::Model> {
    use sea_orm::ActiveModelTrait as _;
    a.insert(db).await.map_err(db_status)
}
pub async fn update_user_credentials(
    db: &DatabaseConnection,
    a: sys_user_credentials::ActiveModel,
) -> StatusResult<sys_user_credentials::Model> {
    use sea_orm::ActiveModelTrait as _;
    a.update(db).await.map_err(db_status)
}
pub async fn delete_user_credentials(db: &DatabaseConnection, id: i64) -> StatusResult<()> {
    sys_user_credentials::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(db_status)?;
    Ok(())
}
