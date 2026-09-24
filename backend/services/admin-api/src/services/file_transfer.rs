//! The file-transfer face — the multipart upload + streaming download
//! the reference registers BY HAND outside the generated handlers (the
//! bind layer parses protojson, not multipart — the reference's
//! `registerFileTransferServiceHandler` bypasses codegen for the same
//! reason, and the reference's generated registration for the face is
//! skipped in kind). rest.rs mounts the router behind the same auth
//! gate as the generated protected subtree — the reference's hand
//! registration sits inside the same server middleware chain.
//!
//! Wire format (the admin-react uploader):
//! * `POST|PUT /admin/v1/file/upload` — multipart fields: `file` (blob),
//!   `sourceFileName`, `mime`, `size`, `method`, `storageObject`
//!   ({bucketName,fileDirectory} — recorded only; the local store
//!   derives its own layout)
//! * `POST /admin/v1/file/asset/upload` — the media-library variant:
//!   the file part alone (its header carries the name and mime)
//! * `GET /admin/v1/file/download?fileId=|fileGuid=` — streams the
//!   object bytes back with the stored content type
//!
//! Interim transport: the object bytes bridge to the core store
//! base64-coded through the unary metadata create (the proto's
//! streaming channels land with the transport phase); the download
//! stopgap reads the single-node store directly off the row's recorded
//! location.

use std::sync::Arc;

use axum::extract::{FromRequest, Multipart, Request, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use base64::Engine as _;

use crate::services::{map_status, with_operator_claims};
use crate::state::{self, AppState};

use proto::proto::media::service::v1 as mediav1;
use proto::proto::storage::service::v1 as storagev1;

fn core_file(
    channel: &tonic::transport::Channel,
) -> storagev1::file_service_client::FileServiceClient<tonic::transport::Channel> {
    storagev1::file_service_client::FileServiceClient::new(channel.clone())
}

fn core_media_asset(
    channel: &tonic::transport::Channel,
) -> mediav1::media_asset_service_client::MediaAssetServiceClient<tonic::transport::Channel> {
    mediav1::media_asset_service_client::MediaAssetServiceClient::new(channel.clone())
}

/// The verified claim bag the auth gate injected into the request
/// extensions (the binding glue reads it the same way for the generated
/// faces).
fn claims_of(req: &Request) -> Option<serde_json::Map<String, serde_json::Value>> {
    req.extensions()
        .get::<auth::AuthClaims>()
        .map(|c| c.0.clone())
}

/// Pulls the `file` part out of the multipart body. The reference's
/// upload handlers let the `sourceFileName`/`mime` form fields override
/// the part header's own values (its asset variant reads the header
/// alone); mirrored here with the field values resolved after the scan
/// so the wire order cannot flip the precedence.
async fn read_upload(
    req: Request,
    asset_shape: bool,
) -> Result<(String, String, Option<Vec<u8>>), state::StatusError> {
    let mut multipart = match Multipart::from_request(req, &()).await {
        Ok(m) => m,
        Err(_) => {
            return Err(rushwind_http_binding::envelope::codec_error(
                "multipart body required",
            ))
        }
    };
    let mut part_name = String::new();
    let mut part_mime = String::new();
    let mut form_name = String::new();
    let mut form_mime = String::new();
    let mut bytes: Option<Vec<u8>> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "file" => {
                part_name = field.file_name().unwrap_or_default().to_string();
                part_mime = field.content_type().map(str::to_string).unwrap_or_default();
                match field.bytes().await {
                    Ok(b) => bytes = Some(b.to_vec()),
                    Err(_) => {
                        return Err(rushwind_http_binding::envelope::codec_error(
                            "read file field",
                        ))
                    }
                }
            }
            "sourceFileName" if !asset_shape => {
                if let Ok(v) = field.text().await {
                    form_name = v;
                }
            }
            "mime" if !asset_shape => {
                if let Ok(v) = field.text().await {
                    form_mime = v;
                }
            }
            // storageObject / size / method — consumed, nothing the
            // local store reads (the core records what it needs).
            _ => {
                let _ = field.bytes().await;
            }
        }
    }
    let file_name = if !asset_shape && !form_name.is_empty() {
        form_name
    } else {
        part_name
    };
    let mime = if !asset_shape && !form_mime.is_empty() {
        form_mime
    } else {
        part_mime
    };
    Ok((file_name, mime, bytes))
}

