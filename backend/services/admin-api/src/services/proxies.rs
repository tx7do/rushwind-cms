//! Generated thin-BFF proxies — one impl per BFF trait method, each a
//! pass-through to the core domain service's gRPC face (the BFF and
//! domain methods share their message types by contract, so no mapping
//! rides here). DO NOT EDIT; regenerate via scripts/gen-proxies.py when
//! the contract re-syncs. Hand-written faces (authentication — captcha/
//! cookie concerns; admin-portal aggregation; file-transfer multipart)
//! live in their own modules.
#![allow(clippy::all)]
#![allow(missing_docs)]

use std::sync::Arc;

use crate::services::map_status;
use crate::services::with_operator;
use crate::state::{AppState, StatusError};

/// The pass-through proxy of `ApiAuditLogServiceHandlers`.
pub struct ApiAuditLogProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::ApiAuditLogServiceHandlers for ApiAuditLogProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::audit::service::v1::ListApiAuditLogResponse, StatusError> {
        let mut core = proto::proto::audit::service::v1::api_audit_log_service_client::ApiAuditLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::audit::service::v1::GetApiAuditLogRequest,
    ) -> Result<proto::proto::audit::service::v1::ApiAuditLog, StatusError> {
        let mut core = proto::proto::audit::service::v1::api_audit_log_service_client::ApiAuditLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `ApiServiceHandlers`.
pub struct ApiProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::ApiServiceHandlers for ApiProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::permission::service::v1::ListApiResponse, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::api_service_client::ApiServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::GetApiRequest,
    ) -> Result<proto::proto::permission::service::v1::Api, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::api_service_client::ApiServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::CreateApiRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::api_service_client::ApiServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::UpdateApiRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::api_service_client::ApiServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::DeleteApiRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::api_service_client::ApiServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn sync_apis(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: pbjson_types::Empty,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let _ = req;
        // Sync the generated route table into sys_apis (the route
        // corpus this very binary was generated from).
        let mut req = proto::proto::permission::service::v1::SyncApisRequest::default();
        for r in proto::gen_admin::routes::ROUTES {
            if r.shadowed {
                continue;
            }
            req.apis.push(proto::proto::permission::service::v1::Api {
                operation: Some(r.operation_id.to_string()),
                path: Some(r.path.to_string()),
                method: Some(r.method.to_string()),
                module: Some(
                    r.service_fq
                        .split('.')
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                ),
                ..Default::default()
            });
        }
        let mut core =
            proto::proto::permission::service::v1::api_service_client::ApiServiceClient::new(
                self.state.core_channel.clone(),
            );
        core.sync_apis(tonic::Request::new(req))
            .await
            .map_err(map_status)?;
        Ok(pbjson_types::Empty {})
    }

    async fn get_walk_route_data(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: pbjson_types::Empty,
    ) -> Result<proto::proto::permission::service::v1::ListApiResponse, StatusError> {
        let _ = (req, &self.state);
        Err(crate::state::internal_error("not implemented"))
    }
}

