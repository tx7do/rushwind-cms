//! The `dict_entry` repository (dict_entry_repo.go).

use sea_orm::{DatabaseConnection, EntityTrait};

use crate::state::{db_status, not_found_status, StatusResult};
use store::entities::sys_dict_entries;


pub async fn dict_entries_by_id(
    db: &DatabaseConnection,
    id: i64,
) -> StatusResult<sys_dict_entries::Model> {
    sys_dict_entries::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| not_found_status("dict entries"))
}

pub async fn insert_dict_entries(
    db: &DatabaseConnection,
    a: sys_dict_entries::ActiveModel,
) -> StatusResult<sys_dict_entries::Model> {
    use sea_orm::ActiveModelTrait as _;
    a.insert(db).await.map_err(db_status)
}

pub async fn update_dict_entries(
    db: &DatabaseConnection,
    a: sys_dict_entries::ActiveModel,
) -> StatusResult<sys_dict_entries::Model> {
    use sea_orm::ActiveModelTrait as _;
    a.update(db).await.map_err(db_status)
}

pub async fn delete_dict_entries(db: &DatabaseConnection, id: i64) -> StatusResult<()> {
    sys_dict_entries::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(db_status)?;
    Ok(())
}