/// The multipart upload handler — parses the form, ships the bytes to
/// the core object store through the interim base64 bridge, answers
/// the contract's `UploadFileResponse` (the recorded download link).
pub async fn upload(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let claims = claims_of(&req);
    let (file_name, _mime, bytes) = match read_upload(req, false).await {
        Ok(v) => v,
        Err(e) => return rushwind_http_binding::envelope::error_response(e),
    };
    let Some(bytes) = bytes else {
        return rushwind_http_binding::envelope::error_response(state::status_error(
            "BAD_REQUEST",
            "file field required",
        ));
    };
    if bytes.is_empty() {
        return rushwind_http_binding::envelope::error_response(state::status_error(
            "BAD_REQUEST",
            "empty file",
        ));
    }
    let payload = storagev1::CreateFileRequest {
        data: Some(storagev1::File {
            file_name: Some(file_name),
            // The interim bridge carrier — the core's create decodes it
            // (the File message carries no byte field).
            bucket_name: Some(base64::engine::general_purpose::STANDARD.encode(&bytes)),
            ..Default::default()
        }),
    };
    let mut core = core_file(&state.core_channel);
    let file = match core.create(with_operator_claims(&claims, payload)).await {
        Ok(resp) => resp.into_inner(),
        Err(e) => return rushwind_http_binding::envelope::error_response(map_status(e)),
    };
    let body = serde_json::json!({ "objectName": file.link_url });
    (
        [(header::CONTENT_TYPE, "application/json")],
        body.to_string(),
    )
        .into_response()
}

