//! ComponentDefinitionService — the admin face's component/field-definition
//! registry. The contract declares the service on the admin face only (the
//! messages live in the content domain); the domain service lands with its
//! feature batch — every operation answers NOT_IMPLEMENTED until then.

use std::sync::Arc;

use pbjson_types::Empty;

use crate::state::{AppState, StatusError};

use proto::proto::content::service::v1::{
    ComponentDefinition, CreateComponentDefinitionRequest, DeleteComponentDefinitionRequest,
    GetComponentDefinitionRequest, ListFieldDefinitionsRequest, ListFieldDefinitionsResponse,
    SaveComponentFieldDefinitionsRequest, UpdateComponentDefinitionRequest,
};
use proto::proto::pagination::PagingRequest;

type Ctx = rushwind_http_binding::ctx::RequestContext;

fn not_implemented(message: &'static str) -> StatusError {
    StatusError::new(501, "NOT_IMPLEMENTED", message)
}

pub struct ComponentDefinitionService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::ComponentDefinitionServiceHandlers for ComponentDefinitionService {
    async fn list(
        &self,
        _ctx: Ctx,
        _req: PagingRequest,
    ) -> Result<proto::proto::content::service::v1::ListComponentDefinitionsResponse, StatusError>
    {
        Err(not_implemented(
            "component definitions land with the content-model feature batch",
        ))
    }

    async fn get(
        &self,
        _ctx: Ctx,
        _req: GetComponentDefinitionRequest,
    ) -> Result<ComponentDefinition, StatusError> {
        Err(not_implemented(
            "component definitions land with the content-model feature batch",
        ))
    }

    async fn create(
        &self,
        _ctx: Ctx,
        _req: CreateComponentDefinitionRequest,
    ) -> Result<ComponentDefinition, StatusError> {
        Err(not_implemented(
            "component definitions land with the content-model feature batch",
        ))
    }

    async fn update(
        &self,
        _ctx: Ctx,
        _req: UpdateComponentDefinitionRequest,
    ) -> Result<ComponentDefinition, StatusError> {
        Err(not_implemented(
            "component definitions land with the content-model feature batch",
        ))
    }

    async fn delete(
        &self,
        _ctx: Ctx,
        _req: DeleteComponentDefinitionRequest,
    ) -> Result<Empty, StatusError> {
        Err(not_implemented(
            "component definitions land with the content-model feature batch",
        ))
    }

    async fn list_field_definitions(
        &self,
        _ctx: Ctx,
        _req: ListFieldDefinitionsRequest,
    ) -> Result<ListFieldDefinitionsResponse, StatusError> {
        Err(not_implemented(
            "component definitions land with the content-model feature batch",
        ))
    }

    async fn save_field_definitions(
        &self,
        _ctx: Ctx,
        _req: SaveComponentFieldDefinitionsRequest,
    ) -> Result<ListFieldDefinitionsResponse, StatusError> {
        Err(not_implemented(
            "component definitions land with the content-model feature batch",
        ))
    }
}
