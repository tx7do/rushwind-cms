//! Generated thin-BFF proxies — a table of `passthrough_proxy!`
//! invocations, one per BFF face (see services/proxy_kit.rs for the
//! pass/drop/stub/hand method kinds; the BFF and domain methods share
//! their message types by contract, so no mapping rides here).
//! DO NOT EDIT; regenerate via scripts/gen-proxies.py when the
//! contract re-syncs. Hand-written faces (authentication — captcha/
//! cookie concerns; admin-portal aggregation; file-transfer multipart)
//! live in their own modules.
#![allow(clippy::all)]
#![allow(missing_docs)]

use crate::passthrough_proxy;

passthrough_proxy! {
    /// The pass-through proxy of `ApiAuditLogServiceHandlers`.
    proto::gen_admin::services::ApiAuditLogServiceHandlers for ApiAuditLogProxy {
        client: proto::proto::audit::service::v1::api_audit_log_service_client::ApiAuditLogServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::audit::service::v1::ListApiAuditLogResponse,
            pass get(proto::proto::audit::service::v1::GetApiAuditLogRequest) -> proto::proto::audit::service::v1::ApiAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `ApiServiceHandlers`.
    proto::gen_admin::services::ApiServiceHandlers for ApiProxy {
        client: proto::proto::permission::service::v1::api_service_client::ApiServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::permission::service::v1::ListApiResponse,
            pass get(proto::proto::permission::service::v1::GetApiRequest) -> proto::proto::permission::service::v1::Api,
            pass create(proto::proto::permission::service::v1::CreateApiRequest) -> pbjson_types::Empty,
            pass update(proto::proto::permission::service::v1::UpdateApiRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::permission::service::v1::DeleteApiRequest) -> pbjson_types::Empty,
            hand sync_apis(pbjson_types::Empty) -> pbjson_types::Empty => sync_apis,
            hand get_walk_route_data(pbjson_types::Empty) -> proto::proto::permission::service::v1::ListApiResponse => walk_route_data,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `CategoryServiceHandlers`.
    proto::gen_admin::services::CategoryServiceHandlers for CategoryProxy {
        client: proto::proto::content::service::v1::category_service_client::CategoryServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::content::service::v1::ListCategoryResponse,
            pass get(proto::proto::content::service::v1::GetCategoryRequest) -> proto::proto::content::service::v1::Category,
            pass create(proto::proto::content::service::v1::CreateCategoryRequest) -> proto::proto::content::service::v1::Category,
            pass update(proto::proto::content::service::v1::UpdateCategoryRequest) -> proto::proto::content::service::v1::Category,
            pass delete(proto::proto::content::service::v1::DeleteCategoryRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `CommentServiceHandlers`.
    proto::gen_admin::services::CommentServiceHandlers for CommentProxy {
        client: proto::proto::comment::service::v1::comment_service_client::CommentServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::comment::service::v1::ListCommentResponse,
            pass get(proto::proto::comment::service::v1::GetCommentRequest) -> proto::proto::comment::service::v1::Comment,
            pass create(proto::proto::comment::service::v1::CreateCommentRequest) -> proto::proto::comment::service::v1::Comment,
            pass update(proto::proto::comment::service::v1::UpdateCommentRequest) -> proto::proto::comment::service::v1::Comment,
            pass delete(proto::proto::comment::service::v1::DeleteCommentRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `ContentModelServiceHandlers`.
    proto::gen_admin::services::ContentModelServiceHandlers for ContentModelProxy {
        client: proto::proto::content::service::v1::content_model_service_client::ContentModelServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::content::service::v1::ListContentModelResponse,
            pass get(proto::proto::content::service::v1::GetContentModelRequest) -> proto::proto::content::service::v1::ContentModel,
            pass create(proto::proto::content::service::v1::CreateContentModelRequest) -> proto::proto::content::service::v1::ContentModel,
            pass update(proto::proto::content::service::v1::UpdateContentModelRequest) -> proto::proto::content::service::v1::ContentModel,
            pass delete(proto::proto::content::service::v1::DeleteContentModelRequest) -> pbjson_types::Empty,
            pass list_field_definitions(proto::proto::content::service::v1::ListFieldDefinitionsRequest) -> proto::proto::content::service::v1::ListFieldDefinitionsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `DataAccessAuditLogServiceHandlers`.
    proto::gen_admin::services::DataAccessAuditLogServiceHandlers for DataAccessAuditLogProxy {
        client: proto::proto::audit::service::v1::data_access_audit_log_service_client::DataAccessAuditLogServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::audit::service::v1::ListDataAccessAuditLogResponse,
            pass get(proto::proto::audit::service::v1::GetDataAccessAuditLogRequest) -> proto::proto::audit::service::v1::DataAccessAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `DictEntryServiceHandlers`.
    proto::gen_admin::services::DictEntryServiceHandlers for DictEntryProxy {
        client: proto::proto::dict::service::v1::dict_entry_service_client::DictEntryServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::dict::service::v1::ListDictEntryResponse,
            pass create(proto::proto::dict::service::v1::CreateDictEntryRequest) -> pbjson_types::Empty,
            pass update(proto::proto::dict::service::v1::UpdateDictEntryRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::dict::service::v1::DeleteDictEntryRequest) -> pbjson_types::Empty,
            pass list_by_type_code(proto::proto::dict::service::v1::ListDictEntryByTypeCodeRequest) -> proto::proto::dict::service::v1::ListDictEntryByTypeCodeResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `DictTypeServiceHandlers`.
    proto::gen_admin::services::DictTypeServiceHandlers for DictTypeProxy {
        client: proto::proto::dict::service::v1::dict_type_service_client::DictTypeServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::dict::service::v1::ListDictTypeResponse,
            pass get(proto::proto::dict::service::v1::GetDictTypeRequest) -> proto::proto::dict::service::v1::DictType,
            pass create(proto::proto::dict::service::v1::CreateDictTypeRequest) -> pbjson_types::Empty,
            pass update(proto::proto::dict::service::v1::UpdateDictTypeRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::dict::service::v1::DeleteDictTypeRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `FileServiceHandlers`.
    proto::gen_admin::services::FileServiceHandlers for FileProxy {
        client: proto::proto::storage::service::v1::file_service_client::FileServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::storage::service::v1::ListFileResponse,
            pass get(proto::proto::storage::service::v1::GetFileRequest) -> proto::proto::storage::service::v1::File,
            hand create(proto::proto::storage::service::v1::CreateFileRequest) -> pbjson_types::Empty => file_create,
            pass update(proto::proto::storage::service::v1::UpdateFileRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::storage::service::v1::DeleteFileRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InteractionAdminServiceHandlers`.
    proto::gen_admin::services::InteractionAdminServiceHandlers for InteractionAdminProxy {
        client: proto::proto::interaction::service::v1::interaction_admin_service_client::InteractionAdminServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass purge_target_interactions(proto::proto::interaction::service::v1::PurgeTargetInteractionsRequest) -> proto::proto::interaction::service::v1::PurgeTargetInteractionsResponse,
            pass purge_user_interactions(proto::proto::interaction::service::v1::PurgeUserInteractionsRequest) -> proto::proto::interaction::service::v1::PurgeUserInteractionsResponse,
            pass reset_counter(proto::proto::interaction::service::v1::ResetCounterRequest) -> proto::proto::interaction::service::v1::ResetCounterResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InternalMessageCategoryServiceHandlers`.
    proto::gen_admin::services::InternalMessageCategoryServiceHandlers for InternalMessageCategoryProxy {
        client: proto::proto::internal_message::service::v1::internal_message_category_service_client::InternalMessageCategoryServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::internal_message::service::v1::ListInternalMessageCategoryResponse,
            pass get(proto::proto::internal_message::service::v1::GetInternalMessageCategoryRequest) -> proto::proto::internal_message::service::v1::InternalMessageCategory,
            pass create(proto::proto::internal_message::service::v1::CreateInternalMessageCategoryRequest) -> pbjson_types::Empty,
            pass update(proto::proto::internal_message::service::v1::UpdateInternalMessageCategoryRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::internal_message::service::v1::DeleteInternalMessageCategoryRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InternalMessageRecipientServiceHandlers`.
    proto::gen_admin::services::InternalMessageRecipientServiceHandlers for InternalMessageRecipientProxy {
        client: proto::proto::internal_message::service::v1::internal_message_recipient_service_client::InternalMessageRecipientServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list_user_inbox(proto::proto::pagination::PagingRequest) -> proto::proto::internal_message::service::v1::ListUserInboxResponse,
            pass delete_notification_from_inbox(proto::proto::internal_message::service::v1::DeleteNotificationFromInboxRequest) -> pbjson_types::Empty,
            pass mark_notification_as_read(proto::proto::internal_message::service::v1::MarkNotificationAsReadRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InternalMessageServiceHandlers`.
    proto::gen_admin::services::InternalMessageServiceHandlers for InternalMessageProxy {
        client: proto::proto::internal_message::service::v1::internal_message_service_client::InternalMessageServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list_message(proto::proto::pagination::PagingRequest) -> proto::proto::internal_message::service::v1::ListInternalMessageResponse,
            pass get_message(proto::proto::internal_message::service::v1::GetInternalMessageRequest) -> proto::proto::internal_message::service::v1::InternalMessage,
            pass update_message(proto::proto::internal_message::service::v1::UpdateInternalMessageRequest) -> pbjson_types::Empty,
            pass delete_message(proto::proto::internal_message::service::v1::DeleteInternalMessageRequest) -> pbjson_types::Empty,
            pass send_message(proto::proto::internal_message::service::v1::SendMessageRequest) -> proto::proto::internal_message::service::v1::SendMessageResponse,
            pass revoke_message(proto::proto::internal_message::service::v1::RevokeMessageRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `LanguageServiceHandlers`.
    proto::gen_admin::services::LanguageServiceHandlers for LanguageProxy {
        client: proto::proto::dict::service::v1::language_service_client::LanguageServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::dict::service::v1::ListLanguageResponse,
            pass get(proto::proto::dict::service::v1::GetLanguageRequest) -> proto::proto::dict::service::v1::Language,
            pass create(proto::proto::dict::service::v1::CreateLanguageRequest) -> pbjson_types::Empty,
            pass update(proto::proto::dict::service::v1::UpdateLanguageRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::dict::service::v1::DeleteLanguageRequest) -> pbjson_types::Empty,
            pass batch_create(proto::proto::dict::service::v1::BatchCreateLanguagesRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `LoginAuditLogServiceHandlers`.
    proto::gen_admin::services::LoginAuditLogServiceHandlers for LoginAuditLogProxy {
        client: proto::proto::audit::service::v1::login_audit_log_service_client::LoginAuditLogServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::audit::service::v1::ListLoginAuditLogResponse,
            pass get(proto::proto::audit::service::v1::GetLoginAuditLogRequest) -> proto::proto::audit::service::v1::LoginAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `LoginPolicyServiceHandlers`.
    proto::gen_admin::services::LoginPolicyServiceHandlers for LoginPolicyProxy {
        client: proto::proto::authentication::service::v1::login_policy_service_client::LoginPolicyServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::authentication::service::v1::ListLoginPolicyResponse,
            pass get(proto::proto::authentication::service::v1::GetLoginPolicyRequest) -> proto::proto::authentication::service::v1::LoginPolicy,
            pass create(proto::proto::authentication::service::v1::CreateLoginPolicyRequest) -> pbjson_types::Empty,
            pass update(proto::proto::authentication::service::v1::UpdateLoginPolicyRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::authentication::service::v1::DeleteLoginPolicyRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `MediaAssetServiceHandlers`.
    proto::gen_admin::services::MediaAssetServiceHandlers for MediaAssetProxy {
        client: proto::proto::media::service::v1::media_asset_service_client::MediaAssetServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::media::service::v1::ListMediaAssetResponse,
            pass get(proto::proto::media::service::v1::GetMediaAssetRequest) -> proto::proto::media::service::v1::MediaAsset,
            pass create(proto::proto::media::service::v1::CreateMediaAssetRequest) -> proto::proto::media::service::v1::MediaAsset,
            pass update(proto::proto::media::service::v1::UpdateMediaAssetRequest) -> proto::proto::media::service::v1::MediaAsset,
            pass delete(proto::proto::media::service::v1::DeleteMediaAssetRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `MenuServiceHandlers`.
    proto::gen_admin::services::MenuServiceHandlers for MenuProxy {
        client: proto::proto::permission::service::v1::menu_service_client::MenuServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::permission::service::v1::ListMenuResponse,
            pass get(proto::proto::permission::service::v1::GetMenuRequest) -> proto::proto::permission::service::v1::Menu,
            pass create(proto::proto::permission::service::v1::CreateMenuRequest) -> pbjson_types::Empty,
            pass update(proto::proto::permission::service::v1::UpdateMenuRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::permission::service::v1::DeleteMenuRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NavigationItemServiceHandlers`.
    proto::gen_admin::services::NavigationItemServiceHandlers for NavigationItemProxy {
        client: proto::proto::site::service::v1::navigation_item_service_client::NavigationItemServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::site::service::v1::ListNavigationItemResponse,
            pass get(proto::proto::site::service::v1::GetNavigationItemRequest) -> proto::proto::site::service::v1::NavigationItem,
            pass create(proto::proto::site::service::v1::CreateNavigationItemRequest) -> proto::proto::site::service::v1::NavigationItem,
            pass update(proto::proto::site::service::v1::UpdateNavigationItemRequest) -> proto::proto::site::service::v1::NavigationItem,
            pass delete(proto::proto::site::service::v1::DeleteNavigationItemRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NavigationServiceHandlers`.
    proto::gen_admin::services::NavigationServiceHandlers for NavigationProxy {
        client: proto::proto::site::service::v1::navigation_service_client::NavigationServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::site::service::v1::ListNavigationResponse,
            pass get(proto::proto::site::service::v1::GetNavigationRequest) -> proto::proto::site::service::v1::Navigation,
            pass create(proto::proto::site::service::v1::CreateNavigationRequest) -> proto::proto::site::service::v1::Navigation,
            pass update(proto::proto::site::service::v1::UpdateNavigationRequest) -> proto::proto::site::service::v1::Navigation,
            pass delete(proto::proto::site::service::v1::DeleteNavigationRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `OperationAuditLogServiceHandlers`.
    proto::gen_admin::services::OperationAuditLogServiceHandlers for OperationAuditLogProxy {
        client: proto::proto::audit::service::v1::operation_audit_log_service_client::OperationAuditLogServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::audit::service::v1::ListOperationAuditLogResponse,
            pass get(proto::proto::audit::service::v1::GetOperationAuditLogRequest) -> proto::proto::audit::service::v1::OperationAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `OrgUnitServiceHandlers`.
    proto::gen_admin::services::OrgUnitServiceHandlers for OrgUnitProxy {
        client: proto::proto::identity::service::v1::org_unit_service_client::OrgUnitServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::identity::service::v1::ListOrgUnitResponse,
            pass get(proto::proto::identity::service::v1::GetOrgUnitRequest) -> proto::proto::identity::service::v1::OrgUnit,
            pass create(proto::proto::identity::service::v1::CreateOrgUnitRequest) -> pbjson_types::Empty,
            pass update(proto::proto::identity::service::v1::UpdateOrgUnitRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::identity::service::v1::DeleteOrgUnitRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PageServiceHandlers`.
    proto::gen_admin::services::PageServiceHandlers for PageProxy {
        client: proto::proto::content::service::v1::page_service_client::PageServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::content::service::v1::ListPageResponse,
            pass get(proto::proto::content::service::v1::GetPageRequest) -> proto::proto::content::service::v1::Page,
            pass create(proto::proto::content::service::v1::CreatePageRequest) -> proto::proto::content::service::v1::Page,
            pass update(proto::proto::content::service::v1::UpdatePageRequest) -> proto::proto::content::service::v1::Page,
            pass delete(proto::proto::content::service::v1::DeletePageRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PermissionAuditLogServiceHandlers`.
    proto::gen_admin::services::PermissionAuditLogServiceHandlers for PermissionAuditLogProxy {
        client: proto::proto::audit::service::v1::permission_audit_log_service_client::PermissionAuditLogServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::audit::service::v1::ListPermissionAuditLogResponse,
            pass get(proto::proto::audit::service::v1::GetPermissionAuditLogRequest) -> proto::proto::audit::service::v1::PermissionAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PermissionGroupServiceHandlers`.
    proto::gen_admin::services::PermissionGroupServiceHandlers for PermissionGroupProxy {
        client: proto::proto::permission::service::v1::permission_group_service_client::PermissionGroupServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::permission::service::v1::ListPermissionGroupResponse,
            pass get(proto::proto::permission::service::v1::GetPermissionGroupRequest) -> proto::proto::permission::service::v1::PermissionGroup,
            pass create(proto::proto::permission::service::v1::CreatePermissionGroupRequest) -> pbjson_types::Empty,
            pass update(proto::proto::permission::service::v1::UpdatePermissionGroupRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::permission::service::v1::DeletePermissionGroupRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PermissionServiceHandlers`.
    proto::gen_admin::services::PermissionServiceHandlers for PermissionProxy {
        client: proto::proto::permission::service::v1::permission_service_client::PermissionServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::permission::service::v1::ListPermissionResponse,
            pass get(proto::proto::permission::service::v1::GetPermissionRequest) -> proto::proto::permission::service::v1::Permission,
            pass create(proto::proto::permission::service::v1::CreatePermissionRequest) -> pbjson_types::Empty,
            pass update(proto::proto::permission::service::v1::UpdatePermissionRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::permission::service::v1::DeletePermissionRequest) -> pbjson_types::Empty,
            hand sync_permissions(pbjson_types::Empty) -> pbjson_types::Empty => sync_permissions,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PolicyEvaluationLogServiceHandlers`.
    proto::gen_admin::services::PolicyEvaluationLogServiceHandlers for PolicyEvaluationLogProxy {
        client: proto::proto::permission::service::v1::policy_evaluation_log_service_client::PolicyEvaluationLogServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::permission::service::v1::ListPolicyEvaluationLogResponse,
            pass get(proto::proto::permission::service::v1::GetPolicyEvaluationLogRequest) -> proto::proto::permission::service::v1::PolicyEvaluationLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PositionServiceHandlers`.
    proto::gen_admin::services::PositionServiceHandlers for PositionProxy {
        client: proto::proto::identity::service::v1::position_service_client::PositionServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::identity::service::v1::ListPositionResponse,
            pass get(proto::proto::identity::service::v1::GetPositionRequest) -> proto::proto::identity::service::v1::Position,
            pass create(proto::proto::identity::service::v1::CreatePositionRequest) -> pbjson_types::Empty,
            pass update(proto::proto::identity::service::v1::UpdatePositionRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::identity::service::v1::DeletePositionRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PostServiceHandlers`.
    proto::gen_admin::services::PostServiceHandlers for PostProxy {
        client: proto::proto::content::service::v1::post_service_client::PostServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::content::service::v1::ListPostResponse,
            pass get(proto::proto::content::service::v1::GetPostRequest) -> proto::proto::content::service::v1::Post,
            pass create(proto::proto::content::service::v1::CreatePostRequest) -> proto::proto::content::service::v1::Post,
            pass update(proto::proto::content::service::v1::UpdatePostRequest) -> proto::proto::content::service::v1::Post,
            pass delete(proto::proto::content::service::v1::DeletePostRequest) -> pbjson_types::Empty,
            pass translation_exists(proto::proto::content::service::v1::PostTranslationExistsRequest) -> proto::proto::content::service::v1::PostTranslationExistsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `RoleServiceHandlers`.
    proto::gen_admin::services::RoleServiceHandlers for RoleProxy {
        client: proto::proto::permission::service::v1::role_service_client::RoleServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::permission::service::v1::ListRoleResponse,
            pass get(proto::proto::permission::service::v1::GetRoleRequest) -> proto::proto::permission::service::v1::Role,
            pass create(proto::proto::permission::service::v1::CreateRoleRequest) -> pbjson_types::Empty,
            pass update(proto::proto::permission::service::v1::UpdateRoleRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::permission::service::v1::DeleteRoleRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `SiteServiceHandlers`.
    proto::gen_admin::services::SiteServiceHandlers for SiteProxy {
        client: proto::proto::site::service::v1::site_service_client::SiteServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::site::service::v1::ListSiteResponse,
            pass get(proto::proto::site::service::v1::GetSiteRequest) -> proto::proto::site::service::v1::Site,
            pass create(proto::proto::site::service::v1::CreateSiteRequest) -> proto::proto::site::service::v1::Site,
            drop update(proto::proto::site::service::v1::UpdateSiteRequest) -> proto::proto::site::service::v1::Site,
            pass delete(proto::proto::site::service::v1::DeleteSiteRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `SiteSettingServiceHandlers`.
    proto::gen_admin::services::SiteSettingServiceHandlers for SiteSettingProxy {
        client: proto::proto::site::service::v1::site_setting_service_client::SiteSettingServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::site::service::v1::ListSiteSettingResponse,
            pass get(proto::proto::site::service::v1::GetSiteSettingRequest) -> proto::proto::site::service::v1::SiteSetting,
            pass create(proto::proto::site::service::v1::CreateSiteSettingRequest) -> proto::proto::site::service::v1::SiteSetting,
            pass update(proto::proto::site::service::v1::UpdateSiteSettingRequest) -> proto::proto::site::service::v1::SiteSetting,
            pass delete(proto::proto::site::service::v1::DeleteSiteSettingRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `StatsServiceHandlers`.
    proto::gen_admin::services::StatsServiceHandlers for StatsProxy {
        client: proto::proto::stats::service::v1::stats_service_client::StatsServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass get_dashboard_overview(proto::proto::stats::service::v1::GetDashboardOverviewRequest) -> proto::proto::stats::service::v1::GetDashboardOverviewResponse,
            pass get_content_trend(proto::proto::stats::service::v1::GetContentTrendRequest) -> proto::proto::stats::service::v1::GetContentTrendResponse,
            pass get_interaction_stats(proto::proto::stats::service::v1::GetInteractionStatsRequest) -> proto::proto::stats::service::v1::GetInteractionStatsResponse,
            pass get_login_activity(proto::proto::stats::service::v1::GetLoginActivityRequest) -> proto::proto::stats::service::v1::GetLoginActivityResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TagServiceHandlers`.
    proto::gen_admin::services::TagServiceHandlers for TagProxy {
        client: proto::proto::content::service::v1::tag_service_client::TagServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::content::service::v1::ListTagResponse,
            pass get(proto::proto::content::service::v1::GetTagRequest) -> proto::proto::content::service::v1::Tag,
            pass create(proto::proto::content::service::v1::CreateTagRequest) -> proto::proto::content::service::v1::Tag,
            pass update(proto::proto::content::service::v1::UpdateTagRequest) -> proto::proto::content::service::v1::Tag,
            pass delete(proto::proto::content::service::v1::DeleteTagRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TaskServiceHandlers`.
    proto::gen_admin::services::TaskServiceHandlers for TaskProxy {
        client: proto::proto::task::service::v1::task_service_client::TaskServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::task::service::v1::ListTaskResponse,
            pass get(proto::proto::task::service::v1::GetTaskRequest) -> proto::proto::task::service::v1::Task,
            pass create(proto::proto::task::service::v1::CreateTaskRequest) -> pbjson_types::Empty,
            pass update(proto::proto::task::service::v1::UpdateTaskRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::task::service::v1::DeleteTaskRequest) -> pbjson_types::Empty,
            pass list_task_type_name(pbjson_types::Empty) -> proto::proto::task::service::v1::ListTaskTypeNameResponse,
            pass restart_all_task(pbjson_types::Empty) -> proto::proto::task::service::v1::RestartAllTaskResponse,
            pass start_all_task(pbjson_types::Empty) -> pbjson_types::Empty,
            pass stop_all_task(pbjson_types::Empty) -> pbjson_types::Empty,
            pass control_task(proto::proto::task::service::v1::ControlTaskRequest) -> pbjson_types::Empty,
            pass list_task_executions(proto::proto::task::service::v1::ListTaskExecutionsRequest) -> proto::proto::task::service::v1::ListTaskExecutionsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TenantServiceHandlers`.
    proto::gen_admin::services::TenantServiceHandlers for TenantProxy {
        client: proto::proto::identity::service::v1::tenant_service_client::TenantServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::identity::service::v1::ListTenantResponse,
            pass get(proto::proto::identity::service::v1::GetTenantRequest) -> proto::proto::identity::service::v1::Tenant,
            drop create(proto::proto::identity::service::v1::CreateTenantRequest) -> pbjson_types::Empty,
            pass update(proto::proto::identity::service::v1::UpdateTenantRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::identity::service::v1::DeleteTenantRequest) -> pbjson_types::Empty,
            pass create_tenant_with_admin_user(proto::proto::identity::service::v1::CreateTenantWithAdminUserRequest) -> pbjson_types::Empty,
            pass tenant_exists(proto::proto::identity::service::v1::TenantExistsRequest) -> proto::proto::identity::service::v1::TenantExistsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TranslatorServiceHandlers`.
    proto::gen_admin::services::TranslatorServiceHandlers for TranslatorProxy {
        client: proto::proto::translator::service::v1::translator_service_client::TranslatorServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass translate(proto::proto::translator::service::v1::TranslateRequest) -> proto::proto::translator::service::v1::TranslateResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `UserProfileServiceHandlers`.
    proto::gen_admin::services::UserProfileServiceHandlers for UserProfileProxy {
        client: proto::proto::identity::service::v1::user_profile_service_client::UserProfileServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass get_user(pbjson_types::Empty) -> proto::proto::identity::service::v1::User,
            pass update_user(proto::proto::identity::service::v1::UpdateUserRequest) -> pbjson_types::Empty,
            hand change_password(proto::proto::identity::service::v1::ChangePasswordRequest) -> pbjson_types::Empty => change_password,
            hand bind_contact(proto::proto::identity::service::v1::BindContactRequest) -> pbjson_types::Empty => bind_contact,
            hand verify_contact(proto::proto::identity::service::v1::VerifyContactRequest) -> pbjson_types::Empty => verify_contact,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `UserServiceHandlers`.
    proto::gen_admin::services::UserServiceHandlers for UserProxy {
        client: proto::proto::identity::service::v1::user_service_client::UserServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::identity::service::v1::ListUserResponse,
            pass get(proto::proto::identity::service::v1::GetUserRequest) -> proto::proto::identity::service::v1::User,
            pass create(proto::proto::identity::service::v1::CreateUserRequest) -> proto::proto::identity::service::v1::User,
            pass update(proto::proto::identity::service::v1::UpdateUserRequest) -> pbjson_types::Empty,
            pass delete(proto::proto::identity::service::v1::DeleteUserRequest) -> pbjson_types::Empty,
            pass user_exists(proto::proto::identity::service::v1::UserExistsRequest) -> proto::proto::identity::service::v1::UserExistsResponse,
            hand edit_user_password(proto::proto::identity::service::v1::EditUserPasswordRequest) -> pbjson_types::Empty => edit_user_password,
        ]
    }
}
