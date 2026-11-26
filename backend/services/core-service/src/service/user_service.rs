//! The user service — the complete user face (list/count/get plus the
//! write paths: create/update/delete with the credential row, the
//! password and existence checks), mirroring the reference's
//! user_service.go.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::data::user_repo;
use crate::service::context::{operator_of, operator_tenant_id};
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{
    sys_org_units, sys_positions, sys_roles, sys_tenants, sys_user_credentials, sys_user_org_units,
    sys_user_positions, sys_user_roles, sys_users,
};
use store::paging::fetch_paged;

use proto::proto::identity::service::v1 as identityv1;

/// Insert a USERNAME credential for a user (bcrypt of the given
/// password; no AES layer — administrative sets arrive in plaintext).
pub(crate) async fn insert_credential(
    db: &sea_orm::DatabaseTransaction,
    tenant_id: i64,
    user_id: i64,
    username: &str,
    password: &str,
) -> Result<(), Status> {
    let hashed = store::crypto::hash_password(password).map_err(Status::internal)?;
    sys_user_credentials::ActiveModel {
        tenant_id: Set(Some(tenant_id)),
        user_id: Set(Some(user_id)),
        identity_type: Set(Some("USERNAME".to_string())),
        identifier: Set(Some(username.to_string())),
        credential_type: Set(Some("PASSWORD_HASH".to_string())),
        credential: Set(Some(hashed)),
        is_primary: Set(Some(true)),
        status: Set(Some("ENABLED".to_string())),
        created_at: Set(Some(store::now())),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_status)?;
    Ok(())
}

/// The reference's constants.DefaultAdminUserName.
const DEFAULT_ADMIN_USER_NAME: &str = "admin";

