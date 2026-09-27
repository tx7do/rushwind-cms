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
    /// The pass-through proxy of `CategoryServiceHandlers`.
    proto::gen_app::services::CategoryServiceHandlers for CategoryProxy {
        client: proto::proto::content::service::v1::category_service_client::CategoryServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            hand list(proto::proto::pagination::PagingRequest) -> proto::proto::content::service::v1::ListCategoryResponse => category_list,
            hand get(proto::proto::content::service::v1::GetCategoryRequest) -> proto::proto::content::service::v1::Category => category_get,
            hand create(proto::proto::content::service::v1::CreateCategoryRequest) -> proto::proto::content::service::v1::Category => forbidden_mutation,
            hand update(proto::proto::content::service::v1::UpdateCategoryRequest) -> proto::proto::content::service::v1::Category => forbidden_mutation,
            hand delete(proto::proto::content::service::v1::DeleteCategoryRequest) -> pbjson_types::Empty => forbidden_mutation,
            pass get_translation(proto::proto::content::service::v1::GetCategoryRequest) -> proto::proto::content::service::v1::CategoryTranslation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `CommentServiceHandlers`.
    proto::gen_app::services::CommentServiceHandlers for CommentProxy {
        client: proto::proto::comment::service::v1::comment_service_client::CommentServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            hand list(proto::proto::pagination::PagingRequest) -> proto::proto::comment::service::v1::ListCommentResponse => comment_list,
            hand get(proto::proto::comment::service::v1::GetCommentRequest) -> proto::proto::comment::service::v1::Comment => comment_get,
            hand create(proto::proto::comment::service::v1::CreateCommentRequest) -> proto::proto::comment::service::v1::Comment => comment_create,
            hand update(proto::proto::comment::service::v1::UpdateCommentRequest) -> proto::proto::comment::service::v1::Comment => comment_update,
            hand delete(proto::proto::comment::service::v1::DeleteCommentRequest) -> pbjson_types::Empty => comment_delete,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InteractionServiceHandlers`.
    proto::gen_app::services::InteractionServiceHandlers for InteractionProxy {
        client: proto::proto::interaction::service::v1::interaction_service_client::InteractionServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            pass like(proto::proto::interaction::service::v1::LikeRequest) -> proto::proto::interaction::service::v1::LikeResponse,
            pass unlike(proto::proto::interaction::service::v1::LikeRequest) -> proto::proto::interaction::service::v1::LikeResponse,
            pass watch(proto::proto::interaction::service::v1::WatchRequest) -> proto::proto::interaction::service::v1::WatchResponse,
            pass unwatch(proto::proto::interaction::service::v1::WatchRequest) -> proto::proto::interaction::service::v1::WatchResponse,
            pass get_interaction_status(proto::proto::interaction::service::v1::GetInteractionStatusRequest) -> proto::proto::interaction::service::v1::GetInteractionStatusResponse,
            pass list_watched_posts(proto::proto::pagination::PagingRequest) -> proto::proto::content::service::v1::ListPostResponse,
            pass get_counts(proto::proto::interaction::service::v1::GetCountsRequest) -> proto::proto::interaction::service::v1::GetCountsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NavigationServiceHandlers`.
    proto::gen_app::services::NavigationServiceHandlers for NavigationProxy {
        client: proto::proto::site::service::v1::navigation_service_client::NavigationServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::site::service::v1::ListNavigationResponse,
            pass get(proto::proto::site::service::v1::GetNavigationRequest) -> proto::proto::site::service::v1::Navigation,
            hand create(proto::proto::site::service::v1::CreateNavigationRequest) -> proto::proto::site::service::v1::Navigation => forbidden_mutation,
            hand update(proto::proto::site::service::v1::UpdateNavigationRequest) -> proto::proto::site::service::v1::Navigation => forbidden_mutation,
            hand delete(proto::proto::site::service::v1::DeleteNavigationRequest) -> pbjson_types::Empty => forbidden_mutation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PageServiceHandlers`.
    proto::gen_app::services::PageServiceHandlers for PageProxy {
        client: proto::proto::content::service::v1::page_service_client::PageServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            hand list(proto::proto::pagination::PagingRequest) -> proto::proto::content::service::v1::ListPageResponse => page_list,
            hand get(proto::proto::content::service::v1::GetPageRequest) -> proto::proto::content::service::v1::Page => page_get,
            hand create(proto::proto::content::service::v1::CreatePageRequest) -> proto::proto::content::service::v1::Page => forbidden_mutation,
            hand update(proto::proto::content::service::v1::UpdatePageRequest) -> proto::proto::content::service::v1::Page => forbidden_mutation,
            hand delete(proto::proto::content::service::v1::DeletePageRequest) -> pbjson_types::Empty => forbidden_mutation,
            pass get_translation(proto::proto::content::service::v1::GetPageRequest) -> proto::proto::content::service::v1::PageTranslation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PostServiceHandlers`.
    proto::gen_app::services::PostServiceHandlers for PostProxy {
        client: proto::proto::content::service::v1::post_service_client::PostServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            hand list(proto::proto::pagination::PagingRequest) -> proto::proto::content::service::v1::ListPostResponse => post_list,
            hand search_posts(proto::proto::content::service::v1::SearchPostsRequest) -> proto::proto::content::service::v1::SearchPostsResponse => post_search,
            hand get(proto::proto::content::service::v1::GetPostRequest) -> proto::proto::content::service::v1::Post => post_get,
            hand create(proto::proto::content::service::v1::CreatePostRequest) -> proto::proto::content::service::v1::Post => forbidden_mutation,
            hand update(proto::proto::content::service::v1::UpdatePostRequest) -> proto::proto::content::service::v1::Post => forbidden_mutation,
            hand delete(proto::proto::content::service::v1::DeletePostRequest) -> pbjson_types::Empty => forbidden_mutation,
            pass get_translation(proto::proto::content::service::v1::GetPostRequest) -> proto::proto::content::service::v1::PostTranslation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `SiteServiceHandlers`.
    proto::gen_app::services::SiteServiceHandlers for SiteProxy {
        client: proto::proto::site::service::v1::site_service_client::SiteServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            hand list(proto::proto::pagination::PagingRequest) -> proto::proto::site::service::v1::ListSiteResponse => forbidden_mutation,
            hand get_site_by_domain(proto::proto::site::service::v1::GetSiteByDomainRequest) -> proto::proto::site::service::v1::Site => site_by_domain,
            hand create(proto::proto::site::service::v1::CreateSiteRequest) -> proto::proto::site::service::v1::Site => forbidden_mutation,
            hand update(proto::proto::site::service::v1::UpdateSiteRequest) -> proto::proto::site::service::v1::Site => forbidden_mutation,
            hand delete(proto::proto::site::service::v1::DeleteSiteRequest) -> pbjson_types::Empty => forbidden_mutation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TagServiceHandlers`.
    proto::gen_app::services::TagServiceHandlers for TagProxy {
        client: proto::proto::content::service::v1::tag_service_client::TagServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            pass list(proto::proto::pagination::PagingRequest) -> proto::proto::content::service::v1::ListTagResponse,
            pass get(proto::proto::content::service::v1::GetTagRequest) -> proto::proto::content::service::v1::Tag,
            hand create(proto::proto::content::service::v1::CreateTagRequest) -> proto::proto::content::service::v1::Tag => forbidden_mutation,
            hand update(proto::proto::content::service::v1::UpdateTagRequest) -> proto::proto::content::service::v1::Tag => forbidden_mutation,
            hand delete(proto::proto::content::service::v1::DeleteTagRequest) -> pbjson_types::Empty => forbidden_mutation,
            pass get_translation(proto::proto::content::service::v1::GetTagRequest) -> proto::proto::content::service::v1::TagTranslation,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `UserProfileServiceHandlers`.
    proto::gen_app::services::UserProfileServiceHandlers for UserProfileProxy {
        client: proto::proto::identity::service::v1::user_profile_service_client::UserProfileServiceClient<tonic::transport::Channel>,
        behaviors: public,
        methods: [
            pass get_user(pbjson_types::Empty) -> proto::proto::identity::service::v1::User,
            pass update_user(proto::proto::identity::service::v1::UpdateUserRequest) -> pbjson_types::Empty,
            hand change_password(proto::proto::identity::service::v1::ChangePasswordRequest) -> pbjson_types::Empty => change_password,
            hand bind_contact(proto::proto::identity::service::v1::BindContactRequest) -> pbjson_types::Empty => bind_contact,
            hand verify_contact(proto::proto::identity::service::v1::VerifyContactRequest) -> pbjson_types::Empty => verify_contact,
        ]
    }
}
