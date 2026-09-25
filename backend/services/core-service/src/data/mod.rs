//! The data layer — one repo file per reference repo, mirroring the
//! reference's `internal/data` layout. The sea-orm entity tree and the
//! paging pipeline live in the shared `store` crate.

pub mod api_repo;
pub mod category_repo;
pub mod category_translation_repo;
pub mod comment_repo;
pub mod content_model_repo;
pub mod dict_entry_repo;
pub mod dict_type_repo;
pub mod file_repo;
pub mod interaction_repo;
pub mod internal_message_category_repo;
pub mod internal_message_recipient_repo;
pub mod internal_message_repo;
pub mod language_repo;
pub mod login_policy_repo;
pub mod media_asset_repo;
pub mod menu_repo;
pub mod navigation_item_repo;
pub mod navigation_repo;
pub mod org_unit_repo;
pub mod page_repo;
pub mod page_translation_repo;
pub mod permission_api_repo;
pub mod permission_group_repo;
pub mod permission_menu_repo;
pub mod permission_repo;
pub mod policy_evaluation_log_repo;
pub mod position_repo;
pub mod post_category_repo;
pub mod post_repo;
pub mod post_tag_repo;
pub mod post_translation_repo;
pub mod role_metadata_repo;
pub mod role_permission_repo;
pub mod role_repo;
pub mod search_repo;
pub mod site_repo;
pub mod site_setting_repo;
pub mod tag_repo;
pub mod tag_translation_repo;
pub mod task_repo;
pub mod tenant_repo;
pub mod user_credential_repo;
pub mod user_repo;
pub mod user_role_repo;
