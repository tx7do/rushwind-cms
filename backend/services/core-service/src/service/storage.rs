//! The file storage face: local-disk object storage (the MinIO bridge
//! lands with the OSS phase) + the files table metadata, and the
//! sync_apis face (upserts the generated route table into sys_apis —
//! the caller ships the route list in the request).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{files, sys_apis};
use store::paging::fetch_paged;

use proto::proto::permission::service::v1 as permissionv1;
use proto::proto::storage::service::v1 as storagev1;

/// The local object root (mount a volume here in deployments).
const STORAGE_ROOT: &str = "/tmp/rushwind-cms-files";

fn file_proto(r: files::Model) -> storagev1::File {
    storagev1::File {
        id: Some(r.id as u32),
        provider: Some(1), // LOCAL
        bucket_name: r.bucket_name.clone(),
        file_directory: r.file_directory.clone(),
        file_guid: r.file_guid.clone(),
        save_file_name: r.save_file_name.clone(),
        file_name: r.file_name.clone(),
        extension: r.extension.clone(),
        size: r.size.map(|v| v as u64),
        size_format: r.size_format.clone(),
        link_url: r.link_url.clone(),
        content_hash: r.content_hash.clone(),
        tenant_id: r.tenant_id.map(|v| v as u32),
        created_by: r.created_by.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct FileServiceImpl {
    pub state: Arc<AppState>,
}

fn operator_of<T>(request: &tonic::Request<T>) -> Result<i64, Status> {
    request
        .metadata()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .ok_or_else(|| Status::unauthenticated("user identity required"))
}

/// A stored object: the metadata row id + the local path + the bytes.
pub struct StoredObject {
    pub file_id: i64,
    pub path: std::path::PathBuf,
    pub bytes: Vec<u8>,
}

impl FileServiceImpl {
    /// Writes an object to the local store + the metadata row (the
    /// upload transport bridge calls this; JSON/base64 for now).
    pub async fn put(
        &self,
        uploader: i64,
        file_name: &str,
        _mime_type: &str,
        bytes: Vec<u8>,
    ) -> Result<StoredObject, Status> {
        let ext = file_name.rsplit('.').next().unwrap_or("bin");
        let key = format!(
            "{}/{}.{}",
            chrono::Utc::now().format("%Y%m%d"),
            uuid::Uuid::now_v7().simple(),
            ext
        );
        let dir = std::path::Path::new(STORAGE_ROOT)
            .join(chrono::Utc::now().format("%Y%m%d").to_string());
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| Status::internal(format!("storage dir: {e}")))?;
        let _ = key;
        let path = dir.join(format!(
            "{}.{}",
            uuid::Uuid::now_v7().simple(),
            file_name.rsplit('.').next().unwrap_or("bin")
        ));
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|e| Status::internal(format!("storage write: {e}")))?;

        let now = store::now();
        let guid = uuid::Uuid::now_v7().simple().to_string();
        let (stem, ext) = file_name
            .rsplit_once('.')
            .map(|(s, e)| (s.to_string(), e.to_string()))
            .unwrap_or((file_name.to_string(), "bin".to_string()));
        let save_name = format!("{}.{}", guid, ext);
        let row = files::ActiveModel {
            provider: Set(Some("LOCAL".to_string())),
            file_directory: Set(Some(
                path.parent()
                    .unwrap_or(std::path::Path::new(""))
                    .to_string_lossy()
                    .to_string(),
            )),
            file_guid: Set(Some(guid.clone())),
            save_file_name: Set(Some(save_name)),
            file_name: Set(Some(stem)),
            extension: Set(Some(ext)),
            size: Set(Some(bytes.len() as i64)),
            content_hash: Set(Some(format!("{:x}", {
                use sha2::Digest as _;
                sha2::Sha256::digest(&bytes)
            }))),
            link_url: Set(Some(format!("/admin/v1/file/download?fileGuid={guid}"))),
            tenant_id: Set(Some(0)),
            created_by: Set(Some(uploader)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;

        Ok(StoredObject {
            file_id: row.id,
            path,
            bytes,
        })
    }

    /// Reads an object back (the download bridge).
    pub async fn get_object(&self, file_id: i64) -> Result<(files::Model, Vec<u8>), Status> {
        let row = files::Entity::find_by_id(file_id)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("file"))?;
        let p = std::path::Path::new(&row.file_directory.clone().unwrap_or_default())
            .join(row.save_file_name.clone().unwrap_or_default());
        let bytes = tokio::fs::read(p)
            .await
            .map_err(|_| Status::not_found("object bytes missing"))?;
        Ok((row, bytes))
    }
}

#[async_trait::async_trait]
impl storagev1::file_service_server::FileService for FileServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<storagev1::ListFileResponse>, Status> {
        let (rows, total) =
            fetch_paged(&self.state.db, files::Entity::find(), &request.into_inner())
                .await
                .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(storagev1::ListFileResponse {
            items: rows.into_iter().map(file_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<storagev1::GetFileRequest>,
    ) -> Result<Response<storagev1::File>, Status> {
        let req = request.into_inner();
        let id = match req.query_by {
            Some(storagev1::get_file_request::QueryBy::Id(id)) => id as i64,
            _ => return Err(bad("query_by required")),
        };
        let row = files::Entity::find_by_id(id)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("file"))?;
        Ok(Response::new(file_proto(row)))
    }

    async fn delete(
        &self,
        request: Request<storagev1::DeleteFileRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let id = match req.query_by {
            Some(storagev1::delete_file_request::QueryBy::Id(id)) => id as i64,
            _ => return Err(bad("query_by required")),
        };
        if let Some(row) = files::Entity::find_by_id(id)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
        {
            let p = std::path::Path::new(&row.file_directory.clone().unwrap_or_default())
                .join(row.save_file_name.clone().unwrap_or_default());
            let _ = tokio::fs::remove_file(p).await;
        }
        files::Entity::delete_by_id(id)
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── sync_apis (the permission face) ──────────────────────────────────

pub struct ApiSyncServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::api_service_server::ApiService for ApiSyncServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::ListApiResponse>, Status> {
        crate::service::permission::ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .list(request)
        .await
    }

    async fn count(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::CountApiResponse>, Status> {
        crate::service::permission::ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .count(request)
        .await
    }

    async fn get(
        &self,
        request: Request<permissionv1::GetApiRequest>,
    ) -> Result<Response<permissionv1::Api>, Status> {
        crate::service::permission::ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .get(request)
        .await
    }

    async fn create(
        &self,
        request: Request<permissionv1::CreateApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        crate::service::permission::ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .create(request)
        .await
    }

    async fn update(
        &self,
        request: Request<permissionv1::UpdateApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        crate::service::permission::ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .update(request)
        .await
    }

    async fn delete(
        &self,
        request: Request<permissionv1::DeleteApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        crate::service::permission::ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .delete(request)
        .await
    }

    async fn sync_apis(
        &self,
        request: Request<permissionv1::SyncApisRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        // Upsert by operation: insert missing, refresh path/method of
        // existing (the BFF ships the route table it generated).
        let now = store::now();
        for api in req.apis {
            let existing = sys_apis::Entity::find()
                .filter(sys_apis::Column::Operation.eq(api.operation.clone().unwrap_or_default()))
                .one(&self.state.db)
                .await
                .map_err(db_status)?;
            match existing {
                Some(row) => {
                    let mut a: sys_apis::ActiveModel = row.into();
                    a.path = Set(api.path.clone());
                    a.method = Set(api.method.clone());
                    a.updated_at = Set(Some(now));
                    a.update(&self.state.db).await.map_err(db_status)?;
                }
                None => {
                    sys_apis::ActiveModel {
                        operation: Set(Some(api.operation.clone().unwrap_or_default())),
                        path: Set(Some(api.path.clone().unwrap_or_default())),
                        method: Set(Some(api.method.clone().unwrap_or_default())),
                        module: Set(api.module.clone()),
                        description: Set(api.description.clone()),
                        status: Set("ON".to_string()),
                        tenant_id: Set(Some(0)),
                        created_at: Set(Some(now)),
                        updated_at: Set(Some(now)),
                        ..Default::default()
                    }
                    .insert(&self.state.db)
                    .await
                    .map_err(db_status)?;
                }
            }
        }
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// operator helper retained for the upload bridge callers.
#[allow(dead_code)]
fn _op<T>(r: &tonic::Request<T>) -> Result<i64, Status> {
    operator_of(r)
}
