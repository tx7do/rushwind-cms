//! The `dict_type` repository (dict_type_repo.go).

use sea_orm::{DatabaseConnection, EntityTrait};

use crate::state::{db_status, not_found_status, StatusResult};
use store::entities::sys_dict_types;

pub async fn dict_types_by_id(
    db: &DatabaseConnection,
    id: i64,
) -> StatusResult<sys_dict_types::Model> {
    sys_dict_types::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| not_found_status("dict types"))
}

pub async fn insert_dict_types(
    db: &DatabaseConnection,
    a: sys_dict_types::ActiveModel,
) -> StatusResult<sys_dict_types::Model> {
    use sea_orm::ActiveModelTrait as _;
    a.insert(db).await.map_err(db_status)
}

pub async fn update_dict_types(
    db: &DatabaseConnection,
    a: sys_dict_types::ActiveModel,
) -> StatusResult<sys_dict_types::Model> {
    use sea_orm::ActiveModelTrait as _;
    a.update(db).await.map_err(db_status)
}

pub async fn delete_dict_types(db: &DatabaseConnection, id: i64) -> StatusResult<()> {
    sys_dict_types::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(db_status)?;
    Ok(())
}
