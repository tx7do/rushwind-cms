//! The shared login-chain queries — both BFF deployments ride these:
//! credential verification (AES unwrap + bcrypt + timing equalizer),
//! the authority resolution (user → roles → permissions gate + role
//! codes), and the C-side register flow.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};

use crate::crypto;
use crate::entities::{
    sys_permissions, sys_role_permissions, sys_roles, sys_user_credentials, sys_user_roles,
    sys_users,
};

/// The verified-operator payload — the JWT claim set of the reference's
/// `UserTokenPayload` (pkg/jwt/user_token_payload.go), field-for-field.
/// The core service mints it; the BFF deployments parse it back off the
/// verified claim bag.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct UserTokenPayload {
    pub user_id: u32,
    pub tenant_id: u32,
    pub username: String,
    pub jti: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub org_unit_id: Option<u32>,
    #[serde(default)]
    pub data_scope: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub device_id: Option<String>,
}

impl UserTokenPayload {
    /// Reads the payload off a verified claim bag.
    pub fn from_claims(claims: &serde_json::Map<String, serde_json::Value>) -> Option<Self> {
        let user_id = claims.get("uid").and_then(|v| v.as_u64())? as u32;
        if user_id == 0 {
            return None;
        }
        Some(Self {
            user_id,
            tenant_id: claims.get("tid").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            username: claims
                .get("sub")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            jti: claims
                .get("jti")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            roles: claims
                .get("roc")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
            org_unit_id: claims
                .get("ouid")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            data_scope: claims.get("ds").and_then(|v| v.as_str()).map(str::to_owned),
            client_id: claims
                .get("cid")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            device_id: claims
                .get("did")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
        })
    }

    /// The access-token claim set (the reference's
    /// NewUserTokenAuthClaims): sub/uid/tid/iat/exp/jti/roc/did/cid/ds/
    /// ouid.
    pub fn to_access_claims(
        &self,
        issued_at: i64,
        expires_at: i64,
    ) -> serde_json::Map<String, serde_json::Value> {
        let mut claims = serde_json::Map::new();
        claims.insert("sub".into(), self.username.clone().into());
        claims.insert("uid".into(), self.user_id.into());
        claims.insert("tid".into(), self.tenant_id.into());
        claims.insert("iat".into(), issued_at.into());
        claims.insert("exp".into(), expires_at.into());
        claims.insert("jti".into(), self.jti.clone().into());
        if !self.roles.is_empty() {
            claims.insert("roc".into(), serde_json::Value::from(self.roles.clone()));
        }
        if let Some(did) = &self.device_id {
            claims.insert("did".into(), did.clone().into());
        }
        if let Some(cid) = &self.client_id {
            claims.insert("cid".into(), cid.clone().into());
        }
        if let Some(ds) = &self.data_scope {
            claims.insert("ds".into(), ds.clone().into());
        }
        if let Some(ouid) = self.org_unit_id {
            claims.insert("ouid".into(), ouid.into());
        }
        claims
    }
}

/// The authority bundle of a user (login response enrichment).
#[allow(dead_code)]
pub struct Authority {
    pub user_id: i64,
    pub tenant_id: i64,
    pub username: String,
    pub role_codes: Vec<String>,
}

/// The credential verification of the reference's FindUserCredential:
/// base64+AES unwrap → single-row query within the tenant → bcrypt
/// compare, with the dummy-hash equalizer on every miss and the
/// USER_NOT_FOUND / INVALID_PASSWORD anti-enumeration split.
///
/// `identifier.0` is the identity type ("USERNAME" | "EMAIL"); tenant 0
/// scans the platform row.
pub async fn verify_credential(
    db: &DatabaseConnection,
    tenant_id: i64,
    identifier: (&str, String),
    encrypted_password: &str,
) -> Result<i64, rushwind_http_binding::envelope::StatusError> {
    fn unauthorized(
        reason: &'static str,
        msg: &str,
    ) -> rushwind_http_binding::envelope::StatusError {
        rushwind_http_binding::envelope::StatusError::new(401, reason, msg)
    }

    let plain = match crypto::decrypt_login_credential(encrypted_password) {
        Ok(p) => p,
        Err(_) => return Err(unauthorized("BAD_REQUEST", "invalid credential format")),
    };

    let row = sys_user_credentials::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(sys_user_credentials::Column::TenantId.eq(tenant_id))
                .add(sys_user_credentials::Column::IdentityType.eq(identifier.0))
                .add(sys_user_credentials::Column::Identifier.eq(identifier.1.clone())),
        )
        .one(db)
        .await
        .map_err(crate::db_err)?;

    let Some(row) = row else {
        crypto::dummy_verify();
        return Err(unauthorized("USER_NOT_FOUND", "user not found"));
    };
    if row.status.as_deref() != Some("ENABLED") {
        crypto::dummy_verify();
        return Err(unauthorized("USER_NOT_FOUND", "user not found"));
    }
    let stored = row.credential.unwrap_or_default();
    if !crypto::verify_password(&plain, &stored) {
        return Err(unauthorized("INVALID_PASSWORD", "incorrect password"));
    }
    Ok(row.user_id.unwrap_or(0))
}

