//! The domain gRPC service implementations. A service absent from the
//! served list (or a method left to its generated default) answers
//! Unimplemented — the reference's servers register every module the
//! same way and land their logic module by module.

pub mod authentication;
pub mod content;
pub mod dict;
pub mod identity;
pub mod site;
pub mod social;

use std::sync::Arc;

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
            dict::LanguageServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::dict::service::v1::dict_type_service_server::DictTypeServiceServer::new(
            dict::DictTypeServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::dict::service::v1::dict_entry_service_server::DictEntryServiceServer::new(
            dict::DictEntryServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::post_service_server::PostServiceServer::new(
            content::PostServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::category_service_server::CategoryServiceServer::new(
            content::CategoryServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::tag_service_server::TagServiceServer::new(
            content::TagServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::content::service::v1::page_service_server::PageServiceServer::new(
            content::PageServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::site_service_server::SiteServiceServer::new(
            site::SiteServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::site_setting_service_server::SiteSettingServiceServer::new(
            site::SiteSettingServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::navigation_service_server::NavigationServiceServer::new(
            site::NavigationServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::site::service::v1::navigation_item_service_server::NavigationItemServiceServer::new(
            site::NavigationItemServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::comment::service::v1::comment_service_server::CommentServiceServer::new(
            social::CommentServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::interaction::service::v1::interaction_service_server::InteractionServiceServer::new(
            social::InteractionServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::interaction::service::v1::interaction_admin_service_server::InteractionAdminServiceServer::new(
            social::InteractionAdminServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::user_service_server::UserServiceServer::new(
            identity::UserServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::identity::service::v1::tenant_service_server::TenantServiceServer::new(
            identity::TenantServiceImpl { state: Arc::clone(&state) },
        ),
    )
    .add_service(
        proto::proto::permission::service::v1::role_service_server::RoleServiceServer::new(
            identity::RoleServiceImpl { state: Arc::clone(&state) },
        ),
    )
}