/// The pass-through proxy of `CategoryServiceHandlers`.
pub struct CategoryProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::CategoryServiceHandlers for CategoryProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::content::service::v1::ListCategoryResponse, StatusError> {
        let mut core =
            proto::proto::content::service::v1::category_service_client::CategoryServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::GetCategoryRequest,
    ) -> Result<proto::proto::content::service::v1::Category, StatusError> {
        let mut core =
            proto::proto::content::service::v1::category_service_client::CategoryServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::CreateCategoryRequest,
    ) -> Result<proto::proto::content::service::v1::Category, StatusError> {
        let mut core =
            proto::proto::content::service::v1::category_service_client::CategoryServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::UpdateCategoryRequest,
    ) -> Result<proto::proto::content::service::v1::Category, StatusError> {
        let mut core =
            proto::proto::content::service::v1::category_service_client::CategoryServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::DeleteCategoryRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::content::service::v1::category_service_client::CategoryServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `CommentServiceHandlers`.
pub struct CommentProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::CommentServiceHandlers for CommentProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::comment::service::v1::ListCommentResponse, StatusError> {
        let mut core =
            proto::proto::comment::service::v1::comment_service_client::CommentServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::comment::service::v1::GetCommentRequest,
    ) -> Result<proto::proto::comment::service::v1::Comment, StatusError> {
        let mut core =
            proto::proto::comment::service::v1::comment_service_client::CommentServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::comment::service::v1::CreateCommentRequest,
    ) -> Result<proto::proto::comment::service::v1::Comment, StatusError> {
        let mut core =
            proto::proto::comment::service::v1::comment_service_client::CommentServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::comment::service::v1::UpdateCommentRequest,
    ) -> Result<proto::proto::comment::service::v1::Comment, StatusError> {
        let mut core =
            proto::proto::comment::service::v1::comment_service_client::CommentServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::comment::service::v1::DeleteCommentRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::comment::service::v1::comment_service_client::CommentServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `ContentModelServiceHandlers`.
pub struct ContentModelProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::ContentModelServiceHandlers for ContentModelProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::content::service::v1::ListContentModelResponse, StatusError> {
        let mut core = proto::proto::content::service::v1::content_model_service_client::ContentModelServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::GetContentModelRequest,
    ) -> Result<proto::proto::content::service::v1::ContentModel, StatusError> {
        let mut core = proto::proto::content::service::v1::content_model_service_client::ContentModelServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::CreateContentModelRequest,
    ) -> Result<proto::proto::content::service::v1::ContentModel, StatusError> {
        let mut core = proto::proto::content::service::v1::content_model_service_client::ContentModelServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::UpdateContentModelRequest,
    ) -> Result<proto::proto::content::service::v1::ContentModel, StatusError> {
        let mut core = proto::proto::content::service::v1::content_model_service_client::ContentModelServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::DeleteContentModelRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::content::service::v1::content_model_service_client::ContentModelServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn list_field_definitions(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::ListFieldDefinitionsRequest,
    ) -> Result<proto::proto::content::service::v1::ListFieldDefinitionsResponse, StatusError> {
        let mut core = proto::proto::content::service::v1::content_model_service_client::ContentModelServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list_field_definitions(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `DataAccessAuditLogServiceHandlers`.
pub struct DataAccessAuditLogProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::DataAccessAuditLogServiceHandlers for DataAccessAuditLogProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::audit::service::v1::ListDataAccessAuditLogResponse, StatusError> {
        let mut core = proto::proto::audit::service::v1::data_access_audit_log_service_client::DataAccessAuditLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::audit::service::v1::GetDataAccessAuditLogRequest,
    ) -> Result<proto::proto::audit::service::v1::DataAccessAuditLog, StatusError> {
        let mut core = proto::proto::audit::service::v1::data_access_audit_log_service_client::DataAccessAuditLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `DictEntryServiceHandlers`.
pub struct DictEntryProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::DictEntryServiceHandlers for DictEntryProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::dict::service::v1::ListDictEntryResponse, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::dict_entry_service_client::DictEntryServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::CreateDictEntryRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::dict_entry_service_client::DictEntryServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::UpdateDictEntryRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::dict_entry_service_client::DictEntryServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::DeleteDictEntryRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::dict_entry_service_client::DictEntryServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn list_by_type_code(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::ListDictEntryByTypeCodeRequest,
    ) -> Result<proto::proto::dict::service::v1::ListDictEntryByTypeCodeResponse, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::dict_entry_service_client::DictEntryServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list_by_type_code(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `DictTypeServiceHandlers`.
pub struct DictTypeProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::DictTypeServiceHandlers for DictTypeProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::dict::service::v1::ListDictTypeResponse, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::dict_type_service_client::DictTypeServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::GetDictTypeRequest,
    ) -> Result<proto::proto::dict::service::v1::DictType, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::dict_type_service_client::DictTypeServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::CreateDictTypeRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::dict_type_service_client::DictTypeServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::UpdateDictTypeRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::dict_type_service_client::DictTypeServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::DeleteDictTypeRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::dict_type_service_client::DictTypeServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `FileServiceHandlers`.
pub struct FileProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::FileServiceHandlers for FileProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::storage::service::v1::ListFileResponse, StatusError> {
        let mut core =
            proto::proto::storage::service::v1::file_service_client::FileServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::storage::service::v1::GetFileRequest,
    ) -> Result<proto::proto::storage::service::v1::File, StatusError> {
        let mut core =
            proto::proto::storage::service::v1::file_service_client::FileServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::storage::service::v1::CreateFileRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let _ = (req, &self.state);
        Err(crate::state::internal_error("not implemented"))
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::storage::service::v1::UpdateFileRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::storage::service::v1::file_service_client::FileServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::storage::service::v1::DeleteFileRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::storage::service::v1::file_service_client::FileServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `InteractionAdminServiceHandlers`.
pub struct InteractionAdminProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::InteractionAdminServiceHandlers for InteractionAdminProxy {
    async fn purge_target_interactions(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::interaction::service::v1::PurgeTargetInteractionsRequest,
    ) -> Result<proto::proto::interaction::service::v1::PurgeTargetInteractionsResponse, StatusError>
    {
        let mut core = proto::proto::interaction::service::v1::interaction_admin_service_client::InteractionAdminServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .purge_target_interactions(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn purge_user_interactions(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::interaction::service::v1::PurgeUserInteractionsRequest,
    ) -> Result<proto::proto::interaction::service::v1::PurgeUserInteractionsResponse, StatusError>
    {
        let mut core = proto::proto::interaction::service::v1::interaction_admin_service_client::InteractionAdminServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .purge_user_interactions(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn reset_counter(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::interaction::service::v1::ResetCounterRequest,
    ) -> Result<proto::proto::interaction::service::v1::ResetCounterResponse, StatusError> {
        let mut core = proto::proto::interaction::service::v1::interaction_admin_service_client::InteractionAdminServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .reset_counter(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `InternalMessageCategoryServiceHandlers`.
pub struct InternalMessageCategoryProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::InternalMessageCategoryServiceHandlers
    for InternalMessageCategoryProxy
{
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<
        proto::proto::internal_message::service::v1::ListInternalMessageCategoryResponse,
        StatusError,
    > {
        let mut core = proto::proto::internal_message::service::v1::internal_message_category_service_client::InternalMessageCategoryServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::internal_message::service::v1::GetInternalMessageCategoryRequest,
    ) -> Result<proto::proto::internal_message::service::v1::InternalMessageCategory, StatusError>
    {
        let mut core = proto::proto::internal_message::service::v1::internal_message_category_service_client::InternalMessageCategoryServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::internal_message::service::v1::CreateInternalMessageCategoryRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::internal_message::service::v1::internal_message_category_service_client::InternalMessageCategoryServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::internal_message::service::v1::UpdateInternalMessageCategoryRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::internal_message::service::v1::internal_message_category_service_client::InternalMessageCategoryServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::internal_message::service::v1::DeleteInternalMessageCategoryRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::internal_message::service::v1::internal_message_category_service_client::InternalMessageCategoryServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `InternalMessageRecipientServiceHandlers`.
pub struct InternalMessageRecipientProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::InternalMessageRecipientServiceHandlers
    for InternalMessageRecipientProxy
{
    async fn list_user_inbox(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::internal_message::service::v1::ListUserInboxResponse, StatusError>
    {
        let mut core = proto::proto::internal_message::service::v1::internal_message_recipient_service_client::InternalMessageRecipientServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list_user_inbox(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete_notification_from_inbox(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::internal_message::service::v1::DeleteNotificationFromInboxRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::internal_message::service::v1::internal_message_recipient_service_client::InternalMessageRecipientServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete_notification_from_inbox(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn mark_notification_as_read(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::internal_message::service::v1::MarkNotificationAsReadRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::internal_message::service::v1::internal_message_recipient_service_client::InternalMessageRecipientServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .mark_notification_as_read(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `InternalMessageServiceHandlers`.
pub struct InternalMessageProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::InternalMessageServiceHandlers for InternalMessageProxy {
    async fn list_message(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::internal_message::service::v1::ListInternalMessageResponse, StatusError>
    {
        let mut core = proto::proto::internal_message::service::v1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list_message(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get_message(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::internal_message::service::v1::GetInternalMessageRequest,
    ) -> Result<proto::proto::internal_message::service::v1::InternalMessage, StatusError> {
        let mut core = proto::proto::internal_message::service::v1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get_message(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update_message(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::internal_message::service::v1::UpdateInternalMessageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::internal_message::service::v1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update_message(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete_message(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::internal_message::service::v1::DeleteInternalMessageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::internal_message::service::v1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete_message(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn send_message(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::internal_message::service::v1::SendMessageRequest,
    ) -> Result<proto::proto::internal_message::service::v1::SendMessageResponse, StatusError> {
        let mut core = proto::proto::internal_message::service::v1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .send_message(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn revoke_message(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::internal_message::service::v1::RevokeMessageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::internal_message::service::v1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .revoke_message(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `LanguageServiceHandlers`.
pub struct LanguageProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::LanguageServiceHandlers for LanguageProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::dict::service::v1::ListLanguageResponse, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::language_service_client::LanguageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::GetLanguageRequest,
    ) -> Result<proto::proto::dict::service::v1::Language, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::language_service_client::LanguageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::CreateLanguageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::language_service_client::LanguageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::UpdateLanguageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::language_service_client::LanguageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::DeleteLanguageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::language_service_client::LanguageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn batch_create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::dict::service::v1::BatchCreateLanguagesRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::dict::service::v1::language_service_client::LanguageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .batch_create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `LoginAuditLogServiceHandlers`.
pub struct LoginAuditLogProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::LoginAuditLogServiceHandlers for LoginAuditLogProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::audit::service::v1::ListLoginAuditLogResponse, StatusError> {
        let mut core = proto::proto::audit::service::v1::login_audit_log_service_client::LoginAuditLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::audit::service::v1::GetLoginAuditLogRequest,
    ) -> Result<proto::proto::audit::service::v1::LoginAuditLog, StatusError> {
        let mut core = proto::proto::audit::service::v1::login_audit_log_service_client::LoginAuditLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `LoginPolicyServiceHandlers`.
pub struct LoginPolicyProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::LoginPolicyServiceHandlers for LoginPolicyProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::authentication::service::v1::ListLoginPolicyResponse, StatusError>
    {
        let mut core = proto::proto::authentication::service::v1::login_policy_service_client::LoginPolicyServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::authentication::service::v1::GetLoginPolicyRequest,
    ) -> Result<proto::proto::authentication::service::v1::LoginPolicy, StatusError> {
        let mut core = proto::proto::authentication::service::v1::login_policy_service_client::LoginPolicyServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::authentication::service::v1::CreateLoginPolicyRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::authentication::service::v1::login_policy_service_client::LoginPolicyServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::authentication::service::v1::UpdateLoginPolicyRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::authentication::service::v1::login_policy_service_client::LoginPolicyServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::authentication::service::v1::DeleteLoginPolicyRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::authentication::service::v1::login_policy_service_client::LoginPolicyServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `MediaAssetServiceHandlers`.
pub struct MediaAssetProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::MediaAssetServiceHandlers for MediaAssetProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::media::service::v1::ListMediaAssetResponse, StatusError> {
        let mut core = proto::proto::media::service::v1::media_asset_service_client::MediaAssetServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::media::service::v1::GetMediaAssetRequest,
    ) -> Result<proto::proto::media::service::v1::MediaAsset, StatusError> {
        let mut core = proto::proto::media::service::v1::media_asset_service_client::MediaAssetServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::media::service::v1::CreateMediaAssetRequest,
    ) -> Result<proto::proto::media::service::v1::MediaAsset, StatusError> {
        let mut core = proto::proto::media::service::v1::media_asset_service_client::MediaAssetServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::media::service::v1::UpdateMediaAssetRequest,
    ) -> Result<proto::proto::media::service::v1::MediaAsset, StatusError> {
        let mut core = proto::proto::media::service::v1::media_asset_service_client::MediaAssetServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::media::service::v1::DeleteMediaAssetRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::media::service::v1::media_asset_service_client::MediaAssetServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `MenuServiceHandlers`.
pub struct MenuProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::MenuServiceHandlers for MenuProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::permission::service::v1::ListMenuResponse, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::menu_service_client::MenuServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::GetMenuRequest,
    ) -> Result<proto::proto::permission::service::v1::Menu, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::menu_service_client::MenuServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::CreateMenuRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::menu_service_client::MenuServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::UpdateMenuRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::menu_service_client::MenuServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::DeleteMenuRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::menu_service_client::MenuServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `NavigationItemServiceHandlers`.
pub struct NavigationItemProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::NavigationItemServiceHandlers for NavigationItemProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::site::service::v1::ListNavigationItemResponse, StatusError> {
        let mut core = proto::proto::site::service::v1::navigation_item_service_client::NavigationItemServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::GetNavigationItemRequest,
    ) -> Result<proto::proto::site::service::v1::NavigationItem, StatusError> {
        let mut core = proto::proto::site::service::v1::navigation_item_service_client::NavigationItemServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::CreateNavigationItemRequest,
    ) -> Result<proto::proto::site::service::v1::NavigationItem, StatusError> {
        let mut core = proto::proto::site::service::v1::navigation_item_service_client::NavigationItemServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::UpdateNavigationItemRequest,
    ) -> Result<proto::proto::site::service::v1::NavigationItem, StatusError> {
        let mut core = proto::proto::site::service::v1::navigation_item_service_client::NavigationItemServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::DeleteNavigationItemRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::site::service::v1::navigation_item_service_client::NavigationItemServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `NavigationServiceHandlers`.
pub struct NavigationProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::NavigationServiceHandlers for NavigationProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::site::service::v1::ListNavigationResponse, StatusError> {
        let mut core = proto::proto::site::service::v1::navigation_service_client::NavigationServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::GetNavigationRequest,
    ) -> Result<proto::proto::site::service::v1::Navigation, StatusError> {
        let mut core = proto::proto::site::service::v1::navigation_service_client::NavigationServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::CreateNavigationRequest,
    ) -> Result<proto::proto::site::service::v1::Navigation, StatusError> {
        let mut core = proto::proto::site::service::v1::navigation_service_client::NavigationServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::UpdateNavigationRequest,
    ) -> Result<proto::proto::site::service::v1::Navigation, StatusError> {
        let mut core = proto::proto::site::service::v1::navigation_service_client::NavigationServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::DeleteNavigationRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::site::service::v1::navigation_service_client::NavigationServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `OperationAuditLogServiceHandlers`.
pub struct OperationAuditLogProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::OperationAuditLogServiceHandlers for OperationAuditLogProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::audit::service::v1::ListOperationAuditLogResponse, StatusError> {
        let mut core = proto::proto::audit::service::v1::operation_audit_log_service_client::OperationAuditLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::audit::service::v1::GetOperationAuditLogRequest,
    ) -> Result<proto::proto::audit::service::v1::OperationAuditLog, StatusError> {
        let mut core = proto::proto::audit::service::v1::operation_audit_log_service_client::OperationAuditLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `OrgUnitServiceHandlers`.
pub struct OrgUnitProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::OrgUnitServiceHandlers for OrgUnitProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::identity::service::v1::ListOrgUnitResponse, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::org_unit_service_client::OrgUnitServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::GetOrgUnitRequest,
    ) -> Result<proto::proto::identity::service::v1::OrgUnit, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::org_unit_service_client::OrgUnitServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::CreateOrgUnitRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::org_unit_service_client::OrgUnitServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::UpdateOrgUnitRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::org_unit_service_client::OrgUnitServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::DeleteOrgUnitRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::org_unit_service_client::OrgUnitServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `PageServiceHandlers`.
pub struct PageProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::PageServiceHandlers for PageProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::content::service::v1::ListPageResponse, StatusError> {
        let mut core =
            proto::proto::content::service::v1::page_service_client::PageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::GetPageRequest,
    ) -> Result<proto::proto::content::service::v1::Page, StatusError> {
        let mut core =
            proto::proto::content::service::v1::page_service_client::PageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::CreatePageRequest,
    ) -> Result<proto::proto::content::service::v1::Page, StatusError> {
        let mut core =
            proto::proto::content::service::v1::page_service_client::PageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::UpdatePageRequest,
    ) -> Result<proto::proto::content::service::v1::Page, StatusError> {
        let mut core =
            proto::proto::content::service::v1::page_service_client::PageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::DeletePageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::content::service::v1::page_service_client::PageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `PermissionAuditLogServiceHandlers`.
pub struct PermissionAuditLogProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::PermissionAuditLogServiceHandlers for PermissionAuditLogProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::audit::service::v1::ListPermissionAuditLogResponse, StatusError> {
        let mut core = proto::proto::audit::service::v1::permission_audit_log_service_client::PermissionAuditLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::audit::service::v1::GetPermissionAuditLogRequest,
    ) -> Result<proto::proto::audit::service::v1::PermissionAuditLog, StatusError> {
        let mut core = proto::proto::audit::service::v1::permission_audit_log_service_client::PermissionAuditLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `PermissionGroupServiceHandlers`.
pub struct PermissionGroupProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::PermissionGroupServiceHandlers for PermissionGroupProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::permission::service::v1::ListPermissionGroupResponse, StatusError>
    {
        let mut core = proto::proto::permission::service::v1::permission_group_service_client::PermissionGroupServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::GetPermissionGroupRequest,
    ) -> Result<proto::proto::permission::service::v1::PermissionGroup, StatusError> {
        let mut core = proto::proto::permission::service::v1::permission_group_service_client::PermissionGroupServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::CreatePermissionGroupRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::permission::service::v1::permission_group_service_client::PermissionGroupServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::UpdatePermissionGroupRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::permission::service::v1::permission_group_service_client::PermissionGroupServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::DeletePermissionGroupRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::permission::service::v1::permission_group_service_client::PermissionGroupServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `PermissionServiceHandlers`.
pub struct PermissionProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::PermissionServiceHandlers for PermissionProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::permission::service::v1::ListPermissionResponse, StatusError> {
        let mut core = proto::proto::permission::service::v1::permission_service_client::PermissionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::GetPermissionRequest,
    ) -> Result<proto::proto::permission::service::v1::Permission, StatusError> {
        let mut core = proto::proto::permission::service::v1::permission_service_client::PermissionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::CreatePermissionRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::permission::service::v1::permission_service_client::PermissionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::UpdatePermissionRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::permission::service::v1::permission_service_client::PermissionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::DeletePermissionRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::permission::service::v1::permission_service_client::PermissionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn sync_permissions(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: pbjson_types::Empty,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let _ = (req, &self.state);
        Err(crate::state::internal_error("not implemented"))
    }
}

/// The pass-through proxy of `PolicyEvaluationLogServiceHandlers`.
pub struct PolicyEvaluationLogProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::PolicyEvaluationLogServiceHandlers for PolicyEvaluationLogProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::permission::service::v1::ListPolicyEvaluationLogResponse, StatusError>
    {
        let mut core = proto::proto::permission::service::v1::policy_evaluation_log_service_client::PolicyEvaluationLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::GetPolicyEvaluationLogRequest,
    ) -> Result<proto::proto::permission::service::v1::PolicyEvaluationLog, StatusError> {
        let mut core = proto::proto::permission::service::v1::policy_evaluation_log_service_client::PolicyEvaluationLogServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `PositionServiceHandlers`.
pub struct PositionProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::PositionServiceHandlers for PositionProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::identity::service::v1::ListPositionResponse, StatusError> {
        let mut core = proto::proto::identity::service::v1::position_service_client::PositionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::GetPositionRequest,
    ) -> Result<proto::proto::identity::service::v1::Position, StatusError> {
        let mut core = proto::proto::identity::service::v1::position_service_client::PositionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::CreatePositionRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::identity::service::v1::position_service_client::PositionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::UpdatePositionRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::identity::service::v1::position_service_client::PositionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::DeletePositionRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::identity::service::v1::position_service_client::PositionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `PostServiceHandlers`.
pub struct PostProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::PostServiceHandlers for PostProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::content::service::v1::ListPostResponse, StatusError> {
        let mut core =
            proto::proto::content::service::v1::post_service_client::PostServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::GetPostRequest,
    ) -> Result<proto::proto::content::service::v1::Post, StatusError> {
        let mut core =
            proto::proto::content::service::v1::post_service_client::PostServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::CreatePostRequest,
    ) -> Result<proto::proto::content::service::v1::Post, StatusError> {
        let mut core =
            proto::proto::content::service::v1::post_service_client::PostServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::UpdatePostRequest,
    ) -> Result<proto::proto::content::service::v1::Post, StatusError> {
        let mut core =
            proto::proto::content::service::v1::post_service_client::PostServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::DeletePostRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::content::service::v1::post_service_client::PostServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn translation_exists(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::PostTranslationExistsRequest,
    ) -> Result<proto::proto::content::service::v1::PostTranslationExistsResponse, StatusError>
    {
        let mut core =
            proto::proto::content::service::v1::post_service_client::PostServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .translation_exists(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `RoleServiceHandlers`.
pub struct RoleProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::RoleServiceHandlers for RoleProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::permission::service::v1::ListRoleResponse, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::role_service_client::RoleServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::GetRoleRequest,
    ) -> Result<proto::proto::permission::service::v1::Role, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::role_service_client::RoleServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::CreateRoleRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::role_service_client::RoleServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::UpdateRoleRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::role_service_client::RoleServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::permission::service::v1::DeleteRoleRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::role_service_client::RoleServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `SiteServiceHandlers`.
pub struct SiteProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::SiteServiceHandlers for SiteProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::site::service::v1::ListSiteResponse, StatusError> {
        let mut core = proto::proto::site::service::v1::site_service_client::SiteServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::GetSiteRequest,
    ) -> Result<proto::proto::site::service::v1::Site, StatusError> {
        let mut core = proto::proto::site::service::v1::site_service_client::SiteServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::CreateSiteRequest,
    ) -> Result<proto::proto::site::service::v1::Site, StatusError> {
        let mut core = proto::proto::site::service::v1::site_service_client::SiteServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::UpdateSiteRequest,
    ) -> Result<proto::proto::site::service::v1::Site, StatusError> {
        let mut core = proto::proto::site::service::v1::site_service_client::SiteServiceClient::new(
            self.state.core_channel.clone(),
        );
        let _ = core
            .update(tonic::Request::new(req))
            .await
            .map_err(map_status)?;
        Ok(<proto::proto::site::service::v1::Site>::default())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::DeleteSiteRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::site::service::v1::site_service_client::SiteServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `SiteSettingServiceHandlers`.
pub struct SiteSettingProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::SiteSettingServiceHandlers for SiteSettingProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::site::service::v1::ListSiteSettingResponse, StatusError> {
        let mut core = proto::proto::site::service::v1::site_setting_service_client::SiteSettingServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::GetSiteSettingRequest,
    ) -> Result<proto::proto::site::service::v1::SiteSetting, StatusError> {
        let mut core = proto::proto::site::service::v1::site_setting_service_client::SiteSettingServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::CreateSiteSettingRequest,
    ) -> Result<proto::proto::site::service::v1::SiteSetting, StatusError> {
        let mut core = proto::proto::site::service::v1::site_setting_service_client::SiteSettingServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::UpdateSiteSettingRequest,
    ) -> Result<proto::proto::site::service::v1::SiteSetting, StatusError> {
        let mut core = proto::proto::site::service::v1::site_setting_service_client::SiteSettingServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::DeleteSiteSettingRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::site::service::v1::site_setting_service_client::SiteSettingServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `StatsServiceHandlers`.
pub struct StatsProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::StatsServiceHandlers for StatsProxy {
    async fn get_dashboard_overview(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::stats::service::v1::GetDashboardOverviewRequest,
    ) -> Result<proto::proto::stats::service::v1::GetDashboardOverviewResponse, StatusError> {
        let mut core =
            proto::proto::stats::service::v1::stats_service_client::StatsServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get_dashboard_overview(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get_content_trend(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::stats::service::v1::GetContentTrendRequest,
    ) -> Result<proto::proto::stats::service::v1::GetContentTrendResponse, StatusError> {
        let mut core =
            proto::proto::stats::service::v1::stats_service_client::StatsServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get_content_trend(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get_interaction_stats(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::stats::service::v1::GetInteractionStatsRequest,
    ) -> Result<proto::proto::stats::service::v1::GetInteractionStatsResponse, StatusError> {
        let mut core =
            proto::proto::stats::service::v1::stats_service_client::StatsServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get_interaction_stats(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get_login_activity(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::stats::service::v1::GetLoginActivityRequest,
    ) -> Result<proto::proto::stats::service::v1::GetLoginActivityResponse, StatusError> {
        let mut core =
            proto::proto::stats::service::v1::stats_service_client::StatsServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get_login_activity(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `TagServiceHandlers`.
pub struct TagProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::TagServiceHandlers for TagProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::content::service::v1::ListTagResponse, StatusError> {
        let mut core =
            proto::proto::content::service::v1::tag_service_client::TagServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::GetTagRequest,
    ) -> Result<proto::proto::content::service::v1::Tag, StatusError> {
        let mut core =
            proto::proto::content::service::v1::tag_service_client::TagServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::CreateTagRequest,
    ) -> Result<proto::proto::content::service::v1::Tag, StatusError> {
        let mut core =
            proto::proto::content::service::v1::tag_service_client::TagServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::UpdateTagRequest,
    ) -> Result<proto::proto::content::service::v1::Tag, StatusError> {
        let mut core =
            proto::proto::content::service::v1::tag_service_client::TagServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::DeleteTagRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::content::service::v1::tag_service_client::TagServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `TaskServiceHandlers`.
pub struct TaskProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::TaskServiceHandlers for TaskProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::task::service::v1::ListTaskResponse, StatusError> {
        let mut core = proto::proto::task::service::v1::task_service_client::TaskServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::task::service::v1::GetTaskRequest,
    ) -> Result<proto::proto::task::service::v1::Task, StatusError> {
        let mut core = proto::proto::task::service::v1::task_service_client::TaskServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::task::service::v1::CreateTaskRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::task::service::v1::task_service_client::TaskServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::task::service::v1::UpdateTaskRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::task::service::v1::task_service_client::TaskServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::task::service::v1::DeleteTaskRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::task::service::v1::task_service_client::TaskServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn list_task_type_name(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: pbjson_types::Empty,
    ) -> Result<proto::proto::task::service::v1::ListTaskTypeNameResponse, StatusError> {
        let mut core = proto::proto::task::service::v1::task_service_client::TaskServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list_task_type_name(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn restart_all_task(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: pbjson_types::Empty,
    ) -> Result<proto::proto::task::service::v1::RestartAllTaskResponse, StatusError> {
        let mut core = proto::proto::task::service::v1::task_service_client::TaskServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .restart_all_task(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn start_all_task(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: pbjson_types::Empty,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::task::service::v1::task_service_client::TaskServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .start_all_task(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn stop_all_task(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: pbjson_types::Empty,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::task::service::v1::task_service_client::TaskServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .stop_all_task(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn control_task(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::task::service::v1::ControlTaskRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::task::service::v1::task_service_client::TaskServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .control_task(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn list_task_executions(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::task::service::v1::ListTaskExecutionsRequest,
    ) -> Result<proto::proto::task::service::v1::ListTaskExecutionsResponse, StatusError> {
        let mut core = proto::proto::task::service::v1::task_service_client::TaskServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list_task_executions(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `TenantServiceHandlers`.
pub struct TenantProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::TenantServiceHandlers for TenantProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::identity::service::v1::ListTenantResponse, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::tenant_service_client::TenantServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::GetTenantRequest,
    ) -> Result<proto::proto::identity::service::v1::Tenant, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::tenant_service_client::TenantServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::CreateTenantRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::tenant_service_client::TenantServiceClient::new(
                self.state.core_channel.clone(),
            );
        let _ = core
            .create(tonic::Request::new(req))
            .await
            .map_err(map_status)?;
        Ok(<pbjson_types::Empty>::default())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::UpdateTenantRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::tenant_service_client::TenantServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::DeleteTenantRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::tenant_service_client::TenantServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create_tenant_with_admin_user(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::CreateTenantWithAdminUserRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::tenant_service_client::TenantServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create_tenant_with_admin_user(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn tenant_exists(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::TenantExistsRequest,
    ) -> Result<proto::proto::identity::service::v1::TenantExistsResponse, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::tenant_service_client::TenantServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .tenant_exists(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `TranslatorServiceHandlers`.
pub struct TranslatorProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::TranslatorServiceHandlers for TranslatorProxy {
    async fn translate(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::translator::service::v1::TranslateRequest,
    ) -> Result<proto::proto::translator::service::v1::TranslateResponse, StatusError> {
        let mut core = proto::proto::translator::service::v1::translator_service_client::TranslatorServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .translate(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}

/// The pass-through proxy of `UserProfileServiceHandlers`.
pub struct UserProfileProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::UserProfileServiceHandlers for UserProfileProxy {
    async fn get_user(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: pbjson_types::Empty,
    ) -> Result<proto::proto::identity::service::v1::User, StatusError> {
        let mut core = proto::proto::identity::service::v1::user_profile_service_client::UserProfileServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get_user(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update_user(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::UpdateUserRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = proto::proto::identity::service::v1::user_profile_service_client::UserProfileServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .update_user(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn change_password(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::ChangePasswordRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let _ = (req, &self.state);
        Err(crate::state::internal_error("not implemented"))
    }

    async fn bind_contact(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::BindContactRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let _ = (req, &self.state);
        Err(crate::state::internal_error("not implemented"))
    }

    async fn verify_contact(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::VerifyContactRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let _ = (req, &self.state);
        Err(crate::state::internal_error("not implemented"))
    }
}

/// The pass-through proxy of `UserServiceHandlers`.
pub struct UserProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::UserServiceHandlers for UserProxy {
    async fn list(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::identity::service::v1::ListUserResponse, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::user_service_client::UserServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .list(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::GetUserRequest,
    ) -> Result<proto::proto::identity::service::v1::User, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::user_service_client::UserServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::CreateUserRequest,
    ) -> Result<proto::proto::identity::service::v1::User, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::user_service_client::UserServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .create(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::UpdateUserRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::user_service_client::UserServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .update(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn delete(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::DeleteUserRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::user_service_client::UserServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .delete(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn user_exists(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::UserExistsRequest,
    ) -> Result<proto::proto::identity::service::v1::UserExistsResponse, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::user_service_client::UserServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .user_exists(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn edit_user_password(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::identity::service::v1::EditUserPasswordRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core =
            proto::proto::identity::service::v1::user_service_client::UserServiceClient::new(
                self.state.core_channel.clone(),
            );
        core.update(tonic::Request::new(
            proto::proto::identity::service::v1::UpdateUserRequest {
                id: req.user_id,
                password: Some(req.new_password),
                ..Default::default()
            },
        ))
        .await
        .map_err(map_status)?;
        Ok(pbjson_types::Empty {})
    }
}
