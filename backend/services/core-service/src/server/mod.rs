//! The domain gRPC service implementations. A service absent from the
//! served list (or a method left to its generated default) answers
//! Unimplemented — the reference's servers register every module the
//! same way and land their logic module by module.

//! The server layer — the gRPC face assembly (the tonic registry),
//! mirroring the reference's `internal/server`. The service modules
//! live in [`crate::service`]; this module owns registration only.

use std::sync::Arc;

use crate::service::{self, authentication_service as authentication};
use crate::state::AppState;

/// Assembles the tonic service registry — one added-server per domain
/// service, mirroring the reference's NewGrpcServer registration block.
pub fn registry(state: Arc<AppState>) -> tonic::service::Routes {
    let auth = authentication::AuthenticationServiceImpl {
        state: Arc::clone(&state),
    };
    tonic::service::Routes::new(
        proto::proto::authentication::service::v1::authentication_service_server::AuthenticationServiceServer::new(
            auth,
        ),
    )
    .add_service(
        proto::proto::dict::service::v1::language_service_server::LanguageServiceServer::new(
            service::language_service::LanguageService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::dict::service::v1::dict_type_service_server::DictTypeServiceServer::new(
            service::dict_type_service::DictTypeService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::dict::service::v1::dict_entry_service_server::DictEntryServiceServer::new(
            service::dict_entry_service::DictEntryService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::post_service_server::PostServiceServer::new(
            service::post_service::PostService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::category_service_server::CategoryServiceServer::new(
            service::category_service::CategoryService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::tag_service_server::TagServiceServer::new(
            service::tag_service::TagService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::page_service_server::PageServiceServer::new(
            service::page_service::PageService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::site_service_server::SiteServiceServer::new(
            service::site_service::SiteService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::site_setting_service_server::SiteSettingServiceServer::new(
            service::site_setting_service::SiteSettingService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::navigation_service_server::NavigationServiceServer::new(
            service::navigation_service::NavigationService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::navigation_item_service_server::NavigationItemServiceServer::new(
            service::navigation_item_service::NavigationItemService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::comment::service::v1::comment_service_server::CommentServiceServer::new(
            service::comment_service::CommentService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::interaction::service::v1::interaction_service_server::InteractionServiceServer::new(
            service::interaction_service::InteractionService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::interaction::service::v1::interaction_admin_service_server::InteractionAdminServiceServer::new(
            service::interaction_admin_service::InteractionAdminService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::user_service_server::UserServiceServer::new(
            service::user_service::UserService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::tenant_service_server::TenantServiceServer::new(
            service::tenant_service::TenantService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::user_profile_service_server::UserProfileServiceServer::new(
            service::user_profile_service::UserProfileService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::translator::service::v1::translator_service_server::TranslatorServiceServer::new(
            service::translator_service::TranslatorService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::storage::service::v1::file_service_server::FileServiceServer::new(
            service::file_service::FileService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::storage::service::v1::file_transfer_service_server::FileTransferServiceServer::new(
            service::file_service::FileService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::role_service_server::RoleServiceServer::new(
            service::role_service::RoleService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::menu_service_server::MenuServiceServer::new(
            service::menu_service::MenuService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::api_service_server::ApiServiceServer::new(
            service::api_service::ApiService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::permission_group_service_server::PermissionGroupServiceServer::new(
            service::permission_group_service::PermissionGroupService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::permission_service_server::PermissionServiceServer::new(
            service::permission_service::PermissionService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::audit::service::v1::api_audit_log_service_server::ApiAuditLogServiceServer::new(
            service::api_audit_log_service::ApiAuditLogService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::audit::service::v1::login_audit_log_service_server::LoginAuditLogServiceServer::new(
            service::login_audit_log_service::LoginAuditLogService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::audit::service::v1::operation_audit_log_service_server::OperationAuditLogServiceServer::new(
            service::operation_audit_log_service::OperationAuditLogService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::audit::service::v1::data_access_audit_log_service_server::DataAccessAuditLogServiceServer::new(
            service::data_access_audit_log_service::DataAccessAuditLogService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::audit::service::v1::permission_audit_log_service_server::PermissionAuditLogServiceServer::new(
            service::permission_audit_log_service::PermissionAuditLogService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::stats::service::v1::stats_service_server::StatsServiceServer::new(
            service::stats_service::StatsService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::org_unit_service_server::OrgUnitServiceServer::new(
            service::org_unit_service::OrgUnitService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::position_service_server::PositionServiceServer::new(
            service::position_service::PositionService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::authentication::service::v1::login_policy_service_server::LoginPolicyServiceServer::new(
            service::login_policy_service::LoginPolicyService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::content_model_service_server::ContentModelServiceServer::new(
            service::content_model_service::ContentModelService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::media::service::v1::media_asset_service_server::MediaAssetServiceServer::new(
            service::media_asset_service::MediaAssetService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::task::service::v1::task_service_server::TaskServiceServer::new(
            service::task_service::TaskService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::internal_message::service::v1::internal_message_service_server::InternalMessageServiceServer::new(
            service::internal_message_service::InternalMessageService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::internal_message::service::v1::internal_message_category_service_server::InternalMessageCategoryServiceServer::new(
            service::internal_message_category_service::InternalMessageCategoryService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::internal_message::service::v1::internal_message_recipient_service_server::InternalMessageRecipientServiceServer::new(
            service::internal_message_recipient_service::InternalMessageRecipientService { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::authentication::service::v1::user_credential_service_server::UserCredentialServiceServer::new(
            service::user_credential_service::UserCredentialServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::policy_evaluation_log_service_server::PolicyEvaluationLogServiceServer::new(
            service::policy_evaluation_log_service::PolicyEvaluationLogServiceImpl { state: Arc::clone(&state) },
        ),
    )
}
