//! The translator passthrough (same-language stub until an external
//! provider lands).

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::state::AppState;

use proto::proto::translator::service::v1 as translatorv1;

// ── Translator ───────────────────────────────────────────────────────

pub struct TranslatorService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl translatorv1::translator_service_server::TranslatorService for TranslatorService {
    async fn translate(
        &self,
        request: Request<translatorv1::TranslateRequest>,
    ) -> Result<Response<translatorv1::TranslateResponse>, Status> {
        // The provider bridge (google/baidu/alibaba/volc, the reference's
        // pkg/translator) lands with the outbound-API phase; the identity
        // passthrough keeps the contract live for same-language calls.
        let _ = &self.state;
        let req = request.into_inner();
        let content = req.content.unwrap_or_default();
        Ok(Response::new(translatorv1::TranslateResponse {
            translated_content: Some(content.clone()),
            raw_content: Some(content),
        }))
    }
}