/// The credential query WITHOUT the AES layer (the register flow stores
/// bcrypt-hashed passwords; verification paths that already hold the
/// plaintext use this).
#[allow(dead_code)]
pub async fn verify_credential_plain(
    db: &DatabaseConnection,
    tenant_id: i64,
    identifier: (&str, String),
    password: &str,
) -> Result<i64, rushwind_http_binding::envelope::StatusError> {
    fn unauthorized(
        reason: &'static str,
        msg: &str,
    ) -> rushwind_http_binding::envelope::StatusError {
        rushwind_http_binding::envelope::StatusError::new(401, reason, msg)
    }
    let row = sys_user_credentials::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(sys_user_credentials::Column::TenantId.eq(tenant_id))
                .add(sys_user_credentials::Column::IdentityType.eq(identifier.0))
                .add(sys_user_credentials::Column::Identifier.eq(identifier.1)),
        )
        .one(db)
        .await
        .map_err(crate::db_err)?;
    let Some(row) = row else {
        crypto::dummy_verify();
        return Err(unauthorized("USER_NOT_FOUND", "user not found"));
    };
    let stored = row.credential.unwrap_or_default();
    if !crypto::verify_password(password, &stored) {
        return Err(unauthorized("INVALID_PASSWORD", "incorrect password"));
    }
    Ok(row.user_id.unwrap_or(0))
}

/// The role codes of a user's active role bindings.
pub async fn role_codes_of_user(
    db: &DatabaseConnection,
    user_id: i64,
) -> Result<Vec<String>, sea_orm::DbErr> {
    let role_ids: Vec<i64> = sys_user_roles::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(sys_user_roles::Column::UserId.eq(user_id))
                .add(sys_user_roles::Column::Status.eq("ACTIVE")),
        )
        .all(db)
        .await?
        .into_iter()
        .map(|r| r.role_id)
        .collect();
    if role_ids.is_empty() {
        return Ok(Vec::new());
    }
    let roles = sys_roles::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(sys_roles::Column::Id.is_in(role_ids))
                .add(sys_roles::Column::Status.eq("ON")),
        )
        .all(db)
        .await?;
    Ok(roles.into_iter().filter_map(|r| r.code).collect())
}

/// The authority resolution of the reference's
/// authorizeAndEnrichUserTokenPayload: the login permission gate (the
/// user's roles' permissions must carry `required_permission`), then
/// the role-code enrichment. Fails FORBIDDEN on any miss.
pub async fn resolve_authority(
    db: &DatabaseConnection,
    user_id: i64,
    required_permission: &str,
    role_codes: &mut Vec<String>,
) -> Result<(), rushwind_http_binding::envelope::StatusError> {
    fn forbidden(msg: &str) -> rushwind_http_binding::envelope::StatusError {
        rushwind_http_binding::envelope::StatusError::new(403, "FORBIDDEN", msg)
    }

    let role_ids: Vec<i64> = sys_user_roles::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(sys_user_roles::Column::UserId.eq(user_id))
                .add(sys_user_roles::Column::Status.eq("ACTIVE")),
        )
        .all(db)
        .await
        .map_err(crate::db_err)?
        .into_iter()
        .map(|r| r.role_id)
        .collect();
    if role_ids.is_empty() {
        return Err(forbidden("insufficient authority"));
    }

    let permission_ids: Vec<i64> = sys_role_permissions::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(sys_role_permissions::Column::RoleId.is_in(role_ids.clone()))
                .add(sys_role_permissions::Column::Status.eq("ON")),
        )
        .all(db)
        .await
        .map_err(crate::db_err)?
        .into_iter()
        .map(|r| r.permission_id)
        .collect();
    if permission_ids.is_empty() {
        return Err(forbidden("insufficient authority"));
    }

    let codes: Vec<String> = sys_permissions::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(sys_permissions::Column::Id.is_in(permission_ids))
                .add(sys_permissions::Column::Status.eq("ON")),
        )
        .all(db)
        .await
        .map_err(crate::db_err)?
        .into_iter()
        .map(|p| p.code)
        .collect();
    if !codes.iter().any(|c| c == required_permission) {
        return Err(forbidden("insufficient authority"));
    }

    let roles = sys_roles::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(sys_roles::Column::Id.is_in(role_ids))
                .add(sys_roles::Column::Status.eq("ON")),
        )
        .all(db)
        .await
        .map_err(crate::db_err)?;
    *role_codes = roles.into_iter().filter_map(|r| r.code).collect();
    if role_codes.is_empty() {
        return Err(forbidden("insufficient authority"));
    }
    Ok(())
}

