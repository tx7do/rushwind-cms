//! The file-transfer face — the multipart upload + streaming download
//! the reference registers BY HAND outside the generated handlers (the
//! bind layer parses protojson, not multipart — the reference's
//! `registerFileTransferServiceHandler` bypasses codegen for the same
//! reason, and the generated registration for the face is skipped in
//! kind). rest.rs mounts the router behind the same auth gate as the
//! generated protected subtree — the reference's hand registration sits
//! inside the same server middleware chain.
//!
//! The C-side shape mirrors the admin face minus the media-library
//! variant: upload POST|PUT plus download GET — the direct upload
//! (metadata recorded core-side, the object through the streaming
//! channel) and the fileId download with the tenant ownership gate.
//!
//! Wire format (the front-end uploader):
//! * `POST|PUT /app/v1/file/upload` — multipart fields: `file` (blob),
//!   `sourceFileName`, `mime`, `size`, `method`, `storageObject`
//!   ({bucketName,fileDirectory} — the placement pair)
//! * `GET /app/v1/file/download?fileId=|fileGuid=` — streams the
//!   object bytes back with the stored content type

use std::sync::Arc;

use axum::extract::{FromRequest, Multipart, Request, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::services::{map_status, with_operator_claims};
use crate::state::{self, AppState};

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

/// The verified claim bag the auth gate injected into the request
/// extensions (the binding glue reads it the same way for the generated
/// faces).
fn claims_of(req: &Request) -> Option<serde_json::Map<String, serde_json::Value>> {
    req.extensions()
        .get::<auth::AuthClaims>()
        .map(|c| c.0.clone())
}

/// The `storageObject` form field's shape — the placement pair the
/// reference parses into its StorageObject message and forwards to the
/// upload (ours rides the streaming call's metadata).
#[derive(Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct StorageObjectForm {
    #[serde(default)]
    bucket_name: Option<String>,
    #[serde(default)]
    file_directory: Option<String>,
}

/// Pulls the `file` part out of the multipart body. The reference's
/// upload handlers let the `sourceFileName`/`mime` form fields override
/// the part header's own values and parse the `storageObject` placement
/// pair out of the form; mirrored here with the field values resolved
/// after the scan so the wire order cannot flip the precedence.
async fn read_upload(
    req: Request,
) -> Result<(String, String, String, String, Option<Vec<u8>>), state::StatusError> {
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
    let mut form_bucket = String::new();
    let mut form_dir = String::new();
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
            "sourceFileName" => {
                if let Ok(v) = field.text().await {
                    form_name = v;
                }
            }
            "mime" => {
                if let Ok(v) = field.text().await {
                    form_mime = v;
                }
            }
            "storageObject" => {
                if let Ok(v) = field.text().await {
                    // The reference unmarshals the placement JSON into
                    // its StorageObject message — a malformed body fails
                    // the call there; mirrored.
                    match serde_json::from_str::<StorageObjectForm>(&v) {
                        Ok(f) => {
                            form_bucket = f.bucket_name.unwrap_or_default();
                            form_dir = f.file_directory.unwrap_or_default();
                        }
                        Err(_) => {
                            return Err(rushwind_http_binding::envelope::codec_error(
                                "malformed storageObject",
                            ))
                        }
                    }
                }
            }
            // size / method — consumed, nothing the transfer reads.
            _ => {
                let _ = field.bytes().await;
            }
        }
    }
    let file_name = if !form_name.is_empty() {
        form_name
    } else {
        part_name
    };
    let mime = if !form_mime.is_empty() {
        form_mime
    } else {
        part_mime
    };
    Ok((file_name, mime, form_bucket, form_dir, bytes))
}

/// The streaming upload's parsed outcome: the contract JSON answer
/// (`UploadFileResponse` — the recorded link) plus the created row's
/// reference fields off the response metadata.
struct UploadedFileRef {
    response: Response,
}

/// Ships one blob through the streaming upload channel: the bytes as a
/// single HttpBody chunk, the file name, mime, and the client's
/// storage-object placement (bucket/directory) — and the operator bag —
/// as the call's gRPC metadata.
#[allow(clippy::too_many_arguments)]
async fn stream_upload(
    state: &AppState,
    claims: &Option<serde_json::Map<String, serde_json::Value>>,
    method: &axum::http::Method,
    file_name: String,
    mime: String,
    bucket: String,
    dir: String,
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
    if let Ok(v) = tonic::metadata::MetadataValue::try_from(bucket.as_str()) {
        req.metadata_mut().insert("x-file-bucket", v);
    }
    if let Ok(v) = tonic::metadata::MetadataValue::try_from(dir.as_str()) {
        req.metadata_mut().insert("x-file-dir", v);
    }
    let mut core = core_transfer(&state.core_channel);
    let call = if *method == axum::http::Method::PUT {
        core.put_upload_file(req).await
    } else {
        core.post_upload_file(req).await
    };
    match call {
        Ok(resp) => {
            let out = resp.into_inner();
            let body = serde_json::json!({ "objectName": out.object_name.clone() });
            let response = (
                [(header::CONTENT_TYPE, "application/json")],
                body.to_string(),
            )
                .into_response();
            UploadedFileRef { response }
        }
        Err(e) => UploadedFileRef {
            response: rushwind_http_binding::envelope::error_response(map_status(e)),
        },
    }
}

/// The multipart upload handler — parses the form, ships the bytes
/// through the streaming channel, answers the contract JSON.
pub async fn upload(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let claims = claims_of(&req);
    let method = req.method().clone();
    let (file_name, mime, bucket, dir, bytes) = match read_upload(req).await {
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
    stream_upload(
        &state, &claims, &method, file_name, mime, bucket, dir, bytes,
    )
    .await
    .response
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
    // guid (list + match).
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
            "/app/v1/file/upload",
            axum::routing::post(upload).put(upload),
        )
        .route("/app/v1/file/download", axum::routing::get(download_query))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_keeps_only_safe_disposition_chars() {
        assert_eq!(sanitize("report 2026.pdf"), "report2026.pdf");
        assert_eq!(sanitize("üñîçödé.png"), "d.png");
        assert_eq!(sanitize(""), "");
    }

    #[test]
    fn storage_object_parses_the_placement_pair() {
        let ok: StorageObjectForm =
            serde_json::from_str(r#"{"bucketName":"images","fileDirectory":"e2e"}"#).unwrap();
        assert_eq!(ok.bucket_name.as_deref(), Some("images"));
        assert_eq!(ok.file_directory.as_deref(), Some("e2e"));
        let partial: StorageObjectForm = serde_json::from_str("{}").unwrap();
        assert_eq!(partial.bucket_name, None);
        assert!(serde_json::from_str::<StorageObjectForm>("{nope").is_err());
    }

    #[test]
    fn download_query_reads_camel_and_snake_forms() {
        let q: DownloadQuery = serde_urlencoded::from_str("fileId=7").unwrap();
        assert_eq!(q.file_id.or(q.file_id_alias), Some(7));
        let q: DownloadQuery = serde_urlencoded::from_str("fileGuid=abc").unwrap();
        assert_eq!(q.file_guid.or(q.file_guid_alias).as_deref(), Some("abc"));
        let q: DownloadQuery = serde_urlencoded::from_str("").unwrap();
        assert_eq!(q.file_id.or(q.file_id_alias), None);
    }
}
