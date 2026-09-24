//! The file storage face: local-disk object storage (the MinIO bridge
//! lands with the OSS phase) + the files table metadata, and the
//! sync_apis face (upserts the generated route table into sys_apis —
//! the caller ships the route list in the request).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::storage_repo as repo;
use crate::state::{bad, db_status, ts_to_proto, AppState};
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

fn format_size(size: i64) -> String {
    if size <= 0 {
        return "0B".to_string();
    }
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut s = size as f64;
    let mut i = 0usize;
    while s >= 1024.0 && i < UNITS.len() - 1 {
        s /= 1024.0;
        i += 1;
    }
    if i == 0 {
        return format!("{size}{}", UNITS[0]);
    }
    let v = (s * 100.0).round() / 100.0;
    let t = format!("{v:.2}");
    format!(
        "{}{}",
        t.trim_end_matches('0').trim_end_matches('.'),
        UNITS[i]
    )
}

fn oss_provider_name(v: i32) -> Option<String> {
    Some(
        match v {
            0 => "MINIO",
            1 => "ALIYUN",
            2 => "AWS",
            3 => "AZURE",
            4 => "BAIDU",
            5 => "QINIU",
            6 => "TENCENT",
            7 => "GOOGLE",
            8 => "HUAWEI",
            10 => "LOCAL",
            _ => return None,
        }
        .to_string(),
    )
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

fn tenant_of<T>(request: &tonic::Request<T>) -> i64 {
    request
        .metadata()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v >= 0)
        .unwrap_or(0)
}

impl FileServiceImpl {
    /// Writes an object to the local store + the metadata row (the
    /// upload transport bridge calls this; JSON/base64 for now).
    pub async fn put(
        &self,
        uploader: i64,
        tenant: i64,
        file_name: &str,
        _mime_type: &str,
        bytes: Vec<u8>,
    ) -> Result<files::Model, Status> {
        let (stem, ext) = file_name
            .rsplit_once('.')
            .map(|(s, e)| (s.to_string(), e.to_string()))
            .unwrap_or((file_name.to_string(), "bin".to_string()));
        let dir = std::path::Path::new(STORAGE_ROOT)
            .join(chrono::Utc::now().format("%Y%m%d").to_string());
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| Status::internal(format!("storage dir: {e}")))?;
        // One guid names both the on-disk object and the row's
        // save_file_name — the download resolves the pair.
        let guid = uuid::Uuid::now_v7().simple().to_string();
        let save_name = format!("{guid}.{ext}");
        let path = dir.join(&save_name);
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|e| Status::internal(format!("storage write: {e}")))?;

        let now = store::now();
        let row = repo::insert_files(
            &self.state.db,
            files::ActiveModel {
                provider: Set(Some("LOCAL".to_string())),
                file_directory: Set(Some(dir.to_string_lossy().to_string())),
                file_guid: Set(Some(guid.clone())),
                save_file_name: Set(Some(save_name)),
                file_name: Set(Some(stem)),
                extension: Set(Some(ext)),
                size: Set(Some(bytes.len() as i64)),
                size_format: Set(Some(format_size(bytes.len() as i64))),
                content_hash: Set(Some(format!("{:x}", {
                    use sha2::Digest as _;
                    sha2::Sha256::digest(&bytes)
                }))),
                link_url: Set(Some(format!("/admin/v1/file/download?fileGuid={guid}"))),
                tenant_id: Set(Some(tenant)),
                created_by: Set(Some(uploader)),
                created_at: Set(Some(now)),
                updated_at: Set(Some(now)),
                ..Default::default()
            },
        )
        .await?;
        Ok(row)
    }

    /// Reads an object back (the download bridge).
    pub async fn get_object(&self, file_id: i64) -> Result<(files::Model, Vec<u8>), Status> {
        let row = repo::files_by_id(&self.state.db, file_id).await?;
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
        let row = repo::files_by_id(&self.state.db, id).await?;
        Ok(Response::new(file_proto(row)))
    }

    async fn create(
        &self,
        request: Request<storagev1::CreateFileRequest>,
    ) -> Result<Response<storagev1::File>, Status> {
        let uploader = operator_of(&request)?;
        let tenant = tenant_of(&request);
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        // The upload transport bridge, interim shape: the BFF ships the
        // object bytes base64-coded in `bucket_name` (the File message
        // carries no byte field) through this unary metadata RPC; the
        // proto's streaming channels land with the transport phase and
        // replace the coding.
        let payload = data.bucket_name.clone().unwrap_or_default();
        let Ok(bytes) = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            payload.as_bytes(),
        ) else {
            return Err(bad("payload decode failed"));
        };
        if bytes.is_empty() {
            return Err(bad("empty payload"));
        }
        let row = self
            .put(
                uploader,
                tenant,
                &data.file_name.clone().unwrap_or_default(),
                "",
                bytes,
            )
            .await?;
        Ok(Response::new(file_proto(row)))
    }

    async fn update(
        &self,
        request: Request<storagev1::UpdateFileRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = repo::files_by_id(&self.state.db, req.id as i64).await?;
        let mut a: files::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.provider {
                a.provider = Set(oss_provider_name(v));
            }
            if let Some(v) = data.bucket_name {
                a.bucket_name = Set(Some(v));
            }
            if let Some(v) = data.file_directory {
                a.file_directory = Set(Some(v));
            }
            if let Some(v) = data.file_guid {
                a.file_guid = Set(Some(v));
            }
            if let Some(v) = data.save_file_name {
                a.save_file_name = Set(Some(v));
            }
            if let Some(v) = data.file_name {
                a.file_name = Set(Some(v));
            }
            if let Some(v) = data.extension {
                a.extension = Set(Some(v));
            }
            // The size re-derives the formatted label (the reference's
            // coupling).
            if let Some(v) = data.size {
                a.size = Set(Some(v as i64));
                a.size_format = Set(Some(format_size(v as i64)));
            } else if let Some(v) = data.size_format {
                a.size_format = Set(Some(v));
            }
            if let Some(v) = data.link_url {
                a.link_url = Set(Some(v));
            }
            if let Some(v) = data.content_hash {
                a.content_hash = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        repo::update_files(&self.state.db, a).await?;
        Ok(Response::new(pbjson_types::Empty {}))
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
        if let Ok(row) = repo::files_by_id(&self.state.db, id).await {
            let p = std::path::Path::new(&row.file_directory.clone().unwrap_or_default())
                .join(row.save_file_name.clone().unwrap_or_default());
            let _ = tokio::fs::remove_file(p).await;
        }
        repo::delete_files(&self.state.db, id).await?;
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