/// The C-side register flow (the reference's RegisterUser): tenant
/// resolve (register under a tenant when tenant-mode data carries one;
/// platform otherwise), user + USERNAME credential in one transaction,
/// and the tenant's `tenant:user` role binding when present.
/// The password arrives ALREADY AES-encrypted (the same front-end
/// layer as login) — it is unwrapped and bcrypt-hashed here.
pub async fn register_user(
    db: &DatabaseConnection,
    tenant_code: &str,
    username: &str,
    encrypted_password: &str,
    email: Option<&str>,
) -> Result<i64, rushwind_http_binding::envelope::StatusError> {
    use rushwind_http_binding::envelope::StatusError;

    let tenant_id: i64 = if tenant_code.trim().is_empty() {
        0
    } else {
        let t = crate::entities::sys_tenants::Entity::find()
            .filter(crate::entities::sys_tenants::Column::Code.eq(tenant_code.trim()))
            .one(db)
            .await
            .map_err(crate::db_err)?;
        t.map(|t| t.id).unwrap_or(0)
    };

    let plain = crypto::decrypt_login_credential(encrypted_password)
        .map_err(|_| StatusError::new(400, "BAD_REQUEST", "invalid credential format"))?;

    // Duplicate guard: the username must be free within the tenant.
    let dupe = sys_user_credentials::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(sys_user_credentials::Column::TenantId.eq(tenant_id))
                .add(sys_user_credentials::Column::IdentityType.eq("USERNAME"))
                .add(sys_user_credentials::Column::Identifier.eq(username)),
        )
        .one(db)
        .await
        .map_err(crate::db_err)?;
    if dupe.is_some() {
        return Err(StatusError::new(409, "CONFLICT", "username already exists"));
    }

    let hashed = crypto::hash_password(&plain).map_err(|e| crate::db_err_internal(&e))?;

    let txn = db.begin().await.map_err(crate::db_err)?;
    let user = sys_users::ActiveModel {
        tenant_id: Set(Some(tenant_id)),
        username: Set(Some(username.to_string())),
        email: Set(email.map(str::to_string)),
        status: Set(Some("NORMAL".to_string())),
        created_at: Set(Some(crate::now())),
        updated_at: Set(Some(crate::now())),
        ..Default::default()
    };
    // The transaction rolls back on drop when the commit never runs.
    let user = user.insert(&txn).await.map_err(crate::db_err)?;

    let credential = sys_user_credentials::ActiveModel {
        tenant_id: Set(Some(tenant_id)),
        user_id: Set(Some(user.id)),
        identity_type: Set(Some("USERNAME".to_string())),
        identifier: Set(Some(username.to_string())),
        credential_type: Set(Some("PASSWORD_HASH".to_string())),
        credential: Set(Some(hashed)),
        is_primary: Set(Some(true)),
        status: Set(Some("ENABLED".to_string())),
        created_at: Set(Some(crate::now())),
        ..Default::default()
    };
    credential.insert(&txn).await.map_err(crate::db_err)?;

    // The C-side default role (tenant:user) when the tenant carries it
    // (the platform seed carries one at tenant 0 too).
    {
        let role = sys_roles::Entity::find()
            .filter(
                sea_orm::Condition::all()
                    .add(sys_roles::Column::TenantId.eq(tenant_id))
                    .add(sys_roles::Column::Code.eq("tenant:user")),
            )
            .one(&txn)
            .await
            .map_err(crate::db_err)?;
        if let Some(role) = role {
            let binding = sys_user_roles::ActiveModel {
                tenant_id: Set(Some(tenant_id)),
                user_id: Set(user.id),
                role_id: Set(role.id),
                is_primary: Set(true),
                status: Set("ACTIVE".to_string()),
                ..Default::default()
            };
            binding.insert(&txn).await.map_err(crate::db_err)?;
        }
    }

    txn.commit().await.map_err(crate::db_err)?;
    Ok(user.id)
}

/// The authority bundle of a user (login response enrichment).
#[allow(dead_code)]
pub async fn authority_of(
    db: &DatabaseConnection,
    user_id: i64,
) -> Result<Authority, sea_orm::DbErr> {
    let user = sys_users::Entity::find_by_id(user_id).one(db).await?;
    let role_codes = role_codes_of_user(db, user_id).await?;
    Ok(Authority {
        user_id,
        tenant_id: user.as_ref().and_then(|u| u.tenant_id).unwrap_or(0),
        username: user
            .as_ref()
            .and_then(|u| u.username.clone())
            .unwrap_or_default(),
        role_codes,
    })
}
