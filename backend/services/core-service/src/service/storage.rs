//! The file storage face: the object store behind the streaming
//! transfer channel — the MinIO endpoint when the config selects one
//! (the reference's layout: the bucket policy, the key layout, the
//! public download link, the bucket materializing on demand), else the
//! local disk store — plus the files table metadata. `FileService`
//! proper stays the metadata CRUD; the object bytes ride the proto's
//! HttpBody streams only.

use std::sync::Arc;

use sea_orm::{EntityTrait, Set};
use tonic::{Request, Response, Status};

use crate::data::storage_repo as repo;
use crate::state::{bad, ts_to_proto, AppState};
use store::entities::files;
use store::paging::fetch_paged;

use proto::proto::storage::service::v1 as storagev1;

/// The local object root (mount a volume here in deployments).
const STORAGE_ROOT: &str = "/tmp/rushwind-cms-files";

fn file_proto(r: files::Model) -> storagev1::File {
    storagev1::File {
        id: Some(r.id as u32),
        // The row's provider label maps back through the proto enum's
        // name table (the reference's registered converter pair);
        // unmapped labels land nil.
        provider: r
            .provider
            .as_deref()
            .and_then(storagev1::OssProvider::from_str_name)
            .map(|v| v as i32),
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

fn mime_of(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "json" => "application/json",
        "txt" | "md" => "text/plain",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

fn metadata_str<T>(request: &tonic::Request<T>, name: &str) -> String {
    request
        .metadata()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string()
}

/// The media type's main/sub pair, lowercased and stripped of
/// parameters (the reference's mime.ParseMediaType preprocessing).
fn media_type_parts(mime: &str) -> Option<(String, String)> {
    let mt = mime
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let (main, sub) = mt.split_once('/')?;
    if main.is_empty() || sub.is_empty() {
        return None;
    }
    Some((main.to_string(), sub.to_string()))
}

/// The content-type storage-bucket policy — the reference's
/// ContentTypeToBucketName table verbatim (media-type prefix routing;
/// text and the office/document family land in docs, the rest in
/// files).
fn content_type_to_bucket(mime: &str) -> &'static str {
    let Some((main, sub)) = media_type_parts(mime) else {
        return "files";
    };
    match main.as_str() {
        "image" => "images",
        "video" => "videos",
        "audio" => "audios",
        "text" => "docs",
        "application" => match sub.as_str() {
            "pdf" | "json" => "docs",
            _ => {
                if sub.starts_with("vnd.ms-")
                    || sub.contains("officedocument")
                    || sub.contains("word")
                    || sub.contains("excel")
                    || sub.contains("powerpoint")
                {
                    "docs"
                } else {
                    "files"
                }
            }
        },
        _ => "files",
    }
}

/// The content-type extension table — the reference's
/// ContentTypeToFileExtension verbatim (dots stripped; the stdlib
/// ExtensionsByType fallback ports as no extension).
fn content_type_to_file_extension(mime: &str) -> &'static str {
    let mt = mime
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    match mt.as_str() {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "image/x-icon" | "image/vnd.microsoft.icon" => "ico",
        "image/svg+xml" => "svg",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        "video/x-matroska" | "video/mkv" => "mkv",
        "audio/mpeg" => "mp3",
        "audio/wav" | "audio/x-wav" => "wav",
        "audio/ogg" | "audio/vorbis" => "ogg",
        "audio/mp4" => "m4a",
        "text/plain" => "txt",
        "text/html" => "html",
        "text/css" => "css",
        "text/csv" => "csv",
        "text/xml" => "xml",
        "text/javascript" | "application/javascript" | "application/x-javascript" => "js",
        "text/x-lua" | "application/x-lua" => "lua",
        "text/x-python" | "application/x-python" | "text/python" => "py",
        "application/pdf" => "pdf",
        "application/json" => "json",
        "application/zip" => "zip",
        "application/x-tar" => "tar",
        "application/gzip" | "application/x-gzip" => "gz",
        "application/x-7z-compressed" | "application/7z" => "7z",
        _ => "",
    }
}

/// The trailing dot-separated segment of a file name (the reference's
/// ExtractFileExtension: a leading dot or no dot yields none).
fn extract_file_extension(file_name: &str) -> String {
    match file_name.rfind('.') {
        Some(idx) if idx > 0 => file_name[idx + 1..].to_ascii_lowercase(),
        _ => String::new(),
    }
}

/// The extension for a generated object name — the reference's
/// EnsureFileExtension: the source name's extension first, else the
/// content-type table, else "bin".
fn ensure_file_extension(file_name: &str, mime: &str) -> String {
    let ext = extract_file_extension(file_name);
    if !ext.is_empty() {
        return ext;
    }
    let ext = content_type_to_file_extension(mime);
    if !ext.is_empty() {
        return ext.to_string();
    }
    "bin".to_string()
}

/// The object URL join — the reference's JoinObjectUrl verbatim.
fn join_object_url(endpoint: &str, bucket_name: &str, object_name: &str) -> String {
    format!(
        "{}/{}/{}",
        endpoint.trim_end_matches('/'),
        bucket_name.trim_matches('/'),
        object_name.trim_start_matches('/')
    )
}

/// The object-store failure form.
fn minio_status(e: minio::s3::error::Error) -> Status {
    Status::internal(format!("object store: {e}"))
}

/// The object-store validation failure form.
fn minio_validation(e: minio::s3::error::ValidationErr) -> Status {
    Status::internal(format!("object store validation: {e:?}"))
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
    /// Writes an object to the configured store + the metadata row —
    /// the streaming upload face's landing point. The object-store
    /// profile picks the backend: the MinIO endpoint when configured
    /// (the reference's layout — the bucket policy, the key layout,
    /// the public download link, the bucket materializing on demand),
    /// else the local disk store.
    #[allow(clippy::too_many_arguments)]
    pub async fn put(
        &self,
        uploader: i64,
        tenant: i64,
        file_name: &str,
        mime_type: &str,
        bucket_param: &str,
        dir_param: &str,
        bytes: Vec<u8>,
    ) -> Result<files::Model, Status> {
        let ext = ensure_file_extension(file_name, mime_type);
        // One guid names both the stored object and the row's
        // save_file_name — the download resolves the pair.
        let guid = uuid::Uuid::now_v7().simple().to_string();
        let save_name = format!("{guid}.{ext}");

        // The provider branch. The MinIO side mirrors the reference's
        // mc.UploadFile: the bucket is the client-supplied one when
        // present, else the content-type policy bucket; the key
        // prefixes the client-supplied directory when present; the
        // recorded link is the download host join.
        let (provider, bucket, dir, link_url) = match &self.state.oss {
            Some(oss) => {
                let bucket = if bucket_param.is_empty() {
                    content_type_to_bucket(mime_type).to_string()
                } else {
                    bucket_param.to_string()
                };
                let key = if dir_param.is_empty() {
                    save_name.clone()
                } else {
                    format!("{dir_param}/{save_name}")
                };
                use minio::s3::types::S3Api as _;
                let exists: minio::s3::response::BucketExistsResponse = oss
                    .client
                    .bucket_exists(bucket.as_str())
                    .map_err(minio_validation)?
                    .build()
                    .send()
                    .await
                    .map_err(minio_status)?;
                if !exists.exists() {
                    let _: minio::s3::response::CreateBucketResponse = oss
                        .client
                        .create_bucket(bucket.as_str())
                        .map_err(minio_validation)?
                        .build()
                        .send()
                        .await
                        .map_err(minio_status)?;
                }
                let _: minio::s3::response::PutObjectContentResponse = oss
                    .client
                    .put_object_content(
                        bucket.as_str(),
                        key.as_str(),
                        minio::s3::builders::ObjectContent::from(bytes.clone()),
                    )
                    .map_err(minio_validation)?
                    .build()
                    .send()
                    .await
                    .map_err(minio_status)?;
                (
                    "MINIO",
                    Some(bucket.clone()),
                    dir_param.to_string(),
                    Some(join_object_url(&oss.download_host, &bucket, &key)),
                )
            }
            None => {
                let dir = std::path::Path::new(STORAGE_ROOT)
                    .join(chrono::Utc::now().format("%Y%m%d").to_string());
                tokio::fs::create_dir_all(&dir)
                    .await
                    .map_err(|e| Status::internal(format!("storage dir: {e}")))?;
                let path = dir.join(&save_name);
                tokio::fs::write(&path, &bytes)
                    .await
                    .map_err(|e| Status::internal(format!("storage write: {e}")))?;
                (
                    "LOCAL",
                    None,
                    dir.to_string_lossy().to_string(),
                    Some(format!("/admin/v1/file/download?fileGuid={guid}")),
                )
            }
        };

        let now = store::now();
        let row = repo::insert_files(
            &self.state.db,
            files::ActiveModel {
                provider: Set(Some(provider.to_string())),
                bucket_name: Set(bucket),
                file_directory: Set(Some(dir)),
                file_guid: Set(Some(guid.clone())),
                save_file_name: Set(Some(save_name)),
                file_name: Set(Some(file_name.to_string())),
                extension: Set(Some(ext)),
                size: Set(Some(bytes.len() as i64)),
                size_format: Set(Some(format_size(bytes.len() as i64))),
                content_hash: Set(Some(format!("{:x}", {
                    use sha2::Digest as _;
                    sha2::Sha256::digest(&bytes)
                }))),
                link_url: Set(link_url),
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

    /// Reads an object back (the download bridge) — the configured
    /// backend: the MinIO object off its recorded bucket/key, else the
    /// local disk store.
    pub async fn get_object(&self, file_id: i64) -> Result<(files::Model, Vec<u8>), Status> {
        let row = repo::files_by_id(&self.state.db, file_id).await?;
        if row.provider.as_deref() == Some("MINIO") {
            let Some(oss) = &self.state.oss else {
                return Err(Status::internal("object store not configured"));
            };
            let bucket = row.bucket_name.clone().unwrap_or_default();
            let dir = row.file_directory.clone().unwrap_or_default();
            let name = row.save_file_name.clone().unwrap_or_default();
            let key = if dir.is_empty() {
                name
            } else {
                format!("{dir}/{name}")
            };
            use minio::s3::types::S3Api as _;
            let resp: minio::s3::response::GetObjectResponse = oss
                .client
                .get_object(bucket.as_str(), key.as_str())
                .map_err(minio_validation)?
                .build()
                .send()
                .await
                .map_err(minio_status)?;
            let bytes = resp.into_bytes().await.map_err(minio_status)?.to_vec();
            return Ok((row, bytes));
        }
        let p = std::path::Path::new(&row.file_directory.clone().unwrap_or_default())
            .join(row.save_file_name.clone().unwrap_or_default());
        let bytes = tokio::fs::read(p)
            .await
            .map_err(|_| Status::not_found("object bytes missing"))?;
        Ok((row, bytes))
    }

    /// The upload streams' shared body: the bytes off the HttpBody
    /// chunks into the configured store; the file name, mime, and the
    /// client's storage-object placement (bucket/directory) ride the
    /// request metadata beside the operator headers.
    async fn upload_stream(
        &self,
        request: Request<tonic::Streaming<proto::proto::google::api::HttpBody>>,
    ) -> Result<Response<storagev1::UploadFileResponse>, Status> {
        let uploader = operator_of(&request)?;
        let tenant = tenant_of(&request);
        let file_name = metadata_str(&request, "x-file-name");
        let mime = metadata_str(&request, "x-mime");
        let bucket = metadata_str(&request, "x-file-bucket");
        let dir = metadata_str(&request, "x-file-dir");
        let mut bytes = Vec::new();
        let mut stream = request.into_inner();
        use tokio_stream::StreamExt as _;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(b) => bytes.extend_from_slice(&b.data),
                Err(e) => return Err(e),
            }
        }
        if bytes.is_empty() {
            return Err(bad("empty payload"));
        }
        let row = self
            .put(uploader, tenant, &file_name, &mime, &bucket, &dir, bytes)
            .await?;
        let mut resp = Response::new(storagev1::UploadFileResponse {
            object_name: row.link_url.clone(),
            ..Default::default()
        });
        // The created row's reference fields ride the response metadata
        // — the wire response carries only the link, and the media
        // library flow's row linkage (id/size/storage location) needs
        // the rest. The storage location takes the provider's recorded
        // form: the object-store bucket/key join, or the local path.
        // ASCII-only; dropped on encoding failure.
        if let Ok(v) = tonic::metadata::MetadataValue::try_from(row.id.to_string().as_str()) {
            resp.metadata_mut().insert("x-file-id", v);
        }
        if let Some(size) = row.size {
            if let Ok(v) = tonic::metadata::MetadataValue::try_from(size.to_string().as_str()) {
                resp.metadata_mut().insert("x-file-size", v);
            }
        }
        let storage_path = if row.provider.as_deref() == Some("MINIO") {
            join_object_url(
                "",
                &row.bucket_name.clone().unwrap_or_default(),
                &format!(
                    "{}/{}",
                    row.file_directory.clone().unwrap_or_default(),
                    row.save_file_name.clone().unwrap_or_default()
                ),
            )
        } else {
            format!(
                "{}/{}",
                row.file_directory.clone().unwrap_or_default(),
                row.save_file_name.clone().unwrap_or_default()
            )
        };
        if let Ok(v) = tonic::metadata::MetadataValue::try_from(storage_path.as_str()) {
            resp.metadata_mut().insert("x-file-path", v);
        }
        Ok(resp)
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
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        // The plain metadata insert (the reference's FileRepo.Create) —
        // the object bytes ride the streaming channel, never this face.
        let size = data.size;
        let row = repo::insert_files(
            &self.state.db,
            files::ActiveModel {
                provider: Set(data.provider.and_then(oss_provider_name)),
                bucket_name: Set(data.bucket_name),
                file_directory: Set(data.file_directory),
                file_guid: Set(data.file_guid),
                save_file_name: Set(data.save_file_name),
                file_name: Set(data.file_name),
                extension: Set(data.extension),
                size: Set(size.map(|v| v as i64)),
                size_format: Set(size.map(|v| format_size(v as i64))),
                link_url: Set(data.link_url),
                content_hash: Set(data.content_hash),
                tenant_id: Set(data.tenant_id.map(|v| v as i64)),
                created_by: Set(data.created_by.map(|v| v as i64)),
                created_at: Set(Some(store::now())),
                ..Default::default()
            },
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
            if row.provider.as_deref() == Some("MINIO") {
                // The MinIO object goes with the row (the reference's
                // FileService.Delete tail).
                if let Some(oss) = &self.state.oss {
                    let bucket = row.bucket_name.clone().unwrap_or_default();
                    let dir = row.file_directory.clone().unwrap_or_default();
                    let name = row.save_file_name.clone().unwrap_or_default();
                    let key = if dir.is_empty() {
                        name
                    } else {
                        format!("{dir}/{name}")
                    };
                    use minio::s3::types::S3Api as _;
                    let _: minio::s3::response::DeleteObjectResponse = oss
                        .client
                        .delete_object(bucket.as_str(), key.as_str())
                        .map_err(minio_validation)?
                        .build()
                        .send()
                        .await
                        .map_err(minio_status)?;
                }
            } else {
                let p = std::path::Path::new(&row.file_directory.clone().unwrap_or_default())
                    .join(row.save_file_name.clone().unwrap_or_default());
                let _ = tokio::fs::remove_file(p).await;
            }
        }
        repo::delete_files(&self.state.db, id).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── FileTransferService (the streaming transport face) ──────────────

#[async_trait::async_trait]
impl storagev1::file_transfer_service_server::FileTransferService for FileServiceImpl {
    /// The download stream: the object bytes + the extension-derived
    /// content type as one HttpBody; the recorded display name rides
    /// the response metadata for the BFF's disposition header. The
    /// tenant ownership gate (the reference's DownloadFile check) sits
    /// here — the row never leaves this service.
    async fn download_file(
        &self,
        request: Request<storagev1::DownloadFileRequest>,
    ) -> Result<Response<tonic::codegen::BoxStream<proto::proto::google::api::HttpBody>>, Status>
    {
        let tenant = tenant_of(&request);
        let req = request.into_inner();
        let Some(storagev1::download_file_request::Selector::FileId(id)) = req.selector else {
            return Err(bad("file_id required"));
        };
        let (row, bytes) = self.get_object(id as i64).await?;
        if row.tenant_id.unwrap_or(0) != tenant {
            return Err(Status::permission_denied(
                "forbidden: file does not belong to caller's tenant",
            ));
        }
        let body = proto::proto::google::api::HttpBody {
            content_type: mime_of(&row.extension.clone().unwrap_or_default()).to_string(),
            data: bytes,
            ..Default::default()
        };
        let stream: tonic::codegen::BoxStream<proto::proto::google::api::HttpBody> =
            Box::pin(tokio_stream::iter(std::iter::once(Ok(body))));
        let mut resp = Response::new(stream);
        if let Ok(v) = tonic::metadata::MetadataValue::try_from(
            row.file_name.clone().unwrap_or_default().as_str(),
        ) {
            resp.metadata_mut().insert("x-file-name", v);
        }
        Ok(resp)
    }

    async fn put_upload_file(
        &self,
        request: Request<tonic::Streaming<proto::proto::google::api::HttpBody>>,
    ) -> Result<Response<storagev1::UploadFileResponse>, Status> {
        self.upload_stream(request).await
    }

    async fn post_upload_file(
        &self,
        request: Request<tonic::Streaming<proto::proto::google::api::HttpBody>>,
    ) -> Result<Response<storagev1::UploadFileResponse>, Status> {
        self.upload_stream(request).await
    }
}
