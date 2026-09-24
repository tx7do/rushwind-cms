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

/// The pass-through proxy of `CategoryServiceHandlers`.
pub struct CategoryProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_app::services::CategoryServiceHandlers for CategoryProxy {
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

    async fn get_translation(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::GetCategoryRequest,
    ) -> Result<proto::proto::content::service::v1::CategoryTranslation, StatusError> {
        let mut core =
            proto::proto::content::service::v1::category_service_client::CategoryServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get_translation(with_operator(&_ctx, req))
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
impl proto::gen_app::services::CommentServiceHandlers for CommentProxy {
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

/// The pass-through proxy of `InteractionServiceHandlers`.
pub struct InteractionProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_app::services::InteractionServiceHandlers for InteractionProxy {
    async fn like(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::interaction::service::v1::LikeRequest,
    ) -> Result<proto::proto::interaction::service::v1::LikeResponse, StatusError> {
        let mut core = proto::proto::interaction::service::v1::interaction_service_client::InteractionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .like(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn unlike(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::interaction::service::v1::LikeRequest,
    ) -> Result<proto::proto::interaction::service::v1::LikeResponse, StatusError> {
        let mut core = proto::proto::interaction::service::v1::interaction_service_client::InteractionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .unlike(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn watch(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::interaction::service::v1::WatchRequest,
    ) -> Result<proto::proto::interaction::service::v1::WatchResponse, StatusError> {
        let mut core = proto::proto::interaction::service::v1::interaction_service_client::InteractionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .watch(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn unwatch(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::interaction::service::v1::WatchRequest,
    ) -> Result<proto::proto::interaction::service::v1::WatchResponse, StatusError> {
        let mut core = proto::proto::interaction::service::v1::interaction_service_client::InteractionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .unwatch(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get_interaction_status(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::interaction::service::v1::GetInteractionStatusRequest,
    ) -> Result<proto::proto::interaction::service::v1::GetInteractionStatusResponse, StatusError>
    {
        let mut core = proto::proto::interaction::service::v1::interaction_service_client::InteractionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get_interaction_status(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn list_watched_posts(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<proto::proto::content::service::v1::ListPostResponse, StatusError> {
        let mut core = proto::proto::interaction::service::v1::interaction_service_client::InteractionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list_watched_posts(with_operator(&_ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get_counts(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::interaction::service::v1::GetCountsRequest,
    ) -> Result<proto::proto::interaction::service::v1::GetCountsResponse, StatusError> {
        let mut core = proto::proto::interaction::service::v1::interaction_service_client::InteractionServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get_counts(with_operator(&_ctx, req))
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
impl proto::gen_app::services::NavigationServiceHandlers for NavigationProxy {
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

/// The pass-through proxy of `PageServiceHandlers`.
pub struct PageProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_app::services::PageServiceHandlers for PageProxy {
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

    async fn get_translation(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::GetPageRequest,
    ) -> Result<proto::proto::content::service::v1::PageTranslation, StatusError> {
        let mut core =
            proto::proto::content::service::v1::page_service_client::PageServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get_translation(with_operator(&_ctx, req))
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
impl proto::gen_app::services::PostServiceHandlers for PostProxy {
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

    async fn search_posts(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::SearchPostsRequest,
    ) -> Result<proto::proto::content::service::v1::SearchPostsResponse, StatusError> {
        let mut core =
            proto::proto::content::service::v1::post_service_client::PostServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .search_posts(with_operator(&_ctx, req))
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

    async fn get_translation(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::GetPostRequest,
    ) -> Result<proto::proto::content::service::v1::PostTranslation, StatusError> {
        let mut core =
            proto::proto::content::service::v1::post_service_client::PostServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get_translation(with_operator(&_ctx, req))
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
impl proto::gen_app::services::SiteServiceHandlers for SiteProxy {
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

    async fn get_site_by_domain(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::site::service::v1::GetSiteByDomainRequest,
    ) -> Result<proto::proto::site::service::v1::Site, StatusError> {
        let mut core = proto::proto::site::service::v1::site_service_client::SiteServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get_site_by_domain(with_operator(&_ctx, req))
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

/// The pass-through proxy of `TagServiceHandlers`.
pub struct TagProxy {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_app::services::TagServiceHandlers for TagProxy {
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

    async fn get_translation(
        &self,
        _ctx: rushwind_http_binding::ctx::RequestContext,
        req: proto::proto::content::service::v1::GetTagRequest,
    ) -> Result<proto::proto::content::service::v1::TagTranslation, StatusError> {
        let mut core =
            proto::proto::content::service::v1::tag_service_client::TagServiceClient::new(
                self.state.core_channel.clone(),
            );
        Ok(core
            .get_translation(with_operator(&_ctx, req))
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
impl proto::gen_app::services::UserProfileServiceHandlers for UserProfileProxy {
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