/// 关联 id 收集：单值并入复数集（>0 才收、重复去重）——参照的
/// CreateWithTx/Update 收集语义。
fn merge_relation_ids(plural: &[i64], singular: Option<i64>) -> Vec<i64> {
    let mut ids = plural.to_vec();
    if let Some(id) = singular.filter(|v| *v > 0) {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

/// 组织单元绑定（create/update 共用，delete-then-insert 整组重建）。
/// 空列表 = 本次不动绑定（参照 AssignUserOrgUnits 的空集早退）；非空 =
/// 清后全量重建，行全部 ACTIVE / is_primary / assigned_at=start_at=now。
async fn sync_org_units(
    db: &sea_orm::DatabaseTransaction,
    tenant_id: i64,
    user_id: i64,
    org_unit_ids: &[i64],
) -> Result<(), Status> {
    let unique: Vec<i64> = org_unit_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if unique.is_empty() {
        return Ok(());
    }
    sys_user_org_units::Entity::delete_many()
        .filter(
            sea_orm::Condition::all()
                .add(sys_user_org_units::Column::TenantId.eq(tenant_id))
                .add(sys_user_org_units::Column::UserId.eq(user_id)),
        )
        .exec(db)
        .await
        .map_err(db_status)?;
    let now = store::now();
    for org_unit_id in &unique {
        sys_user_org_units::ActiveModel {
            tenant_id: Set(Some(tenant_id)),
            user_id: Set(user_id),
            org_unit_id: Set(*org_unit_id),
            is_primary: Set(true),
            status: Set(Some("ACTIVE".to_string())),
            assigned_at: Set(Some(now)),
            start_at: Set(Some(now)),
            created_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(db_status)?;
    }
    Ok(())
}

/// 岗位绑定（组织单元同款 delete-then-insert 语义）。
async fn sync_positions(
    db: &sea_orm::DatabaseTransaction,
    tenant_id: i64,
    user_id: i64,
    position_ids: &[i64],
) -> Result<(), Status> {
    let unique: Vec<i64> = position_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if unique.is_empty() {
        return Ok(());
    }
    sys_user_positions::Entity::delete_many()
        .filter(
            sea_orm::Condition::all()
                .add(sys_user_positions::Column::TenantId.eq(tenant_id))
                .add(sys_user_positions::Column::UserId.eq(user_id)),
        )
        .exec(db)
        .await
        .map_err(db_status)?;
    let now = store::now();
    for position_id in &unique {
        sys_user_positions::ActiveModel {
            tenant_id: Set(Some(tenant_id)),
            user_id: Set(user_id),
            position_id: Set(*position_id),
            is_primary: Set(Some(true)),
            status: Set("ACTIVE".to_string()),
            assigned_at: Set(Some(now)),
            start_at: Set(Some(now)),
            created_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(db_status)?;
    }
    Ok(())
}

/// 用户视图装配：绑定 id 集（角色/组织/岗位）按用户批量加载，名称与
/// 编码按去重后的 id 集各一把查回——list/get 共用（对位参照的
/// enrichRelations 聚合面；绑定 id 引用已删行时名字静默缺席）。
pub(crate) async fn user_views(
    db: &sea_orm::DatabaseConnection,
    rows: Vec<sys_users::Model>,
) -> Result<Vec<identityv1::User>, Status> {
    let user_ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let mut role_of: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    let mut org_of: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    let mut pos_of: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    if !user_ids.is_empty() {
        for r in sys_user_roles::Entity::find()
            .filter(sys_user_roles::Column::UserId.is_in(&user_ids))
            .all(db)
            .await
            .map_err(db_status)?
        {
            role_of.entry(r.user_id).or_default().push(r.role_id);
        }
        for r in sys_user_org_units::Entity::find()
            .filter(sys_user_org_units::Column::UserId.is_in(&user_ids))
            .all(db)
            .await
            .map_err(db_status)?
        {
            org_of.entry(r.user_id).or_default().push(r.org_unit_id);
        }
        for r in sys_user_positions::Entity::find()
            .filter(sys_user_positions::Column::UserId.is_in(&user_ids))
            .all(db)
            .await
            .map_err(db_status)?
        {
            pos_of.entry(r.user_id).or_default().push(r.position_id);
        }
    }

    let role_ids: BTreeSet<i64> = role_of.values().flatten().copied().collect();
    let roles: HashMap<i64, (Option<String>, Option<String>)> = if role_ids.is_empty() {
        HashMap::new()
    } else {
        sys_roles::Entity::find()
            .filter(sys_roles::Column::Id.is_in(role_ids))
            .all(db)
            .await
            .map_err(db_status)?
            .into_iter()
            .map(|r| (r.id, (r.name.clone(), r.code)))
            .collect()
    };
    let org_ids: BTreeSet<i64> = org_of.values().flatten().copied().collect();
    let orgs: HashMap<i64, String> = if org_ids.is_empty() {
        HashMap::new()
    } else {
        sys_org_units::Entity::find()
            .filter(sys_org_units::Column::Id.is_in(org_ids))
            .all(db)
            .await
            .map_err(db_status)?
            .into_iter()
            .map(|r| (r.id, r.name))
            .collect()
    };
    let pos_ids: BTreeSet<i64> = pos_of.values().flatten().copied().collect();
    let positions: HashMap<i64, String> = if pos_ids.is_empty() {
        HashMap::new()
    } else {
        sys_positions::Entity::find()
            .filter(sys_positions::Column::Id.is_in(pos_ids))
            .all(db)
            .await
            .map_err(db_status)?
            .into_iter()
            .map(|r| (r.id, r.name))
            .collect()
    };
    let tenant_ids: BTreeSet<i64> = rows.iter().filter_map(|r| r.tenant_id).collect();
    let tenants: HashMap<i64, String> = if tenant_ids.is_empty() {
        HashMap::new()
    } else {
        sys_tenants::Entity::find()
            .filter(sys_tenants::Column::Id.is_in(tenant_ids))
            .all(db)
            .await
            .map_err(db_status)?
            .into_iter()
            .filter_map(|t| t.name.map(|n| (t.id, n)))
            .collect()
    };

    Ok(rows
        .into_iter()
        .map(|row| {
            let mut u = user_proto(row.clone());
            let role_ids = role_of.remove(&row.id).unwrap_or_default();
            u.role_ids = role_ids.to_vec();
            u.roles = role_ids
                .iter()
                .filter_map(|id| roles.get(id).and_then(|(_, code)| code.clone()))
                .collect();
            u.role_names = role_ids
                .iter()
                .filter_map(|id| roles.get(id).and_then(|(name, _)| name.clone()))
                .collect();
            let org_unit_ids = org_of.remove(&row.id).unwrap_or_default();
            u.org_unit_ids = org_unit_ids.to_vec();
            u.org_unit_names = org_unit_ids
                .iter()
                .filter_map(|id| orgs.get(id).cloned())
                .collect();
            let position_ids = pos_of.remove(&row.id).unwrap_or_default();
            u.position_ids = position_ids.to_vec();
            u.position_names = position_ids
                .iter()
                .filter_map(|id| positions.get(id).cloned())
                .collect();
            u.tenant_name = row.tenant_id.and_then(|tid| tenants.get(&tid).cloned());
            u
        })
        .collect())
}

pub(crate) fn user_proto(r: sys_users::Model) -> identityv1::User {
    identityv1::User {
        id: Some(r.id),
        tenant_id: r.tenant_id,
        username: r.username,
        nickname: r.nickname,
        realname: r.realname,
        email: r.email,
        mobile: r.mobile,
        telephone: r.telephone,
        avatar: r.avatar,
        address: r.address,
        region: r.region,
        description: r.description,
        gender: r.gender.as_deref().map(|_| 1),
        last_login_at: r.last_login_at.and_then(ts_to_proto),
        last_login_ip: r.last_login_ip,
        status: Some(1),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

// ── User writes ──────────────────────────────────────────────────────

pub struct UserService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::user_service_server::UserService for UserService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::ListUserResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_users::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        let items = user_views(&self.state.db, rows).await?;
        Ok(Response::new(identityv1::ListUserResponse {
            items,
            total: total as i64,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::CountUserResponse>, Status> {
        use sea_orm::PaginatorTrait as _;
        let total = sys_users::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(identityv1::CountUserResponse {
            count: total as i64,
        }))
    }

    async fn get(
        &self,
        request: Request<identityv1::GetUserRequest>,
    ) -> Result<Response<identityv1::User>, Status> {
        let req = request.into_inner();
        let Some(identityv1::get_user_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = user_repo::users_by_id(&self.state.db, id).await?;
        let mut views = user_views(&self.state.db, vec![row]).await?;
        let view = views.pop().ok_or_else(|| not_found("user"))?;
        Ok(Response::new(view))
    }

    async fn create(
        &self,
        request: Request<identityv1::CreateUserRequest>,
    ) -> Result<Response<identityv1::User>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let password = req.password.unwrap_or_else(|| "12345678".to_string());
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let row = sys_users::ActiveModel {
            tenant_id: Set(data.tenant_id),
            username: Set(Some(data.username.clone().unwrap_or_default())),
            nickname: Set(data.nickname),
            realname: Set(data.realname),
            email: Set(data.email),
            mobile: Set(data.mobile),
            avatar: Set(data.avatar),
            status: Set(Some("NORMAL".to_string())),
            created_at: Set(Some(store::now())),
            updated_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;
        insert_credential(
            &txn,
            row.tenant_id.unwrap_or(0),
            row.id,
            &row.username.clone().unwrap_or_default(),
            &password,
        )
        .await?;
        // Role bindings (default: the tenant:user role, so created
        // users can actually sign in).
        let role_ids: Vec<i64> = if data.role_ids.is_empty() {
            sys_roles::Entity::find()
                .filter(
                    sea_orm::Condition::all()
                        .add(sys_roles::Column::TenantId.eq(row.tenant_id.unwrap_or(0)))
                        .add(sys_roles::Column::Code.eq("tenant:user")),
                )
                .one(&txn)
                .await
                .map_err(db_status)?
                .map(|r| vec![r.id])
                .unwrap_or_default()
        } else {
            data.role_ids.clone()
        };
        for role_id in role_ids {
            sys_user_roles::ActiveModel {
                tenant_id: Set(row.tenant_id),
                user_id: Set(row.id),
                role_id: Set(role_id),
                is_primary: Set(true),
                status: Set("ACTIVE".to_string()),
                created_at: Set(Some(store::now())),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(db_status)?;
        }
        // 组织/岗位绑定（参照 CreateWithTx：单值并入复数集，>0 才收；
        // 空集 = 无行，非空 = 逐条 ACTIVE 绑定）。
        let org_unit_ids = merge_relation_ids(&data.org_unit_ids, data.org_unit_id);
        let position_ids = merge_relation_ids(&data.position_ids, data.position_id);
        for org_unit_id in org_unit_ids {
            sys_user_org_units::ActiveModel {
                tenant_id: Set(row.tenant_id),
                user_id: Set(row.id),
                org_unit_id: Set(org_unit_id),
                is_primary: Set(true),
                status: Set(Some("ACTIVE".to_string())),
                assigned_at: Set(Some(store::now())),
                start_at: Set(Some(store::now())),
                created_at: Set(Some(store::now())),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(db_status)?;
        }
        for position_id in position_ids {
            sys_user_positions::ActiveModel {
                tenant_id: Set(row.tenant_id),
                user_id: Set(row.id),
                position_id: Set(position_id),
                is_primary: Set(Some(true)),
                status: Set("ACTIVE".to_string()),
                assigned_at: Set(Some(store::now())),
                start_at: Set(Some(store::now())),
                created_at: Set(Some(store::now())),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(db_status)?;
        }
        txn.commit().await.map_err(db_status)?;
        let mut views = user_views(&self.state.db, vec![row]).await?;
        let view = views.pop().ok_or_else(|| not_found("user"))?;
        Ok(Response::new(view))
    }

    async fn update(
        &self,
        request: Request<identityv1::UpdateUserRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let data = req.data;
        let row = sys_users::Entity::find_by_id(req.id)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("user"))?;
        let mut a: sys_users::ActiveModel = row.into();
        if let Some(data) = &data {
            if let Some(v) = data.nickname.clone() {
                a.nickname = Set(Some(v));
            }
            if let Some(v) = data.realname.clone() {
                a.realname = Set(Some(v));
            }
            if let Some(v) = data.email.clone() {
                a.email = Set(Some(v));
            }
            if let Some(v) = data.mobile.clone() {
                a.mobile = Set(Some(v));
            }
            if let Some(v) = data.avatar.clone() {
                a.avatar = Set(Some(v));
            }
            if let Some(v) = data.status {
                a.status = Set(Some(if v == 1 { "NORMAL" } else { "DISABLED" }.to_string()));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = a.update(&self.state.db).await.map_err(db_status)?;
        if let Some(password) = req.password.filter(|p| !p.is_empty()) {
            let cred = sys_user_credentials::Entity::find()
                .filter(
                    sea_orm::Condition::all()
                        .add(sys_user_credentials::Column::UserId.eq(row.id))
                        .add(sys_user_credentials::Column::IdentityType.eq("USERNAME")),
                )
                .one(&self.state.db)
                .await
                .map_err(db_status)?
                .ok_or_else(|| not_found("credential"))?;
            let hashed = store::crypto::hash_password(&password).map_err(Status::internal)?;
            let mut ca: sys_user_credentials::ActiveModel = cred.into();
            ca.credential = Set(Some(hashed));
            ca.updated_at = Set(Some(store::now()));
            ca.update(&self.state.db).await.map_err(db_status)?;
        }
        // 组织/岗位绑定重建（参照 Update：载荷即全量提交语义；空集 =
        // 本次不动，非空 = 清后重建——单事务原子落库）。
        if let Some(data) = &data {
            let org_unit_ids = merge_relation_ids(&data.org_unit_ids, data.org_unit_id);
            let position_ids = merge_relation_ids(&data.position_ids, data.position_id);
            if !org_unit_ids.is_empty() || !position_ids.is_empty() {
                let tenant_id = row.tenant_id.unwrap_or(0);
                let txn = self.state.db.begin().await.map_err(db_status)?;
                sync_org_units(&txn, tenant_id, row.id, &org_unit_ids).await?;
                sync_positions(&txn, tenant_id, row.id, &position_ids).await?;
                txn.commit().await.map_err(db_status)?;
            }
        }
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<identityv1::DeleteUserRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        // The operator context is mandatory: an anonymous delete must
        // never pass (the reference errors before any lookup).
        let operator_id = operator_of(&request)?;
        let caller_tenant_id = operator_tenant_id(&request);
        let req = request.into_inner();
        let Some(identityv1::delete_user_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let target = sys_users::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("user"))?;
        // Tenant scope: the reference's viewer-predicated lookup hides
        // cross-tenant users from tenant operators.
        if caller_tenant_id > 0 && target.tenant_id.unwrap_or(0) != caller_tenant_id {
            return Err(not_found("user"));
        }

        // 禁止删除默认超级管理员：初始化时创建的平台级 admin（恒为 id=1，
        // 即便后续改名也由 id 兜底保护）。
        if target.id == 1
            || (target.username.as_deref() == Some(DEFAULT_ADMIN_USER_NAME)
                && target.tenant_id.unwrap_or(0) == 0)
        {
            return Err(bad("default admin cannot be deleted"));
        }
        // 禁止删除自己：误删自身账号将导致当前会话立即失去管理能力。
        if target.id == operator_id {
            return Err(bad("cannot delete yourself"));
        }

        // The reference cascades via its ent edges inside one tx; here
        // the relation rows go explicitly, same transaction.
        sys_users::Entity::delete_by_id(id)
            .exec(&txn)
            .await
            .map_err(db_status)?;
        sys_user_credentials::Entity::delete_many()
            .filter(sys_user_credentials::Column::UserId.eq(id))
            .exec(&txn)
            .await
            .map_err(db_status)?;
        sys_user_roles::Entity::delete_many()
            .filter(sys_user_roles::Column::UserId.eq(id))
            .exec(&txn)
            .await
            .map_err(db_status)?;
        // 组织/岗位绑定随行清除（参照 removeUserRelations 三面全清）。
        sys_user_org_units::Entity::delete_many()
            .filter(sys_user_org_units::Column::UserId.eq(id))
            .exec(&txn)
            .await
            .map_err(db_status)?;
        sys_user_positions::Entity::delete_many()
            .filter(sys_user_positions::Column::UserId.eq(id))
            .exec(&txn)
            .await
            .map_err(db_status)?;
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn user_exists(
        &self,
        request: Request<identityv1::UserExistsRequest>,
    ) -> Result<Response<identityv1::UserExistsResponse>, Status> {
        let req = request.into_inner();
        let row = match req.query_by {
            Some(identityv1::user_exists_request::QueryBy::Id(id)) => {
                sys_users::Entity::find_by_id(id).one(&self.state.db).await
            }
            Some(identityv1::user_exists_request::QueryBy::Username(username)) => {
                sys_users::Entity::find()
                    .filter(sys_users::Column::Username.eq(username))
                    .one(&self.state.db)
                    .await
            }
            _ => return Err(bad("query_by required")),
        }
        .map_err(db_status)?;
        Ok(Response::new(identityv1::UserExistsResponse {
            exist: row.is_some(),
        }))
    }
}
