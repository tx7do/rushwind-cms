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
use proto::proto::comment::service::v1 as comment_v1;
use proto::proto::content::service::v1 as content_v1;
use proto::proto::identity::service::v1 as identity_v1;
use proto::proto::interaction::service::v1 as interaction_v1;
use proto::proto::pagination;
use proto::proto::site::service::v1 as site_v1;

passthrough_proxy! {
    /// The pass-through proxy of `CategoryServiceHandlers`.
    proto::gen_app::services::CategoryServiceHandlers for CategoryProxy {
        client: content_v1::category_service_client::CategoryServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: public,
        methods: [
            hand list(pagination::PagingRequest)
                -> content_v1::ListCategoryResponse => category_list,
            hand get(content_v1::GetCategoryRequest) -> content_v1::Category => category_get,
            hand create(content_v1::CreateCategoryRequest)
                -> content_v1::Category => forbidden_mutation,
            hand update(content_v1::UpdateCategoryRequest)
                -> content_v1::Category => forbidden_mutation,
            hand delete(content_v1::DeleteCategoryRequest)
                -> pbjson_types::Empty => forbidden_mutation,
            pass get_translation(content_v1::GetCategoryRequest) -> content_v1::CategoryTranslation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `CommentServiceHandlers`.
    proto::gen_app::services::CommentServiceHandlers for CommentProxy {
        client: comment_v1::comment_service_client::CommentServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            hand list(pagination::PagingRequest) -> comment_v1::ListCommentResponse => comment_list,
            hand get(comment_v1::GetCommentRequest) -> comment_v1::Comment => comment_get,
            hand create(comment_v1::CreateCommentRequest) -> comment_v1::Comment => comment_create,
            hand update(comment_v1::UpdateCommentRequest) -> comment_v1::Comment => comment_update,
            hand delete(comment_v1::DeleteCommentRequest) -> pbjson_types::Empty => comment_delete,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InteractionServiceHandlers`.
    proto::gen_app::services::InteractionServiceHandlers for InteractionProxy {
        client: interaction_v1::interaction_service_client::InteractionServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: public,
        methods: [
            pass like(interaction_v1::LikeRequest) -> interaction_v1::LikeResponse,
            pass unlike(interaction_v1::LikeRequest) -> interaction_v1::LikeResponse,
            pass watch(interaction_v1::WatchRequest) -> interaction_v1::WatchResponse,
            pass unwatch(interaction_v1::WatchRequest) -> interaction_v1::WatchResponse,
            pass get_interaction_status(interaction_v1::GetInteractionStatusRequest)
                -> interaction_v1::GetInteractionStatusResponse,
            pass list_watched_posts(pagination::PagingRequest) -> content_v1::ListPostResponse,
            pass get_counts(interaction_v1::GetCountsRequest) -> interaction_v1::GetCountsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NavigationServiceHandlers`.
    proto::gen_app::services::NavigationServiceHandlers for NavigationProxy {
        client: site_v1::navigation_service_client::NavigationServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: public,
        methods: [
            pass list(pagination::PagingRequest) -> site_v1::ListNavigationResponse,
            pass get(site_v1::GetNavigationRequest) -> site_v1::Navigation,
            hand create(site_v1::CreateNavigationRequest)
                -> site_v1::Navigation => forbidden_mutation,
            hand update(site_v1::UpdateNavigationRequest)
                -> site_v1::Navigation => forbidden_mutation,
            hand delete(site_v1::DeleteNavigationRequest)
                -> pbjson_types::Empty => forbidden_mutation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PageServiceHandlers`.
    proto::gen_app::services::PageServiceHandlers for PageProxy {
        client: content_v1::page_service_client::PageServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            hand list(pagination::PagingRequest) -> content_v1::ListPageResponse => page_list,
            hand get(content_v1::GetPageRequest) -> content_v1::Page => page_get,
            hand create(content_v1::CreatePageRequest) -> content_v1::Page => forbidden_mutation,
            hand update(content_v1::UpdatePageRequest) -> content_v1::Page => forbidden_mutation,
            hand delete(content_v1::DeletePageRequest) -> pbjson_types::Empty => forbidden_mutation,
            pass get_translation(content_v1::GetPageRequest) -> content_v1::PageTranslation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PostServiceHandlers`.
    proto::gen_app::services::PostServiceHandlers for PostProxy {
        client: content_v1::post_service_client::PostServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            hand list(pagination::PagingRequest) -> content_v1::ListPostResponse => post_list,
            hand search_posts(content_v1::SearchPostsRequest)
                -> content_v1::SearchPostsResponse => post_search,
            hand get(content_v1::GetPostRequest) -> content_v1::Post => post_get,
            hand create(content_v1::CreatePostRequest) -> content_v1::Post => forbidden_mutation,
            hand update(content_v1::UpdatePostRequest) -> content_v1::Post => forbidden_mutation,
            hand delete(content_v1::DeletePostRequest) -> pbjson_types::Empty => forbidden_mutation,
            pass get_translation(content_v1::GetPostRequest) -> content_v1::PostTranslation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `SiteServiceHandlers`.
    proto::gen_app::services::SiteServiceHandlers for SiteProxy {
        client: site_v1::site_service_client::SiteServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            hand list(pagination::PagingRequest) -> site_v1::ListSiteResponse => forbidden_mutation,
            hand get_site_by_domain(site_v1::GetSiteByDomainRequest)
                -> site_v1::Site => site_by_domain,
            hand create(site_v1::CreateSiteRequest) -> site_v1::Site => forbidden_mutation,
            hand update(site_v1::UpdateSiteRequest) -> site_v1::Site => forbidden_mutation,
            hand delete(site_v1::DeleteSiteRequest) -> pbjson_types::Empty => forbidden_mutation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TagServiceHandlers`.
    proto::gen_app::services::TagServiceHandlers for TagProxy {
        client: content_v1::tag_service_client::TagServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListTagResponse,
            pass get(content_v1::GetTagRequest) -> content_v1::Tag,
            hand create(content_v1::CreateTagRequest) -> content_v1::Tag => forbidden_mutation,
            hand update(content_v1::UpdateTagRequest) -> content_v1::Tag => forbidden_mutation,
            hand delete(content_v1::DeleteTagRequest) -> pbjson_types::Empty => forbidden_mutation,
            pass get_translation(content_v1::GetTagRequest) -> content_v1::TagTranslation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `UserProfileServiceHandlers`.
    proto::gen_app::services::UserProfileServiceHandlers for UserProfileProxy {
        client: identity_v1::user_profile_service_client::UserProfileServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: public,
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
