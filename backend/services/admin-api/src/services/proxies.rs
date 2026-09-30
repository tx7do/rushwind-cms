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
use proto::proto::audit::service::v1 as audit_v1;
use proto::proto::authentication::service::v1 as authentication_v1;
use proto::proto::comment::service::v1 as comment_v1;
use proto::proto::content::service::v1 as content_v1;
use proto::proto::dict::service::v1 as dict_v1;
use proto::proto::identity::service::v1 as identity_v1;
use proto::proto::interaction::service::v1 as interaction_v1;
use proto::proto::internal_message::service::v1 as internal_message_v1;
use proto::proto::media::service::v1 as media_v1;
use proto::proto::pagination;
use proto::proto::permission::service::v1 as permission_v1;
use proto::proto::site::service::v1 as site_v1;
use proto::proto::stats::service::v1 as stats_v1;
use proto::proto::storage::service::v1 as storage_v1;
use proto::proto::task::service::v1 as task_v1;
use proto::proto::translator::service::v1 as translator_v1;

passthrough_proxy! {
    /// The pass-through proxy of `ApiAuditLogServiceHandlers`.
    proto::gen_admin::services::ApiAuditLogServiceHandlers for ApiAuditLogProxy {
        client: audit_v1::api_audit_log_service_client::ApiAuditLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> audit_v1::ListApiAuditLogResponse,
            pass get(audit_v1::GetApiAuditLogRequest) -> audit_v1::ApiAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `ApiServiceHandlers`.
    proto::gen_admin::services::ApiServiceHandlers for ApiProxy {
        client: permission_v1::api_service_client::ApiServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListApiResponse,
            pass get(permission_v1::GetApiRequest) -> permission_v1::Api,
            pass create(permission_v1::CreateApiRequest) -> pbjson_types::Empty,
            pass update(permission_v1::UpdateApiRequest) -> pbjson_types::Empty,
            pass delete(permission_v1::DeleteApiRequest) -> pbjson_types::Empty,
            hand sync_apis(pbjson_types::Empty) -> pbjson_types::Empty => sync_apis,
            hand get_walk_route_data(pbjson_types::Empty)
                -> permission_v1::ListApiResponse => walk_route_data,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `CategoryServiceHandlers`.
    proto::gen_admin::services::CategoryServiceHandlers for CategoryProxy {
        client: content_v1::category_service_client::CategoryServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListCategoryResponse,
            pass get(content_v1::GetCategoryRequest) -> content_v1::Category,
            pass create(content_v1::CreateCategoryRequest) -> content_v1::Category,
            pass update(content_v1::UpdateCategoryRequest) -> content_v1::Category,
            pass delete(content_v1::DeleteCategoryRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `CommentServiceHandlers`.
    proto::gen_admin::services::CommentServiceHandlers for CommentProxy {
        client: comment_v1::comment_service_client::CommentServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> comment_v1::ListCommentResponse,
            pass get(comment_v1::GetCommentRequest) -> comment_v1::Comment,
            pass create(comment_v1::CreateCommentRequest) -> comment_v1::Comment,
            pass update(comment_v1::UpdateCommentRequest) -> comment_v1::Comment,
            pass delete(comment_v1::DeleteCommentRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `ContentModelServiceHandlers`.
    proto::gen_admin::services::ContentModelServiceHandlers for ContentModelProxy {
        client: content_v1::content_model_service_client::ContentModelServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListContentModelResponse,
            pass get(content_v1::GetContentModelRequest) -> content_v1::ContentModel,
            pass create(content_v1::CreateContentModelRequest) -> content_v1::ContentModel,
            pass update(content_v1::UpdateContentModelRequest) -> content_v1::ContentModel,
            pass delete(content_v1::DeleteContentModelRequest) -> pbjson_types::Empty,
            pass list_field_definitions(content_v1::ListFieldDefinitionsRequest)
                -> content_v1::ListFieldDefinitionsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `DataAccessAuditLogServiceHandlers`.
    proto::gen_admin::services::DataAccessAuditLogServiceHandlers for DataAccessAuditLogProxy {
        client: audit_v1::data_access_audit_log_service_client::DataAccessAuditLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> audit_v1::ListDataAccessAuditLogResponse,
            pass get(audit_v1::GetDataAccessAuditLogRequest) -> audit_v1::DataAccessAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `DictEntryServiceHandlers`.
    proto::gen_admin::services::DictEntryServiceHandlers for DictEntryProxy {
        client: dict_v1::dict_entry_service_client::DictEntryServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> dict_v1::ListDictEntryResponse,
            pass create(dict_v1::CreateDictEntryRequest) -> pbjson_types::Empty,
            pass update(dict_v1::UpdateDictEntryRequest) -> pbjson_types::Empty,
            pass delete(dict_v1::DeleteDictEntryRequest) -> pbjson_types::Empty,
            pass list_by_type_code(dict_v1::ListDictEntryByTypeCodeRequest)
                -> dict_v1::ListDictEntryByTypeCodeResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `DictTypeServiceHandlers`.
    proto::gen_admin::services::DictTypeServiceHandlers for DictTypeProxy {
        client: dict_v1::dict_type_service_client::DictTypeServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> dict_v1::ListDictTypeResponse,
            pass get(dict_v1::GetDictTypeRequest) -> dict_v1::DictType,
            pass create(dict_v1::CreateDictTypeRequest) -> pbjson_types::Empty,
            pass update(dict_v1::UpdateDictTypeRequest) -> pbjson_types::Empty,
            pass delete(dict_v1::DeleteDictTypeRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `FileServiceHandlers`.
    proto::gen_admin::services::FileServiceHandlers for FileProxy {
        client: storage_v1::file_service_client::FileServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> storage_v1::ListFileResponse,
            pass get(storage_v1::GetFileRequest) -> storage_v1::File,
            hand create(storage_v1::CreateFileRequest) -> pbjson_types::Empty => file_create,
            pass update(storage_v1::UpdateFileRequest) -> pbjson_types::Empty,
            pass delete(storage_v1::DeleteFileRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InteractionAdminServiceHandlers`.
    proto::gen_admin::services::InteractionAdminServiceHandlers for InteractionAdminProxy {
        client: interaction_v1::interaction_admin_service_client::InteractionAdminServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass purge_target_interactions(interaction_v1::PurgeTargetInteractionsRequest)
                -> interaction_v1::PurgeTargetInteractionsResponse,
            pass purge_user_interactions(interaction_v1::PurgeUserInteractionsRequest)
                -> interaction_v1::PurgeUserInteractionsResponse,
            pass reset_counter(interaction_v1::ResetCounterRequest)
                -> interaction_v1::ResetCounterResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InternalMessageCategoryServiceHandlers`.
    proto::gen_admin::services::InternalMessageCategoryServiceHandlers
        for InternalMessageCategoryProxy {
        client: internal_message_v1::internal_message_category_service_client::
            InternalMessageCategoryServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest)
                -> internal_message_v1::ListInternalMessageCategoryResponse,
            pass get(internal_message_v1::GetInternalMessageCategoryRequest)
                -> internal_message_v1::InternalMessageCategory,
            pass create(internal_message_v1::CreateInternalMessageCategoryRequest)
                -> pbjson_types::Empty,
            pass update(internal_message_v1::UpdateInternalMessageCategoryRequest)
                -> pbjson_types::Empty,
            pass delete(internal_message_v1::DeleteInternalMessageCategoryRequest)
                -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InternalMessageRecipientServiceHandlers`.
    proto::gen_admin::services::InternalMessageRecipientServiceHandlers
        for InternalMessageRecipientProxy {
        client: internal_message_v1::internal_message_recipient_service_client::
            InternalMessageRecipientServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list_user_inbox(pagination::PagingRequest)
                -> internal_message_v1::ListUserInboxResponse,
            pass delete_notification_from_inbox(
                internal_message_v1::DeleteNotificationFromInboxRequest
            ) -> pbjson_types::Empty,
            pass mark_notification_as_read(internal_message_v1::MarkNotificationAsReadRequest)
                -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InternalMessageServiceHandlers`.
    proto::gen_admin::services::InternalMessageServiceHandlers for InternalMessageProxy {
        client: internal_message_v1::internal_message_service_client::InternalMessageServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list_message(pagination::PagingRequest)
                -> internal_message_v1::ListInternalMessageResponse,
            pass get_message(internal_message_v1::GetInternalMessageRequest)
                -> internal_message_v1::InternalMessage,
            pass update_message(internal_message_v1::UpdateInternalMessageRequest)
                -> pbjson_types::Empty,
            pass delete_message(internal_message_v1::DeleteInternalMessageRequest)
                -> pbjson_types::Empty,
            pass send_message(internal_message_v1::SendMessageRequest)
                -> internal_message_v1::SendMessageResponse,
            pass revoke_message(internal_message_v1::RevokeMessageRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `LanguageServiceHandlers`.
    proto::gen_admin::services::LanguageServiceHandlers for LanguageProxy {
        client: dict_v1::language_service_client::LanguageServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> dict_v1::ListLanguageResponse,
            pass get(dict_v1::GetLanguageRequest) -> dict_v1::Language,
            pass create(dict_v1::CreateLanguageRequest) -> pbjson_types::Empty,
            pass update(dict_v1::UpdateLanguageRequest) -> pbjson_types::Empty,
            pass delete(dict_v1::DeleteLanguageRequest) -> pbjson_types::Empty,
            pass batch_create(dict_v1::BatchCreateLanguagesRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `LoginAuditLogServiceHandlers`.
    proto::gen_admin::services::LoginAuditLogServiceHandlers for LoginAuditLogProxy {
        client: audit_v1::login_audit_log_service_client::LoginAuditLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> audit_v1::ListLoginAuditLogResponse,
            pass get(audit_v1::GetLoginAuditLogRequest) -> audit_v1::LoginAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `LoginPolicyServiceHandlers`.
    proto::gen_admin::services::LoginPolicyServiceHandlers for LoginPolicyProxy {
        client: authentication_v1::login_policy_service_client::LoginPolicyServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> authentication_v1::ListLoginPolicyResponse,
            pass get(authentication_v1::GetLoginPolicyRequest) -> authentication_v1::LoginPolicy,
            pass create(authentication_v1::CreateLoginPolicyRequest) -> pbjson_types::Empty,
            pass update(authentication_v1::UpdateLoginPolicyRequest) -> pbjson_types::Empty,
            pass delete(authentication_v1::DeleteLoginPolicyRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `MediaAssetServiceHandlers`.
    proto::gen_admin::services::MediaAssetServiceHandlers for MediaAssetProxy {
        client: media_v1::media_asset_service_client::MediaAssetServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> media_v1::ListMediaAssetResponse,
            pass get(media_v1::GetMediaAssetRequest) -> media_v1::MediaAsset,
            pass create(media_v1::CreateMediaAssetRequest) -> media_v1::MediaAsset,
            pass update(media_v1::UpdateMediaAssetRequest) -> media_v1::MediaAsset,
            pass delete(media_v1::DeleteMediaAssetRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `MenuServiceHandlers`.
    proto::gen_admin::services::MenuServiceHandlers for MenuProxy {
        client: permission_v1::menu_service_client::MenuServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListMenuResponse,
            pass get(permission_v1::GetMenuRequest) -> permission_v1::Menu,
            pass create(permission_v1::CreateMenuRequest) -> pbjson_types::Empty,
            pass update(permission_v1::UpdateMenuRequest) -> pbjson_types::Empty,
            pass delete(permission_v1::DeleteMenuRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NavigationItemServiceHandlers`.
    proto::gen_admin::services::NavigationItemServiceHandlers for NavigationItemProxy {
        client: site_v1::navigation_item_service_client::NavigationItemServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> site_v1::ListNavigationItemResponse,
            pass get(site_v1::GetNavigationItemRequest) -> site_v1::NavigationItem,
            pass create(site_v1::CreateNavigationItemRequest) -> site_v1::NavigationItem,
            pass update(site_v1::UpdateNavigationItemRequest) -> site_v1::NavigationItem,
            pass delete(site_v1::DeleteNavigationItemRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NavigationServiceHandlers`.
    proto::gen_admin::services::NavigationServiceHandlers for NavigationProxy {
        client: site_v1::navigation_service_client::NavigationServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> site_v1::ListNavigationResponse,
            pass get(site_v1::GetNavigationRequest) -> site_v1::Navigation,
            pass create(site_v1::CreateNavigationRequest) -> site_v1::Navigation,
            pass update(site_v1::UpdateNavigationRequest) -> site_v1::Navigation,
            pass delete(site_v1::DeleteNavigationRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `OperationAuditLogServiceHandlers`.
    proto::gen_admin::services::OperationAuditLogServiceHandlers for OperationAuditLogProxy {
        client: audit_v1::operation_audit_log_service_client::OperationAuditLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> audit_v1::ListOperationAuditLogResponse,
            pass get(audit_v1::GetOperationAuditLogRequest) -> audit_v1::OperationAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `OrgUnitServiceHandlers`.
    proto::gen_admin::services::OrgUnitServiceHandlers for OrgUnitProxy {
        client: identity_v1::org_unit_service_client::OrgUnitServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> identity_v1::ListOrgUnitResponse,
            pass get(identity_v1::GetOrgUnitRequest) -> identity_v1::OrgUnit,
            pass create(identity_v1::CreateOrgUnitRequest) -> pbjson_types::Empty,
            pass update(identity_v1::UpdateOrgUnitRequest) -> pbjson_types::Empty,
            pass delete(identity_v1::DeleteOrgUnitRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PageServiceHandlers`.
    proto::gen_admin::services::PageServiceHandlers for PageProxy {
        client: content_v1::page_service_client::PageServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListPageResponse,
            pass get(content_v1::GetPageRequest) -> content_v1::Page,
            pass create(content_v1::CreatePageRequest) -> content_v1::Page,
            pass update(content_v1::UpdatePageRequest) -> content_v1::Page,
            pass delete(content_v1::DeletePageRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PermissionAuditLogServiceHandlers`.
    proto::gen_admin::services::PermissionAuditLogServiceHandlers for PermissionAuditLogProxy {
        client: audit_v1::permission_audit_log_service_client::PermissionAuditLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> audit_v1::ListPermissionAuditLogResponse,
            pass get(audit_v1::GetPermissionAuditLogRequest) -> audit_v1::PermissionAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PermissionGroupServiceHandlers`.
    proto::gen_admin::services::PermissionGroupServiceHandlers for PermissionGroupProxy {
        client: permission_v1::permission_group_service_client::PermissionGroupServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListPermissionGroupResponse,
            pass get(permission_v1::GetPermissionGroupRequest) -> permission_v1::PermissionGroup,
            pass create(permission_v1::CreatePermissionGroupRequest) -> pbjson_types::Empty,
            pass update(permission_v1::UpdatePermissionGroupRequest) -> pbjson_types::Empty,
            pass delete(permission_v1::DeletePermissionGroupRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PermissionServiceHandlers`.
    proto::gen_admin::services::PermissionServiceHandlers for PermissionProxy {
        client: permission_v1::permission_service_client::PermissionServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListPermissionResponse,
            pass get(permission_v1::GetPermissionRequest) -> permission_v1::Permission,
            pass create(permission_v1::CreatePermissionRequest) -> pbjson_types::Empty,
            pass update(permission_v1::UpdatePermissionRequest) -> pbjson_types::Empty,
            pass delete(permission_v1::DeletePermissionRequest) -> pbjson_types::Empty,
            hand sync_permissions(pbjson_types::Empty) -> pbjson_types::Empty => sync_permissions,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PolicyEvaluationLogServiceHandlers`.
    proto::gen_admin::services::PolicyEvaluationLogServiceHandlers for PolicyEvaluationLogProxy {
        client: permission_v1::policy_evaluation_log_service_client::
            PolicyEvaluationLogServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListPolicyEvaluationLogResponse,
            pass get(permission_v1::GetPolicyEvaluationLogRequest)
                -> permission_v1::PolicyEvaluationLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PositionServiceHandlers`.
    proto::gen_admin::services::PositionServiceHandlers for PositionProxy {
        client: identity_v1::position_service_client::PositionServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> identity_v1::ListPositionResponse,
            pass get(identity_v1::GetPositionRequest) -> identity_v1::Position,
            pass create(identity_v1::CreatePositionRequest) -> pbjson_types::Empty,
            pass update(identity_v1::UpdatePositionRequest) -> pbjson_types::Empty,
            pass delete(identity_v1::DeletePositionRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PostServiceHandlers`.
    proto::gen_admin::services::PostServiceHandlers for PostProxy {
        client: content_v1::post_service_client::PostServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListPostResponse,
            pass get(content_v1::GetPostRequest) -> content_v1::Post,
            pass create(content_v1::CreatePostRequest) -> content_v1::Post,
            pass update(content_v1::UpdatePostRequest) -> content_v1::Post,
            pass delete(content_v1::DeletePostRequest) -> pbjson_types::Empty,
            pass translation_exists(content_v1::PostTranslationExistsRequest)
                -> content_v1::PostTranslationExistsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `RoleServiceHandlers`.
    proto::gen_admin::services::RoleServiceHandlers for RoleProxy {
        client: permission_v1::role_service_client::RoleServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListRoleResponse,
            pass get(permission_v1::GetRoleRequest) -> permission_v1::Role,
            pass create(permission_v1::CreateRoleRequest) -> pbjson_types::Empty,
            pass update(permission_v1::UpdateRoleRequest) -> pbjson_types::Empty,
            pass delete(permission_v1::DeleteRoleRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `SiteServiceHandlers`.
    proto::gen_admin::services::SiteServiceHandlers for SiteProxy {
        client: site_v1::site_service_client::SiteServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> site_v1::ListSiteResponse,
            pass get(site_v1::GetSiteRequest) -> site_v1::Site,
            pass create(site_v1::CreateSiteRequest) -> site_v1::Site,
            drop update(site_v1::UpdateSiteRequest) -> site_v1::Site,
            pass delete(site_v1::DeleteSiteRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `SiteSettingServiceHandlers`.
    proto::gen_admin::services::SiteSettingServiceHandlers for SiteSettingProxy {
        client: site_v1::site_setting_service_client::SiteSettingServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> site_v1::ListSiteSettingResponse,
            pass get(site_v1::GetSiteSettingRequest) -> site_v1::SiteSetting,
            pass create(site_v1::CreateSiteSettingRequest) -> site_v1::SiteSetting,
            pass update(site_v1::UpdateSiteSettingRequest) -> site_v1::SiteSetting,
            pass delete(site_v1::DeleteSiteSettingRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `StatsServiceHandlers`.
    proto::gen_admin::services::StatsServiceHandlers for StatsProxy {
        client: stats_v1::stats_service_client::StatsServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass get_dashboard_overview(stats_v1::GetDashboardOverviewRequest)
                -> stats_v1::GetDashboardOverviewResponse,
            pass get_content_trend(stats_v1::GetContentTrendRequest)
                -> stats_v1::GetContentTrendResponse,
            pass get_interaction_stats(stats_v1::GetInteractionStatsRequest)
                -> stats_v1::GetInteractionStatsResponse,
            pass get_login_activity(stats_v1::GetLoginActivityRequest)
                -> stats_v1::GetLoginActivityResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TagServiceHandlers`.
    proto::gen_admin::services::TagServiceHandlers for TagProxy {
        client: content_v1::tag_service_client::TagServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListTagResponse,
            pass get(content_v1::GetTagRequest) -> content_v1::Tag,
            pass create(content_v1::CreateTagRequest) -> content_v1::Tag,
            pass update(content_v1::UpdateTagRequest) -> content_v1::Tag,
            pass delete(content_v1::DeleteTagRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TaskServiceHandlers`.
    proto::gen_admin::services::TaskServiceHandlers for TaskProxy {
        client: task_v1::task_service_client::TaskServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> task_v1::ListTaskResponse,
            pass get(task_v1::GetTaskRequest) -> task_v1::Task,
            pass create(task_v1::CreateTaskRequest) -> pbjson_types::Empty,
            pass update(task_v1::UpdateTaskRequest) -> pbjson_types::Empty,
            pass delete(task_v1::DeleteTaskRequest) -> pbjson_types::Empty,
            pass list_task_type_name(pbjson_types::Empty) -> task_v1::ListTaskTypeNameResponse,
            pass restart_all_task(pbjson_types::Empty) -> task_v1::RestartAllTaskResponse,
            pass start_all_task(pbjson_types::Empty) -> pbjson_types::Empty,
            pass stop_all_task(pbjson_types::Empty) -> pbjson_types::Empty,
            pass control_task(task_v1::ControlTaskRequest) -> pbjson_types::Empty,
            pass list_task_executions(task_v1::ListTaskExecutionsRequest)
                -> task_v1::ListTaskExecutionsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TenantServiceHandlers`.
    proto::gen_admin::services::TenantServiceHandlers for TenantProxy {
        client: identity_v1::tenant_service_client::TenantServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> identity_v1::ListTenantResponse,
            pass get(identity_v1::GetTenantRequest) -> identity_v1::Tenant,
            drop create(identity_v1::CreateTenantRequest) -> pbjson_types::Empty,
            pass update(identity_v1::UpdateTenantRequest) -> pbjson_types::Empty,
            pass delete(identity_v1::DeleteTenantRequest) -> pbjson_types::Empty,
            pass create_tenant_with_admin_user(identity_v1::CreateTenantWithAdminUserRequest)
                -> pbjson_types::Empty,
            pass tenant_exists(identity_v1::TenantExistsRequest)
                -> identity_v1::TenantExistsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TranslatorServiceHandlers`.
    proto::gen_admin::services::TranslatorServiceHandlers for TranslatorProxy {
        client: translator_v1::translator_service_client::TranslatorServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass translate(translator_v1::TranslateRequest) -> translator_v1::TranslateResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `UserProfileServiceHandlers`.
    proto::gen_admin::services::UserProfileServiceHandlers for UserProfileProxy {
        client: identity_v1::user_profile_service_client::UserProfileServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass get_user(pbjson_types::Empty) -> identity_v1::User,
            pass update_user(identity_v1::UpdateUserRequest) -> pbjson_types::Empty,
            hand change_password(identity_v1::ChangePasswordRequest)
                -> pbjson_types::Empty => change_password,
            hand bind_contact(identity_v1::BindContactRequest)
                -> pbjson_types::Empty => bind_contact,
            hand verify_contact(identity_v1::VerifyContactRequest)
                -> pbjson_types::Empty => verify_contact,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `UserServiceHandlers`.
    proto::gen_admin::services::UserServiceHandlers for UserProxy {
        client: identity_v1::user_service_client::UserServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> identity_v1::ListUserResponse,
            pass get(identity_v1::GetUserRequest) -> identity_v1::User,
            pass create(identity_v1::CreateUserRequest) -> identity_v1::User,
            pass update(identity_v1::UpdateUserRequest) -> pbjson_types::Empty,
            pass delete(identity_v1::DeleteUserRequest) -> pbjson_types::Empty,
            pass user_exists(identity_v1::UserExistsRequest) -> identity_v1::UserExistsResponse,
            hand edit_user_password(identity_v1::EditUserPasswordRequest)
                -> pbjson_types::Empty => edit_user_password,
        ]
    }
}