/// The media-library variant (the reference's `UploadMediaAsset`): the
/// file part alone, then the file row through the bridge followed by
/// the media-asset row (the asset classifier and the COMPLETED status
/// mirror the reference's upload tail).
pub async fn upload_asset(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let claims = claims_of(&req);
    let (file_name, mime, bytes) = match read_upload(req, true).await {
        Ok(v) => v,
        Err(e) => return rushwind_http_binding::envelope::error_response(e),
    };
    let Some(bytes) = bytes else {
        return rushwind_http_binding::envelope::error_response(state::status_error(
            "BAD_REQUEST",
            "file field required",
        ));
    };
    if bytes.is_empty() {
        return rushwind_http_binding::envelope::error_response(state::status_error(
            "BAD_REQUEST",
            "empty file",
        ));
    }
    let file_payload = storagev1::CreateFileRequest {
        data: Some(storagev1::File {
            file_name: Some(file_name.clone()),
            bucket_name: Some(base64::engine::general_purpose::STANDARD.encode(&bytes)),
            ..Default::default()
        }),
    };
    let mut file_core = core_file(&state.core_channel);
    let file = match file_core
        .create(with_operator_claims(&claims, file_payload))
        .await
    {
        Ok(resp) => resp.into_inner(),
        Err(e) => return rushwind_http_binding::envelope::error_response(map_status(e)),
    };
    let storage_path = format!(
        "{}/{}",
        file.file_directory.clone().unwrap_or_default(),
        file.save_file_name.clone().unwrap_or_default()
    );
    let asset_type = asset_type_of(&mime);
    let asset = mediav1::CreateMediaAssetRequest {
        data: Some(mediav1::MediaAsset {
            file_id: file.id,
            filename: Some(file_name),
            mime_type: Some(mime),
            size: file.size,
            url: file.link_url.clone(),
            storage_path: Some(storage_path),
            r#type: Some(asset_type),
            processing_status: Some(mediav1::media_asset::ProcessingStatus::Completed as i32),
            created_by: claims
                .as_ref()
                .and_then(|c| c.get("uid"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            ..Default::default()
        }),
    };
    let mut asset_core = core_media_asset(&state.core_channel);
    if let Err(e) = asset_core
        .create(with_operator_claims(&claims, asset))
        .await
    {
        return rushwind_http_binding::envelope::error_response(map_status(e));
    }
    let body = serde_json::json!({ "objectName": file.link_url });
    (
        [(header::CONTENT_TYPE, "application/json")],
        body.to_string(),
    )
        .into_response()
}

/// Query shape of the download endpoint.
#[derive(Default, serde::Deserialize)]
pub struct DownloadQuery {
    pub file_id: Option<u32>,
    #[serde(alias = "fileId")]
    pub file_id_alias: Option<u32>,
    pub file_guid: Option<String>,
    #[serde(alias = "fileGuid")]
    pub file_guid_alias: Option<String>,
}

pub async fn download_query(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let claims = claims_of(&req);
    let q = req
        .uri()
        .query()
        .and_then(|s| serde_urlencoded::from_str::<DownloadQuery>(s).ok())
        .unwrap_or_default();
    let file_id = q.file_id.or(q.file_id_alias);
    let file_guid = q.file_guid.or(q.file_guid_alias);

    let mut core = core_file(&state.core_channel);
    // Look the file up (by id or guid — the recorded links carry the
    // guid), then stream the object.
    let lookup = if let Some(id) = file_id {
        core.get(tonic::Request::new(storagev1::GetFileRequest {
            query_by: Some(storagev1::get_file_request::QueryBy::Id(id)),
            ..Default::default()
        }))
        .await
    } else if let Some(guid) = file_guid {
        // The generated face carries no by-guid query; list + match.
        let list = match core
            .list(tonic::Request::new(
                proto::proto::pagination::PagingRequest {
                    no_paging: Some(true),
                    ..Default::default()
                },
            ))
            .await
        {
            Ok(resp) => resp.into_inner().items,
            Err(e) => return rushwind_http_binding::envelope::error_response(map_status(e)),
        };
        let Some(hit) = list
            .into_iter()
            .find(|f| f.file_guid.as_deref() == Some(guid.as_str()))
        else {
            return rushwind_http_binding::envelope::error_response(state::not_found("file"));
        };
        core.get(tonic::Request::new(storagev1::GetFileRequest {
            query_by: Some(storagev1::get_file_request::QueryBy::Id(
                hit.id.unwrap_or(0),
            )),
            ..Default::default()
        }))
        .await
    } else {
        return rushwind_http_binding::envelope::error_response(state::status_error(
            "BAD_REQUEST",
            "fileId or fileGuid required",
        ));
    };

    let file = match lookup {
        Ok(resp) => resp.into_inner(),
        Err(e) => return rushwind_http_binding::envelope::error_response(map_status(e)),
    };
    // The tenant ownership check (the reference's DownloadFile gate).
    let row_tenant = file.tenant_id.unwrap_or(0);
    let caller_tenant = claims
        .as_ref()
        .and_then(|c| c.get("tid"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    if row_tenant != caller_tenant {
        return rushwind_http_binding::envelope::error_response(state::status_error(
            "FORBIDDEN",
            "file does not belong to caller's tenant",
        ));
    }
    let name = file.file_name.clone().unwrap_or_default();
    let ext = file.extension.clone().unwrap_or_default();
    let content_type = mime_of(&ext);
    // The stopgap read: the single-node store recorded on the row (the
    // streaming download channel lands with the transport phase).
    let path = std::path::Path::new(&file.file_directory.clone().unwrap_or_default())
        .join(file.save_file_name.clone().unwrap_or_default());
    match tokio::fs::read(&path).await {
        Ok(bytes) => {
            let mut resp = Response::new(axum::body::Body::from(bytes));
            let h = resp.headers_mut();
            h.insert(
                header::CONTENT_TYPE,
                content_type
                    .parse()
                    .unwrap_or(axum::http::HeaderValue::from_static(
                        "application/octet-stream",
                    )),
            );
            if let Ok(v) = format!("attachment; filename=\"{}\"", sanitize(&name)).parse() {
                h.insert(header::CONTENT_DISPOSITION, v);
            }
            resp
        }
        Err(_) => rushwind_http_binding::envelope::error_response(state::not_found("object bytes")),
    }
}

/// The asset classifier — the reference's mimeTypeToAssetType table
/// verbatim (image/video/audio prefixes, the archive and office
/// document lists, else OTHER).
fn asset_type_of(mime: &str) -> i32 {
    use mediav1::media_asset::AssetType as T;
    if mime.is_empty() {
        return T::Unspecified as i32;
    }
    let mt = mime
        .to_ascii_lowercase()
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if mt.starts_with("image/") {
        return T::Image as i32;
    }
    if mt.starts_with("video/") {
        return T::Video as i32;
    }
    if mt.starts_with("audio/") {
        return T::Audio as i32;
    }
    if matches!(
        mt.as_str(),
        "application/zip"
            | "application/x-zip-compressed"
            | "multipart/x-zip"
            | "application/x-7z-compressed"
            | "application/x-tar"
            | "application/gzip"
            | "application/x-gzip"
            | "application/x-bzip2"
            | "application/x-bzip"
            | "application/x-xz"
            | "application/vnd.rar"
            | "application/x-rar-compressed"
            | "application/java-archive"
            | "application/zstd"
            | "application/x-zstd"
    ) {
        return T::Archive as i32;
    }
    if matches!(
        mt.as_str(),
        "application/pdf"
            | "application/msword"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.ms-excel"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-powerpoint"
            | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
    ) {
        return T::Document as i32;
    }
    T::Other as i32
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

fn sanitize(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect()
}

/// The mounted upload/download router (merged beside — and replacing
/// the generated registration of — the rest of the surface).
pub fn router(state: Arc<AppState>) -> axum::Router {
    axum::Router::new()
        .route(
            "/admin/v1/file/upload",
            axum::routing::post(upload).put(upload),
        )
        .route(
            "/admin/v1/file/asset/upload",
            axum::routing::post(upload_asset),
        )
        .route(
            "/admin/v1/file/download",
            axum::routing::get(download_query),
        )
        .with_state(state)
}
