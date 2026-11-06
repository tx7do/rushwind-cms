//! PreviewService — the admin face's preview-token issuer (post/page draft
//! preview links). The contract declares the service on the admin face
//! only; the domain service lands with its feature batch — every
//! operation answers NOT_IMPLEMENTED until then.

use std::sync::Arc;

use crate::state::{AppState, StatusError};

use proto::proto::admin::service::v1::{
    CreatePagePreviewTokenRequest, CreatePostPreviewTokenRequest, PreviewToken,
};

type Ctx = rushwind_http_binding::ctx::RequestContext;

fn not_implemented(message: &'static str) -> StatusError {
    StatusError::new(501, "NOT_IMPLEMENTED", message)
}

pub struct PreviewService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::PreviewServiceHandlers for PreviewService {
    async fn create_post_preview_token(
        &self,
        _ctx: Ctx,
        _req: CreatePostPreviewTokenRequest,
    ) -> Result<PreviewToken, StatusError> {
        Err(not_implemented(
            "preview tokens land with the draft-preview feature batch",
        ))
    }

    async fn create_page_preview_token(
        &self,
        _ctx: Ctx,
        _req: CreatePagePreviewTokenRequest,
    ) -> Result<PreviewToken, StatusError> {
        Err(not_implemented(
            "preview tokens land with the draft-preview feature batch",
        ))
    }
}
