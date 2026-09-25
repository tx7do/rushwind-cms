//! The service layer — one file per gRPC service, mirroring the
//! reference's `internal/service` one-file-per-service layout (the
//! *_support/context/permission_sync files are the shared machinery
//! the reference keeps as package-level helpers). Queries ride the
//! data layer ([`crate::data`]) and the shared store crate.

pub mod api_audit_log_service;
pub mod api_service;
pub mod audit_support;
pub mod authentication_service;
pub mod category_service;
pub mod comment_service;
pub mod content_model_service;
pub mod content_support;
pub mod context;
pub mod data_access_audit_log_service;
pub mod dict_entry_service;
pub mod dict_type_service;
pub mod file_service;
pub mod interaction_admin_service;
pub mod interaction_service;
pub mod internal_message_category_service;
pub mod internal_message_recipient_service;
pub mod internal_message_service;
pub mod language_service;
pub mod login_audit_log_service;
pub mod login_policy_service;
pub mod media_asset_service;
pub mod menu_service;
pub mod navigation_item_service;
pub mod navigation_service;
pub mod operation_audit_log_service;
pub mod org_unit_service;
pub mod page_service;
pub mod permission_audit_log_service;
pub mod permission_group_service;
pub mod permission_service;
pub mod permission_sync;
pub mod policy_evaluation_log_service;
pub mod position_service;
pub mod post_service;
pub mod role_service;
pub mod site_service;
pub mod site_setting_service;
pub mod stats_service;
pub mod tag_service;
pub mod task_service;
pub mod tenant_service;
pub mod translator_service;
pub mod user_credential_service;
pub mod user_profile_service;
pub mod user_service;
