//! The domain gRPC service implementations. A service absent from the
//! served list (or a method left to its generated default) answers
//! Unimplemented — the reference's servers register every module the
//! same way and land their logic module by module.

//! The server layer — the gRPC face assembly (the tonic registry),
//! mirroring the reference's `internal/server`. The service modules
//! live in [`crate::service`]; this module owns registration only.

use std::sync::Arc;

use crate::service::{self, authentication};
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
            service::dict::LanguageServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::dict::service::v1::dict_type_service_server::DictTypeServiceServer::new(
            service::dict::DictTypeServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::dict::service::v1::dict_entry_service_server::DictEntryServiceServer::new(
            service::dict::DictEntryServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::post_service_server::PostServiceServer::new(
            service::content::PostServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::category_service_server::CategoryServiceServer::new(
            service::content::CategoryServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::tag_service_server::TagServiceServer::new(
            service::content::TagServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::page_service_server::PageServiceServer::new(
            service::content::PageServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::site_service_server::SiteServiceServer::new(
            service::site::SiteServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::site_setting_service_server::SiteSettingServiceServer::new(
            service::site::SiteSettingServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::navigation_service_server::NavigationServiceServer::new(
            service::site::NavigationServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::navigation_item_service_server::NavigationItemServiceServer::new(
            service::site::NavigationItemServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::comment::service::v1::comment_service_server::CommentServiceServer::new(
            service::social::CommentServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::interaction::service::v1::interaction_service_server::InteractionServiceServer::new(
            service::social::InteractionServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::interaction::service::v1::interaction_admin_service_server::InteractionAdminServiceServer::new(
            service::social::InteractionAdminServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::user_service_server::UserServiceServer::new(
            service::writes::UserWriteServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::tenant_service_server::TenantServiceServer::new(
            service::writes::TenantWriteServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::user_profile_service_server::UserProfileServiceServer::new(
            service::writes::UserProfileServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::translator::service::v1::translator_service_server::TranslatorServiceServer::new(
            service::writes::TranslatorServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::storage::service::v1::file_service_server::FileServiceServer::new(
            service::storage::FileServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::storage::service::v1::file_transfer_service_server::FileTransferServiceServer::new(
            service::storage::FileServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::role_service_server::RoleServiceServer::new(
            service::writes::RoleWriteServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::menu_service_server::MenuServiceServer::new(
            service::permission::MenuServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::api_service_server::ApiServiceServer::new(
            service::permission::ApiSyncServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::permission_group_service_server::PermissionGroupServiceServer::new(
            service::permission::PermissionGroupServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::permission_service_server::PermissionServiceServer::new(
            service::permission::PermissionServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::audit::service::v1::api_audit_log_service_server::ApiAuditLogServiceServer::new(
            service::audit::ApiAuditLogServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::audit::service::v1::login_audit_log_service_server::LoginAuditLogServiceServer::new(
            service::audit::LoginAuditLogServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::audit::service::v1::operation_audit_log_service_server::OperationAuditLogServiceServer::new(
            service::audit::OperationAuditLogServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::audit::service::v1::data_access_audit_log_service_server::DataAccessAuditLogServiceServer::new(
            service::audit::DataAccessAuditLogServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::audit::service::v1::permission_audit_log_service_server::PermissionAuditLogServiceServer::new(
            service::audit::PermissionAuditLogServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::stats::service::v1::stats_service_server::StatsServiceServer::new(
            service::stats::StatsServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::org_unit_service_server::OrgUnitServiceServer::new(
            service::misc::OrgUnitServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::position_service_server::PositionServiceServer::new(
            service::misc::PositionServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::authentication::service::v1::login_policy_service_server::LoginPolicyServiceServer::new(
            service::misc::LoginPolicyServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::content_model_service_server::ContentModelServiceServer::new(
            service::misc::ContentModelServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::media::service::v1::media_asset_service_server::MediaAssetServiceServer::new(
            service::misc::MediaAssetServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::task::service::v1::task_service_server::TaskServiceServer::new(
            service::messaging::TaskControlServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::internal_message::service::v1::internal_message_service_server::InternalMessageServiceServer::new(
            service::messaging::InternalMessageWriteServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::internal_message::service::v1::internal_message_category_service_server::InternalMessageCategoryServiceServer::new(
            service::misc::InternalMessageCategoryServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::internal_message::service::v1::internal_message_recipient_service_server::InternalMessageRecipientServiceServer::new(
            service::messaging::RecipientWriteServiceImpl { state: Arc::clone(&state) },
        ),
    )
}
