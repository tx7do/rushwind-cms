//! The file-transfer face — the multipart upload + streaming download
//! the reference registers BY HAND outside the generated handlers (the
//! bind layer parses protojson, not multipart — the reference's
//! `registerFileTransferServiceHandler` bypasses codegen for the same
//! reason, and the reference's generated registration for the face is
//! skipped in kind). rest.rs mounts the router behind the same auth
//! gate as the generated protected subtree — the reference's hand
//! registration sits inside the same server middleware chain.
//!
//! Transport: the object bytes ride the proto's streaming channels —
//! upload as one HttpBody chunk (the file name and mime beside the
//! operator in the gRPC metadata), download as a relayed HttpBody
//! stream (the content type off the stream, the recorded display name
//! off the response metadata for the disposition header). The BFF
//! touches no disk and carries no storage-root knowledge; the tenant
//! ownership gate sits core-side with the rows.
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

use std::sync::Arc;

use axum::extract::{FromRequest, Multipart, Request, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::services::{map_status, with_operator_claims};
use crate::state::{self, AppState};

use proto::proto::media::service::v1 as mediav1;
use proto::proto::storage::service::v1 as storagev1;

fn core_file(
    channel: &tonic::transport::Channel,
) -> storagev1::file_service_client::FileServiceClient<tonic::transport::Channel> {
    storagev1::file_service_client::FileServiceClient::new(channel.clone())
}

fn core_transfer(
    channel: &tonic::transport::Channel,
) -> storagev1::file_transfer_service_client::FileTransferServiceClient<tonic::transport::Channel> {
    storagev1::file_transfer_service_client::FileTransferServiceClient::new(channel.clone())
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

/// The streaming upload's parsed outcome: the contract JSON answer
/// (`UploadFileResponse` — the recorded link) plus the created row's
/// reference fields off the response metadata (the media library
/// flow's row linkage; the wire response carries only the link).
struct UploadedFileRef {
    response: Response,
    link: Option<String>,
    file_id: Option<u32>,
    size: Option<u64>,
    storage_path: Option<String>,
}

/// Ships one blob through the streaming upload channel: the bytes as a
/// single HttpBody chunk, the file name and mime (and the operator bag)
/// as the call's gRPC metadata.
async fn stream_upload(
    state: &AppState,
    claims: &Option<serde_json::Map<String, serde_json::Value>>,
    method: &axum::http::Method,
    file_name: String,
    mime: String,
    bytes: Vec<u8>,
) -> UploadedFileRef {
    let body = proto::proto::google::api::HttpBody {
        content_type: mime.clone(),
        data: bytes,
        ..Default::default()
    };
    let mut req = with_operator_claims(claims, futures_util::stream::iter(std::iter::once(body)));
    if let Ok(v) = tonic::metadata::MetadataValue::try_from(file_name.as_str()) {
        req.metadata_mut().insert("x-file-name", v);
    }
    if let Ok(v) = tonic::metadata::MetadataValue::try_from(mime.as_str()) {
        req.metadata_mut().insert("x-mime", v);
    }
    let mut core = core_transfer(&state.core_channel);
    let call = if *method == axum::http::Method::PUT {
        core.put_upload_file(req).await
    } else {
        core.post_upload_file(req).await
    };
    match call {
        Ok(resp) => {
            let file_id = resp
                .metadata()
                .get("x-file-id")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u32>().ok());
            let size = resp
                .metadata()
                .get("x-file-size")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            let storage_path = resp
                .metadata()
                .get("x-file-path")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);
            let out = resp.into_inner();
            let body = serde_json::json!({ "objectName": out.object_name.clone() });
            let response = (
                [(header::CONTENT_TYPE, "application/json")],
                body.to_string(),
            )
                .into_response();
            UploadedFileRef {
                response,
                link: out.object_name,
                file_id,
                size,
                storage_path,
            }
        }
        Err(e) => UploadedFileRef {
            response: rushwind_http_binding::envelope::error_response(map_status(e)),
            link: None,
            file_id: None,
            size: None,
            storage_path: None,
        },
    }
}

/// The multipart upload handler — parses the form, ships the bytes
/// through the streaming channel, answers the contract JSON.
pub async fn upload(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let claims = claims_of(&req);
    let method = req.method().clone();
    let (file_name, mime, bytes) = match read_upload(req, false).await {
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
    stream_upload(&state, &claims, &method, file_name, mime, bytes)
        .await
        .response
}

/// The media-library variant (the reference's `UploadMediaAsset`): the
/// file part alone through the streaming channel, then the media-asset
/// row (the linkage fields off the upload's response metadata, the
/// classifier and COMPLETED status mirroring the reference's tail).
pub async fn upload_asset(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let claims = claims_of(&req);
    let method = req.method().clone();
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
    let asset_type = asset_type_of(&mime);
    let out = stream_upload(
        &state,
        &claims,
        &method,
        file_name.clone(),
        mime.clone(),
        bytes,
    )
    .await;
    let mut asset_core = core_media_asset(&state.core_channel);
    let asset = mediav1::CreateMediaAssetRequest {
        data: Some(mediav1::MediaAsset {
            file_id: out.file_id,
            filename: Some(file_name),
            mime_type: Some(mime),
            size: out.size,
            url: out.link,
            storage_path: out.storage_path,
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
    if let Err(e) = asset_core
        .create(with_operator_claims(&claims, asset))
        .await
    {
        return rushwind_http_binding::envelope::error_response(map_status(e));
    }
    out.response
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
    let mut core = core_file(&state.core_channel);
    // Resolve the target row id: direct, or via the recorded link's
    // guid (the generated face carries no by-guid query; list + match).
    let id = if let Some(id) = q.file_id.or(q.file_id_alias) {
        Some(id)
    } else if let Some(guid) = q.file_guid.or(q.file_guid_alias) {
        let list = match core
            .list(with_operator_claims(
                &claims,
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
        list.into_iter()
            .find(|f| f.file_guid.as_deref() == Some(guid.as_str()))
            .and_then(|f| f.id)
    } else {
        return rushwind_http_binding::envelope::error_response(state::status_error(
            "BAD_REQUEST",
            "fileId or fileGuid required",
        ));
    };
    let Some(id) = id else {
        return rushwind_http_binding::envelope::error_response(state::not_found("file"));
    };

    let mut transfer = core_transfer(&state.core_channel);
    let resp = match transfer
        .download_file(with_operator_claims(
            &claims,
            storagev1::DownloadFileRequest {
                selector: Some(storagev1::download_file_request::Selector::FileId(id)),
                ..Default::default()
            },
        ))
        .await
    {
        Ok(r) => r,
        Err(e) => return rushwind_http_binding::envelope::error_response(map_status(e)),
    };
    // The display name off the response metadata (the disposition
    // header's filename); the content type off the stream's chunks.
    let display_name = resp
        .metadata()
        .get("x-file-name")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("file")
        .to_string();
    let mut stream = resp.into_inner();
    let mut bytes = Vec::new();
    let mut content_type = String::new();
    use futures_util::StreamExt as _;
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(b) => {
                if content_type.is_empty() {
                    content_type = b.content_type.clone();
                }
                bytes.extend_from_slice(&b.data);
            }
            Err(e) => return rushwind_http_binding::envelope::error_response(map_status(e)),
        }
    }
    if bytes.is_empty() {
        return rushwind_http_binding::envelope::error_response(state::not_found("object bytes"));
    }
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
    if let Ok(v) = format!("attachment; filename=\"{}\"", sanitize(&display_name)).parse() {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    resp
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
