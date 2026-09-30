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
use proto::proto::audit::service::v1::{
    api_audit_log_service_server::ApiAuditLogServiceServer,
    data_access_audit_log_service_server::DataAccessAuditLogServiceServer,
    login_audit_log_service_server::LoginAuditLogServiceServer,
    operation_audit_log_service_server::OperationAuditLogServiceServer,
    permission_audit_log_service_server::PermissionAuditLogServiceServer,
};
use proto::proto::authentication::service::v1::{
    authentication_service_server::AuthenticationServiceServer,
    login_policy_service_server::LoginPolicyServiceServer,
    user_credential_service_server::UserCredentialServiceServer,
};
use proto::proto::comment::service::v1::comment_service_server::CommentServiceServer;
use proto::proto::content::service::v1::{
    category_service_server::CategoryServiceServer,
    content_model_service_server::ContentModelServiceServer,
    page_service_server::PageServiceServer, post_service_server::PostServiceServer,
    tag_service_server::TagServiceServer,
};
use proto::proto::dict::service::v1::{
    dict_entry_service_server::DictEntryServiceServer,
    dict_type_service_server::DictTypeServiceServer,
    language_service_server::LanguageServiceServer,
};
use proto::proto::identity::service::v1::{
    org_unit_service_server::OrgUnitServiceServer, position_service_server::PositionServiceServer,
    tenant_service_server::TenantServiceServer,
    user_profile_service_server::UserProfileServiceServer, user_service_server::UserServiceServer,
};
use proto::proto::interaction::service::v1::{
    interaction_admin_service_server::InteractionAdminServiceServer,
    interaction_service_server::InteractionServiceServer,
};
use proto::proto::internal_message::service::v1::{
    internal_message_category_service_server::InternalMessageCategoryServiceServer,
    internal_message_recipient_service_server::InternalMessageRecipientServiceServer,
    internal_message_service_server::InternalMessageServiceServer,
};
use proto::proto::media::service::v1::media_asset_service_server::MediaAssetServiceServer;
use proto::proto::permission::service::v1::{
    api_service_server::ApiServiceServer, menu_service_server::MenuServiceServer,
    permission_group_service_server::PermissionGroupServiceServer,
    permission_service_server::PermissionServiceServer,
    policy_evaluation_log_service_server::PolicyEvaluationLogServiceServer,
    role_service_server::RoleServiceServer,
};
use proto::proto::site::service::v1::{
    navigation_item_service_server::NavigationItemServiceServer,
    navigation_service_server::NavigationServiceServer, site_service_server::SiteServiceServer,
    site_setting_service_server::SiteSettingServiceServer,
};
use proto::proto::stats::service::v1::stats_service_server::StatsServiceServer;
use proto::proto::storage::service::v1::{
    file_service_server::FileServiceServer, file_transfer_service_server::FileTransferServiceServer,
};
use proto::proto::task::service::v1::task_service_server::TaskServiceServer;
use proto::proto::translator::service::v1::translator_service_server::TranslatorServiceServer;

/// Assembles the tonic service registry — one added-server per domain
/// service, mirroring the reference's NewGrpcServer registration block.
pub fn registry(state: Arc<AppState>) -> tonic::service::Routes {
    let auth = authentication::AuthenticationServiceImpl {
        state: Arc::clone(&state),
    };
    tonic::service::Routes::new(AuthenticationServiceServer::new(auth))
        .add_service(LanguageServiceServer::new(
            service::language_service::LanguageService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(DictTypeServiceServer::new(
            service::dict_type_service::DictTypeService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(DictEntryServiceServer::new(
            service::dict_entry_service::DictEntryService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(PostServiceServer::new(service::post_service::PostService {
            state: Arc::clone(&state),
        }))
        .add_service(CategoryServiceServer::new(
            service::category_service::CategoryService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(TagServiceServer::new(service::tag_service::TagService {
            state: Arc::clone(&state),
        }))
        .add_service(PageServiceServer::new(service::page_service::PageService {
            state: Arc::clone(&state),
        }))
        .add_service(SiteServiceServer::new(service::site_service::SiteService {
            state: Arc::clone(&state),
        }))
        .add_service(SiteSettingServiceServer::new(
            service::site_setting_service::SiteSettingService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(NavigationServiceServer::new(
            service::navigation_service::NavigationService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(NavigationItemServiceServer::new(
            service::navigation_item_service::NavigationItemService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(CommentServiceServer::new(
            service::comment_service::CommentService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(InteractionServiceServer::new(
            service::interaction_service::InteractionService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(InteractionAdminServiceServer::new(
            service::interaction_admin_service::InteractionAdminService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(UserServiceServer::new(service::user_service::UserService {
            state: Arc::clone(&state),
        }))
        .add_service(TenantServiceServer::new(
            service::tenant_service::TenantService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(UserProfileServiceServer::new(
            service::user_profile_service::UserProfileService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(TranslatorServiceServer::new(
            service::translator_service::TranslatorService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(FileServiceServer::new(service::file_service::FileService {
            state: Arc::clone(&state),
        }))
        .add_service(FileTransferServiceServer::new(
            service::file_service::FileService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(RoleServiceServer::new(service::role_service::RoleService {
            state: Arc::clone(&state),
        }))
        .add_service(MenuServiceServer::new(service::menu_service::MenuService {
            state: Arc::clone(&state),
        }))
        .add_service(ApiServiceServer::new(service::api_service::ApiService {
            state: Arc::clone(&state),
        }))
        .add_service(PermissionGroupServiceServer::new(
            service::permission_group_service::PermissionGroupService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(PermissionServiceServer::new(
            service::permission_service::PermissionService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(ApiAuditLogServiceServer::new(
            service::api_audit_log_service::ApiAuditLogService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(LoginAuditLogServiceServer::new(
            service::login_audit_log_service::LoginAuditLogService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(OperationAuditLogServiceServer::new(
            service::operation_audit_log_service::OperationAuditLogService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(DataAccessAuditLogServiceServer::new(
            service::data_access_audit_log_service::DataAccessAuditLogService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(PermissionAuditLogServiceServer::new(
            service::permission_audit_log_service::PermissionAuditLogService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(StatsServiceServer::new(
            service::stats_service::StatsService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(OrgUnitServiceServer::new(
            service::org_unit_service::OrgUnitService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(PositionServiceServer::new(
            service::position_service::PositionService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(LoginPolicyServiceServer::new(
            service::login_policy_service::LoginPolicyService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(ContentModelServiceServer::new(
            service::content_model_service::ContentModelService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(MediaAssetServiceServer::new(
            service::media_asset_service::MediaAssetService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(TaskServiceServer::new(service::task_service::TaskService {
            state: Arc::clone(&state),
        }))
        .add_service(InternalMessageServiceServer::new(
            service::internal_message_service::InternalMessageService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(InternalMessageCategoryServiceServer::new(
            service::internal_message_category_service::InternalMessageCategoryService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(InternalMessageRecipientServiceServer::new(
            service::internal_message_recipient_service::InternalMessageRecipientService {
                state: Arc::clone(&state),
            },
        ))
        .add_service(UserCredentialServiceServer::new(
            service::user_credential_service::UserCredentialServiceImpl {
                state: Arc::clone(&state),
            },
        ))
        .add_service(PolicyEvaluationLogServiceServer::new(
            service::policy_evaluation_log_service::PolicyEvaluationLogServiceImpl {
                state: Arc::clone(&state),
            },
        ))
}
