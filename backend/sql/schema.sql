-- Golden DDL（从 pg_catalog 导出；空库引导时按语句原样执行）

CREATE SEQUENCE IF NOT EXISTS "categories_id_seq";
CREATE SEQUENCE IF NOT EXISTS "category_translations_id_seq";
CREATE SEQUENCE IF NOT EXISTS "comment_likes_id_seq";
CREATE SEQUENCE IF NOT EXISTS "comments_id_seq";
CREATE SEQUENCE IF NOT EXISTS "content_model_translations_id_seq";
CREATE SEQUENCE IF NOT EXISTS "content_models_id_seq";
CREATE SEQUENCE IF NOT EXISTS "field_definition_translations_id_seq";
CREATE SEQUENCE IF NOT EXISTS "field_definitions_id_seq";
CREATE SEQUENCE IF NOT EXISTS "files_id_seq";
CREATE SEQUENCE IF NOT EXISTS "interaction_counters_id_seq";
CREATE SEQUENCE IF NOT EXISTS "internal_message_categories_id_seq";
CREATE SEQUENCE IF NOT EXISTS "internal_message_recipients_id_seq";
CREATE SEQUENCE IF NOT EXISTS "internal_messages_id_seq";
CREATE SEQUENCE IF NOT EXISTS "media_assets_id_seq";
CREATE SEQUENCE IF NOT EXISTS "media_variants_id_seq";
CREATE SEQUENCE IF NOT EXISTS "navigation_items_id_seq";
CREATE SEQUENCE IF NOT EXISTS "navigations_id_seq";
CREATE SEQUENCE IF NOT EXISTS "page_translations_id_seq";
CREATE SEQUENCE IF NOT EXISTS "pages_id_seq";
CREATE SEQUENCE IF NOT EXISTS "post_categories_id_seq";
CREATE SEQUENCE IF NOT EXISTS "post_likes_id_seq";
CREATE SEQUENCE IF NOT EXISTS "post_tags_id_seq";
CREATE SEQUENCE IF NOT EXISTS "post_translations_id_seq";
CREATE SEQUENCE IF NOT EXISTS "post_watches_id_seq";
CREATE SEQUENCE IF NOT EXISTS "posts_id_seq";
CREATE SEQUENCE IF NOT EXISTS "section_translations_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sections_id_seq";
CREATE SEQUENCE IF NOT EXISTS "site_settings_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sites_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_api_audit_logs_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_apis_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_data_access_audit_logs_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_dict_entries_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_dict_entry_i18n_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_dict_types_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_languages_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_login_audit_logs_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_login_policies_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_membership_org_units_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_membership_positions_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_membership_roles_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_memberships_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_menus_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_operation_audit_logs_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_org_units_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_permission_apis_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_permission_audit_logs_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_permission_groups_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_permission_menus_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_permission_policies_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_permissions_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_policy_evaluation_logs_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_positions_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_role_metadata_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_role_permissions_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_roles_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_tasks_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_tenants_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_user_credentials_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_user_org_units_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_user_positions_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_user_roles_id_seq";
CREATE SEQUENCE IF NOT EXISTS "sys_users_id_seq";
CREATE SEQUENCE IF NOT EXISTS "tag_translations_id_seq";
CREATE SEQUENCE IF NOT EXISTS "tags_id_seq";

CREATE TABLE IF NOT EXISTS "categories" (
  "id" bigint DEFAULT nextval('categories_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "sort_order" bigint DEFAULT 0,
  "path" character varying,
  "tenant_id" bigint DEFAULT 0,
  "status" character varying,
  "is_nav" boolean DEFAULT false,
  "icon" character varying,
  "code" character varying,
  "thumbnail" character varying,
  "post_count" bigint DEFAULT 0,
  "direct_post_count" bigint DEFAULT 0,
  "depth" integer DEFAULT 0,
  "custom_fields" jsonb,
  "content_model_id" bigint,
  "parent_id" bigint
);

CREATE TABLE IF NOT EXISTS "category_translations" (
  "id" bigint DEFAULT nextval('category_translations_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "seo" jsonb,
  "tenant_id" bigint DEFAULT 0,
  "category_id" bigint,
  "language_code" character varying,
  "name" character varying,
  "slug" character varying,
  "description" character varying,
  "cover_image" character varying,
  "full_path" character varying
);

CREATE TABLE IF NOT EXISTS "comment_likes" (
  "id" bigint DEFAULT nextval('comment_likes_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "user_id" bigint,
  "comment_id" bigint
);

CREATE TABLE IF NOT EXISTS "comments" (
  "id" bigint DEFAULT nextval('comments_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "content_type" character varying,
  "object_id" bigint,
  "content" character varying,
  "author_id" bigint DEFAULT 0,
  "author_name" character varying,
  "author_email" character varying,
  "author_url" character varying,
  "author_type" character varying,
  "status" character varying,
  "ip_address" character varying,
  "location" character varying,
  "user_agent" character varying,
  "detected_language" character varying,
  "is_spam" boolean DEFAULT false,
  "is_sticky" boolean DEFAULT false,
  "reply_to_id" bigint,
  "parent_id" bigint
);

CREATE TABLE IF NOT EXISTS "content_model_translations" (
  "id" bigint DEFAULT nextval('content_model_translations_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "content_model_id" bigint,
  "language_code" character varying,
  "name" character varying,
  "description" character varying
);

CREATE TABLE IF NOT EXISTS "content_models" (
  "id" bigint DEFAULT nextval('content_models_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "sort_order" bigint DEFAULT 0,
  "tenant_id" bigint DEFAULT 0,
  "name" character varying,
  "code" character varying,
  "description" character varying
);

CREATE TABLE IF NOT EXISTS "field_definition_translations" (
  "id" bigint DEFAULT nextval('field_definition_translations_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "field_definition_id" bigint,
  "language_code" character varying,
  "label" character varying,
  "description" character varying,
  "placeholder" character varying
);

CREATE TABLE IF NOT EXISTS "field_definitions" (
  "id" bigint DEFAULT nextval('field_definitions_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "sort_order" bigint DEFAULT 0,
  "tenant_id" bigint DEFAULT 0,
  "content_model_id" bigint,
  "name" character varying,
  "type" character varying DEFAULT 'FIELD_TYPE_TEXT'::character varying,
  "label" character varying,
  "description" character varying,
  "placeholder" character varying,
  "is_required" boolean DEFAULT false,
  "validation_regex" character varying,
  "options" jsonb,
  "relation_config" jsonb
);

CREATE TABLE IF NOT EXISTS "files" (
  "id" bigint DEFAULT nextval('files_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "remark" character varying,
  "tenant_id" bigint DEFAULT 0,
  "provider" character varying DEFAULT 'MINIO'::character varying,
  "bucket_name" character varying,
  "file_directory" character varying,
  "file_guid" character varying,
  "save_file_name" character varying,
  "file_name" character varying,
  "extension" character varying,
  "size" bigint,
  "size_format" character varying,
  "link_url" character varying,
  "content_hash" character varying
);

CREATE TABLE IF NOT EXISTS "interaction_counters" (
  "id" bigint DEFAULT nextval('interaction_counters_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "target_type" smallint,
  "target_id" bigint,
  "metric" smallint,
  "count" bigint DEFAULT 0
);

CREATE TABLE IF NOT EXISTS "internal_message_categories" (
  "id" bigint DEFAULT nextval('internal_message_categories_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "is_enabled" boolean DEFAULT true,
  "sort_order" bigint DEFAULT 0,
  "path" character varying,
  "remark" character varying,
  "tenant_id" bigint DEFAULT 0,
  "name" character varying,
  "code" character varying,
  "icon_url" character varying,
  "depth" integer DEFAULT 0,
  "parent_id" bigint
);

CREATE TABLE IF NOT EXISTS "internal_message_recipients" (
  "id" bigint DEFAULT nextval('internal_message_recipients_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "message_id" bigint,
  "recipient_user_id" bigint,
  "status" character varying,
  "received_at" timestamp with time zone,
  "read_at" timestamp with time zone
);

CREATE TABLE IF NOT EXISTS "internal_messages" (
  "id" bigint DEFAULT nextval('internal_messages_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "title" character varying,
  "content" character varying,
  "sender_id" bigint NOT NULL,
  "category_id" bigint,
  "status" character varying DEFAULT 'DRAFT'::character varying,
  "type" character varying DEFAULT 'NOTIFICATION'::character varying
);

CREATE TABLE IF NOT EXISTS "media_assets" (
  "id" bigint DEFAULT nextval('media_assets_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "filename" character varying,
  "type" character varying,
  "mime_type" character varying,
  "size" bigint DEFAULT 0,
  "storage_path" character varying,
  "url" character varying,
  "width" bigint DEFAULT 0,
  "height" bigint DEFAULT 0,
  "duration" bigint DEFAULT 0,
  "alt_text" character varying,
  "title" character varying,
  "caption" character varying,
  "processing_status" character varying,
  "processing_error" character varying,
  "file_hash" character varying,
  "folder_id" bigint DEFAULT 0,
  "file_id" bigint,
  "reference_count" bigint DEFAULT 0,
  "is_private" boolean DEFAULT false
);

CREATE TABLE IF NOT EXISTS "media_variants" (
  "id" bigint DEFAULT nextval('media_variants_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "media_id" bigint NOT NULL,
  "file_id" bigint NOT NULL,
  "variant_name" bigint
);

CREATE TABLE IF NOT EXISTS "navigation_items" (
  "id" bigint DEFAULT nextval('navigation_items_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "sort_order" bigint DEFAULT 0,
  "tenant_id" bigint DEFAULT 0,
  "link_type" character varying DEFAULT 'LINK_TYPE_POST'::character varying,
  "navigation_id" bigint,
  "title" character varying,
  "url" character varying,
  "object_id" bigint,
  "icon" character varying,
  "description" character varying,
  "is_open_new_tab" boolean DEFAULT false,
  "is_invalid" boolean DEFAULT false,
  "required_permission" character varying,
  "parent_id" bigint
);

CREATE TABLE IF NOT EXISTS "navigations" (
  "id" bigint DEFAULT nextval('navigations_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "name" character varying,
  "location" character varying DEFAULT 'HEADER'::character varying,
  "locale" character varying,
  "is_active" boolean DEFAULT true
);

CREATE TABLE IF NOT EXISTS "page_translations" (
  "id" bigint DEFAULT nextval('page_translations_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "seo" jsonb,
  "tenant_id" bigint DEFAULT 0,
  "page_id" bigint,
  "language_code" character varying,
  "title" character varying,
  "slug" character varying,
  "cover_image" character varying,
  "full_path" character varying
);

CREATE TABLE IF NOT EXISTS "pages" (
  "id" bigint DEFAULT nextval('pages_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "sort_order" bigint DEFAULT 0,
  "path" character varying,
  "editor_type" character varying DEFAULT 'EDITOR_TYPE_MARKDOWN'::character varying,
  "tenant_id" bigint DEFAULT 0,
  "status" character varying DEFAULT 'PAGE_STATUS_DRAFT'::character varying,
  "type" character varying DEFAULT 'PAGE_TYPE_HOME'::character varying,
  "slug" character varying,
  "author_id" bigint DEFAULT 0,
  "author_name" character varying,
  "disallow_comment" boolean DEFAULT false,
  "redirect_url" character varying,
  "show_in_navigation" boolean DEFAULT false,
  "template" character varying,
  "is_custom_template" boolean DEFAULT false,
  "thumbnail" character varying,
  "custom_fields" jsonb,
  "content_model_id" bigint,
  "depth" integer DEFAULT 0,
  "parent_id" bigint
);

CREATE TABLE IF NOT EXISTS "post_categories" (
  "id" bigint DEFAULT nextval('post_categories_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "post_id" bigint NOT NULL,
  "category_id" bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS "post_likes" (
  "id" bigint DEFAULT nextval('post_likes_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "user_id" bigint,
  "post_id" bigint
);

CREATE TABLE IF NOT EXISTS "post_tags" (
  "id" bigint DEFAULT nextval('post_tags_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "post_id" bigint NOT NULL,
  "tag_id" bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS "post_translations" (
  "id" bigint DEFAULT nextval('post_translations_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "seo" jsonb,
  "tenant_id" bigint DEFAULT 0,
  "post_id" bigint,
  "language_code" character varying,
  "title" character varying,
  "slug" character varying,
  "summary" character varying,
  "content" character varying,
  "original_content" character varying,
  "full_path" character varying,
  "word_count" bigint DEFAULT 0
);

CREATE TABLE IF NOT EXISTS "post_watches" (
  "id" bigint DEFAULT nextval('post_watches_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "user_id" bigint,
  "post_id" bigint
);

CREATE TABLE IF NOT EXISTS "posts" (
  "id" bigint DEFAULT nextval('posts_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "sort_order" bigint DEFAULT 0,
  "editor_type" character varying DEFAULT 'EDITOR_TYPE_MARKDOWN'::character varying,
  "tenant_id" bigint DEFAULT 0,
  "status" character varying DEFAULT 'POST_STATUS_DRAFT'::character varying,
  "code" character varying,
  "disallow_comment" boolean DEFAULT false,
  "in_progress" boolean DEFAULT false,
  "auto_summary" boolean DEFAULT true,
  "is_featured" boolean DEFAULT false,
  "author_id" bigint DEFAULT 0,
  "author_name" character varying,
  "thumbnail" character varying,
  "password_hash" character varying,
  "custom_fields" jsonb,
  "publish_time" timestamp with time zone
);

CREATE TABLE IF NOT EXISTS "section_translations" (
  "id" bigint DEFAULT nextval('section_translations_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "section_id" bigint,
  "language_code" character varying,
  "content" jsonb
);

CREATE TABLE IF NOT EXISTS "sections" (
  "id" bigint DEFAULT nextval('sections_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "sort_order" bigint DEFAULT 0,
  "tenant_id" bigint DEFAULT 0,
  "page_id" bigint,
  "type" character varying DEFAULT 'SECTION_TYPE_RICH_TEXT'::character varying,
  "name" character varying,
  "config" jsonb
);

CREATE TABLE IF NOT EXISTS "site_settings" (
  "id" bigint DEFAULT nextval('site_settings_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "site_id" bigint,
  "locale" character varying,
  "group" character varying,
  "key" character varying,
  "value" character varying,
  "type" character varying DEFAULT 'SETTING_TYPE_TEXT'::character varying,
  "label" character varying,
  "description" character varying,
  "placeholder" character varying,
  "options" jsonb,
  "is_required" boolean DEFAULT false,
  "validation_regex" character varying
);

CREATE TABLE IF NOT EXISTS "sites" (
  "id" bigint DEFAULT nextval('sites_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "name" character varying,
  "slug" character varying,
  "domain" character varying,
  "alternate_domains" jsonb,
  "is_default" boolean DEFAULT false,
  "status" character varying DEFAULT 'SITE_STATUS_ACTIVE'::character varying,
  "default_locale" character varying,
  "template" character varying,
  "theme" character varying
);

CREATE TABLE IF NOT EXISTS "sys_api_audit_logs" (
  "id" bigint DEFAULT nextval('sys_api_audit_logs_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "user_id" bigint,
  "username" character varying,
  "ip_address" character varying,
  "geo_location" jsonb,
  "device_info" jsonb,
  "referer" character varying,
  "app_version" character varying,
  "http_method" character varying,
  "path" character varying,
  "request_uri" character varying,
  "api_module" character varying,
  "api_operation" character varying,
  "api_description" character varying,
  "request_id" character varying,
  "trace_id" character varying,
  "span_id" character varying,
  "latency_ms" bigint,
  "success" boolean,
  "status_code" bigint,
  "reason" character varying,
  "request_header" character varying,
  "request_body" character varying,
  "response" character varying,
  "log_hash" character varying,
  "signature" bytea
);

CREATE TABLE IF NOT EXISTS "sys_apis" (
  "id" bigint DEFAULT nextval('sys_apis_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "status" character varying NOT NULL DEFAULT 'ON'::character varying,
  "tenant_id" bigint DEFAULT 0,
  "description" character varying,
  "module" character varying,
  "module_description" character varying,
  "operation" character varying,
  "path" character varying,
  "method" character varying,
  "scope" character varying DEFAULT 'ADMIN'::character varying
);

CREATE TABLE IF NOT EXISTS "sys_data_access_audit_logs" (
  "id" bigint DEFAULT nextval('sys_data_access_audit_logs_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "user_id" bigint,
  "username" character varying,
  "ip_address" character varying,
  "geo_location" jsonb,
  "device_info" jsonb,
  "request_id" character varying,
  "trace_id" character varying,
  "data_source" character varying,
  "table_name" character varying,
  "data_id" character varying,
  "access_type" character varying,
  "sql_digest" character varying,
  "sql_text" character varying,
  "affected_rows" bigint,
  "latency_ms" bigint,
  "success" boolean,
  "sensitive_level" character varying,
  "data_masked" boolean,
  "masking_rules" character varying,
  "business_purpose" character varying,
  "data_category" character varying,
  "db_user" character varying,
  "log_hash" character varying,
  "signature" bytea
);

CREATE TABLE IF NOT EXISTS "sys_dict_entries" (
  "id" bigint DEFAULT nextval('sys_dict_entries_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "sort_order" bigint DEFAULT 0,
  "is_enabled" boolean DEFAULT true,
  "tenant_id" bigint DEFAULT 0,
  "entry_value" character varying NOT NULL,
  "numeric_value" integer,
  "type_id" bigint
);

CREATE TABLE IF NOT EXISTS "sys_dict_entry_i18n" (
  "id" bigint DEFAULT nextval('sys_dict_entry_i18n_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "description" character varying,
  "sort_order" bigint DEFAULT 0,
  "tenant_id" bigint DEFAULT 0,
  "language_code" character varying,
  "entry_label" character varying,
  "entry_id" bigint
);

CREATE TABLE IF NOT EXISTS "sys_dict_types" (
  "id" bigint DEFAULT nextval('sys_dict_types_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "is_enabled" boolean DEFAULT true,
  "sort_order" bigint DEFAULT 0,
  "tenant_id" bigint DEFAULT 0,
  "type_code" character varying,
  "type_name" character varying
);

CREATE TABLE IF NOT EXISTS "sys_languages" (
  "id" bigint DEFAULT nextval('sys_languages_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "sort_order" bigint DEFAULT 0,
  "is_enabled" boolean DEFAULT true,
  "language_code" character varying,
  "language_name" character varying,
  "native_name" character varying,
  "is_default" boolean DEFAULT false
);

CREATE TABLE IF NOT EXISTS "sys_login_audit_logs" (
  "id" bigint DEFAULT nextval('sys_login_audit_logs_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "user_id" bigint,
  "username" character varying,
  "ip_address" character varying,
  "geo_location" jsonb,
  "session_id" character varying,
  "device_info" jsonb,
  "request_id" character varying,
  "trace_id" character varying,
  "action_type" character varying,
  "status" character varying,
  "login_method" character varying,
  "failure_reason" character varying,
  "mfa_status" character varying,
  "risk_score" bigint,
  "risk_level" character varying,
  "risk_factors" jsonb,
  "log_hash" character varying,
  "signature" bytea
);

CREATE TABLE IF NOT EXISTS "sys_login_policies" (
  "id" bigint DEFAULT nextval('sys_login_policies_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "target_id" bigint,
  "value" character varying,
  "reason" character varying,
  "type" character varying DEFAULT 'BLACKLIST'::character varying,
  "method" character varying DEFAULT 'IP'::character varying
);

CREATE TABLE IF NOT EXISTS "sys_membership_org_units" (
  "id" bigint DEFAULT nextval('sys_membership_org_units_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "remark" character varying,
  "membership_id" bigint NOT NULL,
  "org_unit_id" bigint NOT NULL,
  "position_id" bigint,
  "role_id" bigint,
  "start_at" timestamp with time zone,
  "end_at" timestamp with time zone,
  "assigned_at" timestamp with time zone,
  "assigned_by" bigint,
  "is_primary" boolean NOT NULL DEFAULT false,
  "status" character varying DEFAULT 'ACTIVE'::character varying
);

CREATE TABLE IF NOT EXISTS "sys_membership_positions" (
  "id" bigint DEFAULT nextval('sys_membership_positions_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "remark" character varying,
  "membership_id" bigint NOT NULL,
  "position_id" bigint NOT NULL,
  "is_primary" boolean DEFAULT false,
  "start_at" timestamp with time zone,
  "end_at" timestamp with time zone,
  "assigned_at" timestamp with time zone,
  "assigned_by" bigint,
  "status" character varying NOT NULL DEFAULT 'ACTIVE'::character varying
);

CREATE TABLE IF NOT EXISTS "sys_membership_roles" (
  "id" bigint DEFAULT nextval('sys_membership_roles_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "membership_id" bigint NOT NULL,
  "role_id" bigint NOT NULL,
  "start_at" timestamp with time zone,
  "end_at" timestamp with time zone,
  "assigned_at" timestamp with time zone,
  "assigned_by" bigint,
  "is_primary" boolean NOT NULL DEFAULT false,
  "status" character varying NOT NULL DEFAULT 'ACTIVE'::character varying
);

CREATE TABLE IF NOT EXISTS "sys_memberships" (
  "id" bigint DEFAULT nextval('sys_memberships_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "remark" character varying,
  "user_id" bigint NOT NULL,
  "org_unit_id" bigint,
  "position_id" bigint,
  "role_id" bigint,
  "is_primary" boolean NOT NULL DEFAULT false,
  "start_at" timestamp with time zone,
  "end_at" timestamp with time zone,
  "assigned_at" timestamp with time zone,
  "assigned_by" bigint,
  "joined_at" timestamp with time zone,
  "status" character varying DEFAULT 'ACTIVE'::character varying
);

CREATE TABLE IF NOT EXISTS "sys_menus" (
  "id" bigint DEFAULT nextval('sys_menus_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "remark" character varying,
  "status" character varying NOT NULL DEFAULT 'ON'::character varying,
  "tenant_id" bigint DEFAULT 0,
  "type" character varying DEFAULT 'MENU'::character varying,
  "path" character varying DEFAULT ''::character varying,
  "redirect" character varying,
  "alias" character varying,
  "name" character varying,
  "component" character varying DEFAULT ''::character varying,
  "meta" jsonb,
  "parent_id" bigint
);

CREATE TABLE IF NOT EXISTS "sys_operation_audit_logs" (
  "id" bigint DEFAULT nextval('sys_operation_audit_logs_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "user_id" bigint,
  "username" character varying,
  "resource_type" character varying,
  "resource_id" character varying,
  "action" character varying,
  "before_data" jsonb,
  "after_data" jsonb,
  "sensitive_level" character varying,
  "request_id" character varying,
  "trace_id" character varying,
  "success" boolean,
  "failure_reason" character varying,
  "ip_address" character varying,
  "geo_location" jsonb,
  "device_info" jsonb,
  "log_hash" character varying,
  "signature" bytea
);

CREATE TABLE IF NOT EXISTS "sys_org_units" (
  "id" bigint DEFAULT nextval('sys_org_units_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "status" character varying NOT NULL DEFAULT 'ON'::character varying,
  "sort_order" bigint DEFAULT 0,
  "tenant_id" bigint DEFAULT 0,
  "remark" character varying,
  "description" character varying,
  "path" character varying,
  "name" character varying NOT NULL,
  "code" character varying,
  "leader_id" bigint,
  "type" character varying NOT NULL DEFAULT 'DEPARTMENT'::character varying,
  "business_scopes" jsonb,
  "external_id" character varying,
  "is_legal_entity" boolean DEFAULT false,
  "registration_number" character varying,
  "tax_id" character varying,
  "legal_entity_org_id" bigint,
  "address" character varying,
  "phone" character varying,
  "email" character varying,
  "timezone" character varying,
  "country" character varying,
  "latitude" double precision,
  "longitude" double precision,
  "start_at" timestamp with time zone,
  "end_at" timestamp with time zone,
  "contact_user_id" bigint,
  "permission_tags" jsonb,
  "parent_id" bigint
);

CREATE TABLE IF NOT EXISTS "sys_permission_apis" (
  "id" bigint DEFAULT nextval('sys_permission_apis_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "permission_id" bigint NOT NULL,
  "api_id" bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS "sys_permission_audit_logs" (
  "id" bigint DEFAULT nextval('sys_permission_audit_logs_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "operator_id" bigint,
  "target_type" character varying,
  "target_id" character varying,
  "action" character varying,
  "old_value" jsonb,
  "new_value" jsonb,
  "ip_address" character varying NOT NULL,
  "request_id" character varying NOT NULL,
  "reason" character varying NOT NULL,
  "log_hash" character varying,
  "signature" bytea
);

CREATE TABLE IF NOT EXISTS "sys_permission_groups" (
  "id" bigint DEFAULT nextval('sys_permission_groups_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "description" character varying,
  "status" character varying NOT NULL DEFAULT 'ON'::character varying,
  "sort_order" bigint DEFAULT 0,
  "path" character varying,
  "tenant_id" bigint DEFAULT 0,
  "name" character varying NOT NULL,
  "module" character varying,
  "parent_id" bigint
);

CREATE TABLE IF NOT EXISTS "sys_permission_menus" (
  "id" bigint DEFAULT nextval('sys_permission_menus_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "permission_id" bigint NOT NULL,
  "menu_id" bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS "sys_permission_policies" (
  "id" bigint DEFAULT nextval('sys_permission_policies_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "status" character varying NOT NULL DEFAULT 'ON'::character varying,
  "tenant_id" bigint DEFAULT 0,
  "permission_id" bigint NOT NULL,
  "policy_engine" character varying NOT NULL DEFAULT 'CASBIN'::character varying,
  "definition" jsonb,
  "version" bigint NOT NULL DEFAULT 1,
  "eval_order" bigint NOT NULL DEFAULT 0,
  "cache_ttl" bigint NOT NULL DEFAULT 300
);

CREATE TABLE IF NOT EXISTS "sys_permissions" (
  "id" bigint DEFAULT nextval('sys_permissions_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "status" character varying NOT NULL DEFAULT 'ON'::character varying,
  "description" character varying,
  "tenant_id" bigint DEFAULT 0,
  "name" character varying NOT NULL,
  "code" character varying NOT NULL,
  "group_id" bigint
);

CREATE TABLE IF NOT EXISTS "sys_policy_evaluation_logs" (
  "id" bigint DEFAULT nextval('sys_policy_evaluation_logs_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "user_id" bigint NOT NULL,
  "membership_id" bigint NOT NULL,
  "permission_id" bigint NOT NULL,
  "policy_id" bigint,
  "request_path" character varying,
  "request_method" character varying,
  "result" boolean NOT NULL DEFAULT false,
  "effect_details" character varying,
  "scope_sql" character varying,
  "ip_address" character varying,
  "trace_id" character varying,
  "evaluation_context" character varying,
  "log_hash" character varying,
  "signature" bytea
);

CREATE TABLE IF NOT EXISTS "sys_positions" (
  "id" bigint DEFAULT nextval('sys_positions_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "sort_order" bigint DEFAULT 0,
  "remark" character varying,
  "tenant_id" bigint DEFAULT 0,
  "status" character varying NOT NULL DEFAULT 'ON'::character varying,
  "name" character varying NOT NULL,
  "code" character varying NOT NULL,
  "org_unit_id" bigint NOT NULL,
  "reports_to_position_id" bigint,
  "description" character varying,
  "job_family" character varying,
  "job_grade" character varying,
  "level" integer,
  "headcount" bigint NOT NULL DEFAULT 0,
  "is_key_position" boolean NOT NULL DEFAULT false,
  "type" character varying NOT NULL DEFAULT 'REGULAR'::character varying,
  "start_at" timestamp with time zone,
  "end_at" timestamp with time zone
);

CREATE TABLE IF NOT EXISTS "sys_role_metadata" (
  "id" bigint DEFAULT nextval('sys_role_metadata_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "role_id" bigint,
  "is_template" boolean DEFAULT false,
  "template_for" character varying,
  "template_version" integer DEFAULT 1,
  "last_synced_version" integer,
  "last_synced_at" timestamp with time zone,
  "sync_policy" character varying DEFAULT 'AUTO'::character varying,
  "scope" character varying DEFAULT 'TENANT'::character varying,
  "custom_overrides" jsonb NOT NULL
);

CREATE TABLE IF NOT EXISTS "sys_role_permissions" (
  "id" bigint DEFAULT nextval('sys_role_permissions_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "status" character varying NOT NULL DEFAULT 'ON'::character varying,
  "role_id" bigint NOT NULL,
  "permission_id" bigint NOT NULL,
  "effect" character varying DEFAULT 'ALLOW'::character varying,
  "priority" integer DEFAULT 0
);

CREATE TABLE IF NOT EXISTS "sys_roles" (
  "id" bigint DEFAULT nextval('sys_roles_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "remark" character varying,
  "description" character varying,
  "sort_order" bigint DEFAULT 0,
  "tenant_id" bigint DEFAULT 0,
  "status" character varying NOT NULL DEFAULT 'ON'::character varying,
  "name" character varying,
  "code" character varying,
  "is_protected" boolean NOT NULL DEFAULT false,
  "type" character varying NOT NULL DEFAULT 'TENANT'::character varying
);

CREATE TABLE IF NOT EXISTS "sys_tasks" (
  "id" bigint DEFAULT nextval('sys_tasks_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "remark" character varying,
  "tenant_id" bigint DEFAULT 0,
  "type" character varying DEFAULT 'PERIODIC'::character varying,
  "type_name" character varying,
  "task_payload" jsonb,
  "cron_spec" character varying,
  "task_options" jsonb,
  "enable" boolean DEFAULT false
);

CREATE TABLE IF NOT EXISTS "sys_tenants" (
  "id" bigint DEFAULT nextval('sys_tenants_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "remark" character varying,
  "name" character varying,
  "code" character varying,
  "logo_url" character varying,
  "domain" character varying,
  "industry" character varying,
  "admin_user_id" bigint,
  "status" character varying DEFAULT 'ON'::character varying,
  "type" character varying DEFAULT 'PAID'::character varying,
  "audit_status" character varying,
  "subscription_at" timestamp with time zone,
  "unsubscribe_at" timestamp with time zone,
  "subscription_plan" character varying,
  "expired_at" timestamp with time zone
);

CREATE TABLE IF NOT EXISTS "sys_user_credentials" (
  "id" bigint DEFAULT nextval('sys_user_credentials_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "tenant_id" bigint DEFAULT 0,
  "user_id" bigint,
  "identity_type" character varying DEFAULT 'USERNAME'::character varying,
  "identifier" character varying,
  "credential_type" character varying DEFAULT 'PASSWORD_HASH'::character varying,
  "credential" character varying,
  "is_primary" boolean DEFAULT false,
  "status" character varying DEFAULT 'ENABLED'::character varying,
  "extra_info" jsonb,
  "provider" character varying,
  "provider_account_id" character varying,
  "activate_token_hash" character varying,
  "activate_token_expires_at" timestamp with time zone,
  "activate_token_used_at" timestamp with time zone,
  "reset_token_hash" character varying,
  "reset_token_expires_at" timestamp with time zone,
  "reset_token_used_at" timestamp with time zone
);

CREATE TABLE IF NOT EXISTS "sys_user_org_units" (
  "id" bigint DEFAULT nextval('sys_user_org_units_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "remark" character varying,
  "user_id" bigint NOT NULL,
  "org_unit_id" bigint NOT NULL,
  "position_id" bigint,
  "start_at" timestamp with time zone,
  "end_at" timestamp with time zone,
  "assigned_at" timestamp with time zone,
  "assigned_by" bigint,
  "is_primary" boolean NOT NULL DEFAULT false,
  "status" character varying DEFAULT 'ACTIVE'::character varying
);

CREATE TABLE IF NOT EXISTS "sys_user_positions" (
  "id" bigint DEFAULT nextval('sys_user_positions_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "remark" character varying,
  "user_id" bigint NOT NULL,
  "position_id" bigint NOT NULL,
  "is_primary" boolean DEFAULT false,
  "start_at" timestamp with time zone,
  "end_at" timestamp with time zone,
  "assigned_at" timestamp with time zone,
  "assigned_by" bigint,
  "status" character varying NOT NULL DEFAULT 'ACTIVE'::character varying
);

CREATE TABLE IF NOT EXISTS "sys_user_roles" (
  "id" bigint DEFAULT nextval('sys_user_roles_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "tenant_id" bigint DEFAULT 0,
  "user_id" bigint NOT NULL,
  "role_id" bigint NOT NULL,
  "start_at" timestamp with time zone,
  "end_at" timestamp with time zone,
  "assigned_at" timestamp with time zone,
  "assigned_by" bigint,
  "is_primary" boolean NOT NULL DEFAULT false,
  "status" character varying NOT NULL DEFAULT 'ACTIVE'::character varying
);

CREATE TABLE IF NOT EXISTS "sys_users" (
  "id" bigint DEFAULT nextval('sys_users_id_seq') NOT NULL,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "remark" character varying,
  "tenant_id" bigint DEFAULT 0,
  "username" character varying,
  "nickname" character varying,
  "realname" character varying,
  "email" character varying,
  "mobile" character varying DEFAULT ''::character varying,
  "telephone" character varying DEFAULT ''::character varying,
  "avatar" character varying,
  "address" character varying DEFAULT ''::character varying,
  "region" character varying DEFAULT ''::character varying,
  "description" character varying,
  "gender" character varying DEFAULT 'SECRET'::character varying,
  "last_login_at" timestamp with time zone,
  "last_login_ip" character varying,
  "locked_until" timestamp with time zone,
  "status" character varying DEFAULT 'NORMAL'::character varying
);

CREATE TABLE IF NOT EXISTS "tag_translations" (
  "id" bigint DEFAULT nextval('tag_translations_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "seo" jsonb,
  "tenant_id" bigint DEFAULT 0,
  "tag_id" bigint,
  "language_code" character varying,
  "name" character varying,
  "slug" character varying,
  "description" character varying,
  "cover_image" character varying,
  "full_path" character varying
);

CREATE TABLE IF NOT EXISTS "tags" (
  "id" bigint DEFAULT nextval('tags_id_seq') NOT NULL,
  "created_at" timestamp with time zone,
  "updated_at" timestamp with time zone,
  "deleted_at" timestamp with time zone,
  "created_by" bigint,
  "updated_by" bigint,
  "deleted_by" bigint,
  "sort_order" bigint DEFAULT 0,
  "tenant_id" bigint DEFAULT 0,
  "status" character varying DEFAULT 'TAG_STATUS_ACTIVE'::character varying,
  "color" character varying,
  "icon" character varying,
  "group" character varying,
  "code" character varying,
  "is_featured" boolean DEFAULT false,
  "post_count" bigint DEFAULT 0
);

DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'categories_pkey') THEN ALTER TABLE "categories" ADD CONSTRAINT "categories_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'category_translations_pkey') THEN ALTER TABLE "category_translations" ADD CONSTRAINT "category_translations_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'comment_likes_pkey') THEN ALTER TABLE "comment_likes" ADD CONSTRAINT "comment_likes_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'comments_pkey') THEN ALTER TABLE "comments" ADD CONSTRAINT "comments_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'content_model_translations_pkey') THEN ALTER TABLE "content_model_translations" ADD CONSTRAINT "content_model_translations_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'content_models_pkey') THEN ALTER TABLE "content_models" ADD CONSTRAINT "content_models_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'field_definition_translations_pkey') THEN ALTER TABLE "field_definition_translations" ADD CONSTRAINT "field_definition_translations_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'field_definitions_pkey') THEN ALTER TABLE "field_definitions" ADD CONSTRAINT "field_definitions_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'files_pkey') THEN ALTER TABLE "files" ADD CONSTRAINT "files_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'interaction_counters_pkey') THEN ALTER TABLE "interaction_counters" ADD CONSTRAINT "interaction_counters_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'internal_message_categories_pkey') THEN ALTER TABLE "internal_message_categories" ADD CONSTRAINT "internal_message_categories_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'internal_message_recipients_pkey') THEN ALTER TABLE "internal_message_recipients" ADD CONSTRAINT "internal_message_recipients_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'internal_messages_pkey') THEN ALTER TABLE "internal_messages" ADD CONSTRAINT "internal_messages_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'media_assets_pkey') THEN ALTER TABLE "media_assets" ADD CONSTRAINT "media_assets_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'media_variants_pkey') THEN ALTER TABLE "media_variants" ADD CONSTRAINT "media_variants_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'navigation_items_pkey') THEN ALTER TABLE "navigation_items" ADD CONSTRAINT "navigation_items_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'navigations_pkey') THEN ALTER TABLE "navigations" ADD CONSTRAINT "navigations_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'page_translations_pkey') THEN ALTER TABLE "page_translations" ADD CONSTRAINT "page_translations_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'pages_pkey') THEN ALTER TABLE "pages" ADD CONSTRAINT "pages_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'post_categories_pkey') THEN ALTER TABLE "post_categories" ADD CONSTRAINT "post_categories_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'post_likes_pkey') THEN ALTER TABLE "post_likes" ADD CONSTRAINT "post_likes_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'post_tags_pkey') THEN ALTER TABLE "post_tags" ADD CONSTRAINT "post_tags_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'post_translations_pkey') THEN ALTER TABLE "post_translations" ADD CONSTRAINT "post_translations_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'post_watches_pkey') THEN ALTER TABLE "post_watches" ADD CONSTRAINT "post_watches_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'posts_pkey') THEN ALTER TABLE "posts" ADD CONSTRAINT "posts_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'section_translations_pkey') THEN ALTER TABLE "section_translations" ADD CONSTRAINT "section_translations_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sections_pkey') THEN ALTER TABLE "sections" ADD CONSTRAINT "sections_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'site_settings_pkey') THEN ALTER TABLE "site_settings" ADD CONSTRAINT "site_settings_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sites_pkey') THEN ALTER TABLE "sites" ADD CONSTRAINT "sites_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_api_audit_logs_pkey') THEN ALTER TABLE "sys_api_audit_logs" ADD CONSTRAINT "sys_api_audit_logs_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_apis_pkey') THEN ALTER TABLE "sys_apis" ADD CONSTRAINT "sys_apis_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_data_access_audit_logs_pkey') THEN ALTER TABLE "sys_data_access_audit_logs" ADD CONSTRAINT "sys_data_access_audit_logs_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_dict_entries_pkey') THEN ALTER TABLE "sys_dict_entries" ADD CONSTRAINT "sys_dict_entries_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_dict_entry_i18n_pkey') THEN ALTER TABLE "sys_dict_entry_i18n" ADD CONSTRAINT "sys_dict_entry_i18n_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_dict_types_pkey') THEN ALTER TABLE "sys_dict_types" ADD CONSTRAINT "sys_dict_types_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_languages_pkey') THEN ALTER TABLE "sys_languages" ADD CONSTRAINT "sys_languages_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_login_audit_logs_pkey') THEN ALTER TABLE "sys_login_audit_logs" ADD CONSTRAINT "sys_login_audit_logs_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_login_policies_pkey') THEN ALTER TABLE "sys_login_policies" ADD CONSTRAINT "sys_login_policies_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_membership_org_units_pkey') THEN ALTER TABLE "sys_membership_org_units" ADD CONSTRAINT "sys_membership_org_units_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_membership_positions_pkey') THEN ALTER TABLE "sys_membership_positions" ADD CONSTRAINT "sys_membership_positions_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_membership_roles_pkey') THEN ALTER TABLE "sys_membership_roles" ADD CONSTRAINT "sys_membership_roles_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_memberships_pkey') THEN ALTER TABLE "sys_memberships" ADD CONSTRAINT "sys_memberships_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_menus_pkey') THEN ALTER TABLE "sys_menus" ADD CONSTRAINT "sys_menus_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_operation_audit_logs_pkey') THEN ALTER TABLE "sys_operation_audit_logs" ADD CONSTRAINT "sys_operation_audit_logs_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_org_units_pkey') THEN ALTER TABLE "sys_org_units" ADD CONSTRAINT "sys_org_units_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_permission_apis_pkey') THEN ALTER TABLE "sys_permission_apis" ADD CONSTRAINT "sys_permission_apis_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_permission_audit_logs_pkey') THEN ALTER TABLE "sys_permission_audit_logs" ADD CONSTRAINT "sys_permission_audit_logs_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_permission_groups_pkey') THEN ALTER TABLE "sys_permission_groups" ADD CONSTRAINT "sys_permission_groups_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_permission_menus_pkey') THEN ALTER TABLE "sys_permission_menus" ADD CONSTRAINT "sys_permission_menus_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_permission_policies_pkey') THEN ALTER TABLE "sys_permission_policies" ADD CONSTRAINT "sys_permission_policies_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_permissions_pkey') THEN ALTER TABLE "sys_permissions" ADD CONSTRAINT "sys_permissions_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_policy_evaluation_logs_pkey') THEN ALTER TABLE "sys_policy_evaluation_logs" ADD CONSTRAINT "sys_policy_evaluation_logs_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_positions_pkey') THEN ALTER TABLE "sys_positions" ADD CONSTRAINT "sys_positions_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_role_metadata_pkey') THEN ALTER TABLE "sys_role_metadata" ADD CONSTRAINT "sys_role_metadata_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_role_permissions_pkey') THEN ALTER TABLE "sys_role_permissions" ADD CONSTRAINT "sys_role_permissions_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_roles_pkey') THEN ALTER TABLE "sys_roles" ADD CONSTRAINT "sys_roles_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_tasks_pkey') THEN ALTER TABLE "sys_tasks" ADD CONSTRAINT "sys_tasks_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_tenants_pkey') THEN ALTER TABLE "sys_tenants" ADD CONSTRAINT "sys_tenants_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_user_credentials_pkey') THEN ALTER TABLE "sys_user_credentials" ADD CONSTRAINT "sys_user_credentials_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_user_org_units_pkey') THEN ALTER TABLE "sys_user_org_units" ADD CONSTRAINT "sys_user_org_units_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_user_positions_pkey') THEN ALTER TABLE "sys_user_positions" ADD CONSTRAINT "sys_user_positions_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_user_roles_pkey') THEN ALTER TABLE "sys_user_roles" ADD CONSTRAINT "sys_user_roles_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_users_pkey') THEN ALTER TABLE "sys_users" ADD CONSTRAINT "sys_users_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'tag_translations_pkey') THEN ALTER TABLE "tag_translations" ADD CONSTRAINT "tag_translations_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'tags_pkey') THEN ALTER TABLE "tags" ADD CONSTRAINT "tags_pkey" PRIMARY KEY (id); END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'categories_categories_children') THEN ALTER TABLE "categories" ADD CONSTRAINT "categories_categories_children" FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE SET NULL; END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'comments_comments_children') THEN ALTER TABLE "comments" ADD CONSTRAINT "comments_comments_children" FOREIGN KEY (parent_id) REFERENCES comments(id) ON DELETE SET NULL; END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'internal_message_categories_in_8a268228b9922ecb0c6e7d2099d6aa98') THEN ALTER TABLE "internal_message_categories" ADD CONSTRAINT "internal_message_categories_in_8a268228b9922ecb0c6e7d2099d6aa98" FOREIGN KEY (parent_id) REFERENCES internal_message_categories(id) ON DELETE SET NULL; END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'navigation_items_navigation_items_children') THEN ALTER TABLE "navigation_items" ADD CONSTRAINT "navigation_items_navigation_items_children" FOREIGN KEY (parent_id) REFERENCES navigation_items(id) ON DELETE SET NULL; END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'pages_pages_children') THEN ALTER TABLE "pages" ADD CONSTRAINT "pages_pages_children" FOREIGN KEY (parent_id) REFERENCES pages(id) ON DELETE SET NULL; END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_dict_entries_sys_dict_types_entries') THEN ALTER TABLE "sys_dict_entries" ADD CONSTRAINT "sys_dict_entries_sys_dict_types_entries" FOREIGN KEY (type_id) REFERENCES sys_dict_types(id) ON DELETE CASCADE; END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_dict_entry_i18n_sys_dict_entries_i18ns') THEN ALTER TABLE "sys_dict_entry_i18n" ADD CONSTRAINT "sys_dict_entry_i18n_sys_dict_entries_i18ns" FOREIGN KEY (entry_id) REFERENCES sys_dict_entries(id) ON DELETE CASCADE; END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_menus_sys_menus_children') THEN ALTER TABLE "sys_menus" ADD CONSTRAINT "sys_menus_sys_menus_children" FOREIGN KEY (parent_id) REFERENCES sys_menus(id) ON DELETE SET NULL; END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_org_units_sys_org_units_children') THEN ALTER TABLE "sys_org_units" ADD CONSTRAINT "sys_org_units_sys_org_units_children" FOREIGN KEY (parent_id) REFERENCES sys_org_units(id) ON DELETE SET NULL; END IF; END $$;
DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sys_permission_groups_sys_permission_groups_children') THEN ALTER TABLE "sys_permission_groups" ADD CONSTRAINT "sys_permission_groups_sys_permission_groups_children" FOREIGN KEY (parent_id) REFERENCES sys_permission_groups(id) ON DELETE SET NULL; END IF; END $$;

CREATE INDEX IF NOT EXISTS category_is_nav ON public.categories USING btree (is_nav);
CREATE INDEX IF NOT EXISTS category_parent_id ON public.categories USING btree (parent_id);
CREATE INDEX IF NOT EXISTS category_status ON public.categories USING btree (status);
CREATE INDEX IF NOT EXISTS categorytranslation_category_id ON public.category_translations USING btree (category_id);
CREATE UNIQUE INDEX IF NOT EXISTS categorytranslation_category_id_language_code ON public.category_translations USING btree (category_id, language_code);
CREATE INDEX IF NOT EXISTS categorytranslation_language_code ON public.category_translations USING btree (language_code);
CREATE INDEX IF NOT EXISTS categorytranslation_slug ON public.category_translations USING btree (slug);
CREATE UNIQUE INDEX IF NOT EXISTS commentlike_tenant_id_user_id_comment_id ON public.comment_likes USING btree (tenant_id, user_id, comment_id);
CREATE INDEX IF NOT EXISTS comment_author_id ON public.comments USING btree (author_id);
CREATE INDEX IF NOT EXISTS comment_content_type_object_id ON public.comments USING btree (content_type, object_id);
CREATE INDEX IF NOT EXISTS comment_is_spam ON public.comments USING btree (is_spam);
CREATE INDEX IF NOT EXISTS comment_is_sticky ON public.comments USING btree (is_sticky);
CREATE INDEX IF NOT EXISTS comment_reply_to_id ON public.comments USING btree (reply_to_id);
CREATE INDEX IF NOT EXISTS comment_status ON public.comments USING btree (status);
CREATE INDEX IF NOT EXISTS contentmodeltranslation_content_model_id ON public.content_model_translations USING btree (content_model_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_content_model_translation_model_lang ON public.content_model_translations USING btree (content_model_id, language_code);
CREATE UNIQUE INDEX IF NOT EXISTS uix_content_model_tenant_code ON public.content_models USING btree (tenant_id, code);
CREATE INDEX IF NOT EXISTS fielddefinitiontranslation_field_definition_id ON public.field_definition_translations USING btree (field_definition_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_field_definition_translation_field_lang ON public.field_definition_translations USING btree (field_definition_id, language_code);
CREATE INDEX IF NOT EXISTS fielddefinition_content_model_id ON public.field_definitions USING btree (content_model_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_field_definition_model_name ON public.field_definitions USING btree (content_model_id, name);
CREATE INDEX IF NOT EXISTS idx_files_bucket_name ON public.files USING btree (bucket_name);
CREATE INDEX IF NOT EXISTS idx_files_content_hash ON public.files USING btree (content_hash);
CREATE INDEX IF NOT EXISTS idx_files_created_at ON public.files USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_files_extension ON public.files USING btree (extension);
CREATE INDEX IF NOT EXISTS idx_files_file_name ON public.files USING btree (file_name);
CREATE INDEX IF NOT EXISTS idx_files_size ON public.files USING btree (size);
CREATE INDEX IF NOT EXISTS idx_files_tenant_content_hash ON public.files USING btree (tenant_id, content_hash);
CREATE INDEX IF NOT EXISTS idx_files_tenant_id ON public.files USING btree (tenant_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_files_tenant_file_guid ON public.files USING btree (tenant_id, file_guid);
CREATE UNIQUE INDEX IF NOT EXISTS interactioncounter_tenant_id_target_type_target_id_metric ON public.interaction_counters USING btree (tenant_id, target_type, target_id, metric);
CREATE UNIQUE INDEX IF NOT EXISTS idx_internal_msg_cat_tenant_code ON public.internal_message_categories USING btree (tenant_id, code);
CREATE INDEX IF NOT EXISTS idx_internal_msg_cat_tenant_created_at ON public.internal_message_categories USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_internal_msg_cat_tenant_created_by ON public.internal_message_categories USING btree (tenant_id, created_by);
CREATE INDEX IF NOT EXISTS idx_internal_msg_cat_tenant_enabled ON public.internal_message_categories USING btree (tenant_id, is_enabled);
CREATE INDEX IF NOT EXISTS idx_internal_msg_cat_tenant_name ON public.internal_message_categories USING btree (tenant_id, name);
CREATE INDEX IF NOT EXISTS internalmessagecategory_parent_id ON public.internal_message_categories USING btree (parent_id);
CREATE INDEX IF NOT EXISTS idx_internal_msg_recipient_message_recipient ON public.internal_message_recipients USING btree (message_id, recipient_user_id);
CREATE INDEX IF NOT EXISTS idx_internal_msg_recipient_recipient_status_created_at ON public.internal_message_recipients USING btree (recipient_user_id, status, created_at);
CREATE INDEX IF NOT EXISTS idx_internal_msg_recipient_tenant_created_at ON public.internal_message_recipients USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_internal_msg_recipient_tenant_message ON public.internal_message_recipients USING btree (tenant_id, message_id);
CREATE INDEX IF NOT EXISTS idx_internal_msg_recipient_tenant_recipient_created_at ON public.internal_message_recipients USING btree (tenant_id, recipient_user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_internal_msg_recipient_tenant_status_created_at ON public.internal_message_recipients USING btree (tenant_id, status, created_at);
CREATE INDEX IF NOT EXISTS idx_internal_msg_tenant_category ON public.internal_messages USING btree (tenant_id, category_id);
CREATE INDEX IF NOT EXISTS idx_internal_msg_tenant_created_at ON public.internal_messages USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_internal_msg_tenant_created_by_created_at ON public.internal_messages USING btree (tenant_id, created_by, created_at);
CREATE INDEX IF NOT EXISTS idx_internal_msg_tenant_sender_created_at ON public.internal_messages USING btree (tenant_id, sender_id, created_at);
CREATE INDEX IF NOT EXISTS idx_internal_msg_tenant_status_created_at ON public.internal_messages USING btree (tenant_id, status, created_at);
CREATE INDEX IF NOT EXISTS mediaasset_file_hash ON public.media_assets USING btree (file_hash);
CREATE INDEX IF NOT EXISTS mediaasset_file_id ON public.media_assets USING btree (file_id);
CREATE INDEX IF NOT EXISTS mediaasset_folder_id ON public.media_assets USING btree (folder_id);
CREATE INDEX IF NOT EXISTS mediaasset_folder_id_is_private ON public.media_assets USING btree (folder_id, is_private);
CREATE INDEX IF NOT EXISTS mediaasset_is_private ON public.media_assets USING btree (is_private);
CREATE INDEX IF NOT EXISTS mediaasset_processing_status ON public.media_assets USING btree (processing_status);
CREATE INDEX IF NOT EXISTS mediaasset_type ON public.media_assets USING btree (type);
CREATE INDEX IF NOT EXISTS mediaasset_type_processing_status ON public.media_assets USING btree (type, processing_status);
CREATE INDEX IF NOT EXISTS mediavariant_file_id ON public.media_variants USING btree (file_id);
CREATE INDEX IF NOT EXISTS mediavariant_media_id ON public.media_variants USING btree (media_id);
CREATE UNIQUE INDEX IF NOT EXISTS mediavariant_media_id_file_id ON public.media_variants USING btree (media_id, file_id);
CREATE INDEX IF NOT EXISTS navigationitem_is_invalid ON public.navigation_items USING btree (is_invalid);
CREATE INDEX IF NOT EXISTS navigationitem_link_type ON public.navigation_items USING btree (link_type);
CREATE INDEX IF NOT EXISTS navigationitem_navigation_id ON public.navigation_items USING btree (navigation_id);
CREATE INDEX IF NOT EXISTS navigationitem_navigation_id_is_invalid ON public.navigation_items USING btree (navigation_id, is_invalid);
CREATE INDEX IF NOT EXISTS navigationitem_navigation_id_link_type ON public.navigation_items USING btree (navigation_id, link_type);
CREATE INDEX IF NOT EXISTS navigationitem_object_id ON public.navigation_items USING btree (object_id);
CREATE INDEX IF NOT EXISTS navigationitem_parent_id ON public.navigation_items USING btree (parent_id);
CREATE INDEX IF NOT EXISTS navigation_is_active ON public.navigations USING btree (is_active);
CREATE INDEX IF NOT EXISTS navigation_locale ON public.navigations USING btree (locale);
CREATE INDEX IF NOT EXISTS navigation_location_locale ON public.navigations USING btree (location, locale);
CREATE INDEX IF NOT EXISTS navigation_location_locale_is_active ON public.navigations USING btree (location, locale, is_active);
CREATE INDEX IF NOT EXISTS pagetranslation_language_code ON public.page_translations USING btree (language_code);
CREATE INDEX IF NOT EXISTS pagetranslation_language_code_slug ON public.page_translations USING btree (language_code, slug);
CREATE INDEX IF NOT EXISTS pagetranslation_page_id ON public.page_translations USING btree (page_id);
CREATE INDEX IF NOT EXISTS pagetranslation_page_id_language_code ON public.page_translations USING btree (page_id, language_code);
CREATE INDEX IF NOT EXISTS pagetranslation_slug ON public.page_translations USING btree (slug);
CREATE INDEX IF NOT EXISTS page_author_id ON public.pages USING btree (author_id);
CREATE INDEX IF NOT EXISTS page_disallow_comment ON public.pages USING btree (disallow_comment);
CREATE INDEX IF NOT EXISTS page_parent_id ON public.pages USING btree (parent_id);
CREATE INDEX IF NOT EXISTS page_show_in_navigation ON public.pages USING btree (show_in_navigation);
CREATE INDEX IF NOT EXISTS page_slug ON public.pages USING btree (slug);
CREATE INDEX IF NOT EXISTS page_status ON public.pages USING btree (status);
CREATE INDEX IF NOT EXISTS page_status_show_in_navigation ON public.pages USING btree (status, show_in_navigation);
CREATE INDEX IF NOT EXISTS page_status_type ON public.pages USING btree (status, type);
CREATE INDEX IF NOT EXISTS page_type ON public.pages USING btree (type);
CREATE INDEX IF NOT EXISTS postcategory_category_id ON public.post_categories USING btree (category_id);
CREATE INDEX IF NOT EXISTS postcategory_post_id ON public.post_categories USING btree (post_id);
CREATE UNIQUE INDEX IF NOT EXISTS postcategory_post_id_category_id ON public.post_categories USING btree (post_id, category_id);
CREATE UNIQUE INDEX IF NOT EXISTS postlike_tenant_id_user_id_post_id ON public.post_likes USING btree (tenant_id, user_id, post_id);
CREATE INDEX IF NOT EXISTS posttag_post_id ON public.post_tags USING btree (post_id);
CREATE UNIQUE INDEX IF NOT EXISTS posttag_post_id_tag_id ON public.post_tags USING btree (post_id, tag_id);
CREATE INDEX IF NOT EXISTS posttag_tag_id ON public.post_tags USING btree (tag_id);
CREATE INDEX IF NOT EXISTS posttranslation_full_path ON public.post_translations USING btree (full_path);
CREATE INDEX IF NOT EXISTS posttranslation_language_code ON public.post_translations USING btree (language_code);
CREATE INDEX IF NOT EXISTS posttranslation_language_code_slug ON public.post_translations USING btree (language_code, slug);
CREATE INDEX IF NOT EXISTS posttranslation_post_id ON public.post_translations USING btree (post_id);
CREATE INDEX IF NOT EXISTS posttranslation_post_id_language_code ON public.post_translations USING btree (post_id, language_code);
CREATE UNIQUE INDEX IF NOT EXISTS posttranslation_post_id_language_code_slug ON public.post_translations USING btree (post_id, language_code, slug);
CREATE INDEX IF NOT EXISTS posttranslation_slug ON public.post_translations USING btree (slug);
CREATE UNIQUE INDEX IF NOT EXISTS postwatch_tenant_id_user_id_post_id ON public.post_watches USING btree (tenant_id, user_id, post_id);
CREATE INDEX IF NOT EXISTS post_author_id ON public.posts USING btree (author_id);
CREATE INDEX IF NOT EXISTS post_code ON public.posts USING btree (code);
CREATE INDEX IF NOT EXISTS post_disallow_comment ON public.posts USING btree (disallow_comment);
CREATE INDEX IF NOT EXISTS post_editor_type ON public.posts USING btree (editor_type);
CREATE INDEX IF NOT EXISTS post_in_progress ON public.posts USING btree (in_progress);
CREATE INDEX IF NOT EXISTS post_is_featured ON public.posts USING btree (is_featured);
CREATE INDEX IF NOT EXISTS post_status ON public.posts USING btree (status);
CREATE INDEX IF NOT EXISTS post_status_author_id ON public.posts USING btree (status, author_id);
CREATE INDEX IF NOT EXISTS post_status_in_progress ON public.posts USING btree (status, in_progress);
CREATE INDEX IF NOT EXISTS post_status_is_featured ON public.posts USING btree (status, is_featured);
CREATE INDEX IF NOT EXISTS sectiontranslation_language_code ON public.section_translations USING btree (language_code);
CREATE INDEX IF NOT EXISTS sectiontranslation_section_id ON public.section_translations USING btree (section_id);
CREATE UNIQUE INDEX IF NOT EXISTS sectiontranslation_section_id_language_code ON public.section_translations USING btree (section_id, language_code);
CREATE INDEX IF NOT EXISTS section_page_id ON public.sections USING btree (page_id);
CREATE INDEX IF NOT EXISTS section_page_id_sort_order ON public.sections USING btree (page_id, sort_order);
CREATE INDEX IF NOT EXISTS sitesetting_group ON public.site_settings USING btree ("group");
CREATE INDEX IF NOT EXISTS sitesetting_key ON public.site_settings USING btree (key);
CREATE INDEX IF NOT EXISTS sitesetting_locale ON public.site_settings USING btree (locale);
CREATE INDEX IF NOT EXISTS sitesetting_locale_group_key ON public.site_settings USING btree (locale, "group", key);
CREATE INDEX IF NOT EXISTS sitesetting_site_id ON public.site_settings USING btree (site_id);
CREATE INDEX IF NOT EXISTS sitesetting_site_id_group_key ON public.site_settings USING btree (site_id, "group", key);
CREATE INDEX IF NOT EXISTS sitesetting_site_id_locale ON public.site_settings USING btree (site_id, locale);
CREATE INDEX IF NOT EXISTS sitesetting_type ON public.site_settings USING btree (type);
CREATE UNIQUE INDEX IF NOT EXISTS site_tenant_id_domain ON public.sites USING btree (tenant_id, domain);
CREATE INDEX IF NOT EXISTS site_tenant_id_is_default ON public.sites USING btree (tenant_id, is_default);
CREATE UNIQUE INDEX IF NOT EXISTS site_tenant_id_slug ON public.sites USING btree (tenant_id, slug);
CREATE INDEX IF NOT EXISTS site_tenant_id_status ON public.sites USING btree (tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_sys_api_audit_logs_created_at ON public.sys_api_audit_logs USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_sys_api_audit_logs_tenant_api_created_at ON public.sys_api_audit_logs USING btree (tenant_id, api_module, api_operation, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_api_audit_logs_tenant_created_at ON public.sys_api_audit_logs USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_api_audit_logs_tenant_ip_created_at ON public.sys_api_audit_logs USING btree (tenant_id, ip_address, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_api_audit_logs_tenant_path_method_created_at ON public.sys_api_audit_logs USING btree (tenant_id, path, http_method, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_api_audit_logs_tenant_status_created_at ON public.sys_api_audit_logs USING btree (tenant_id, status_code, success, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_api_audit_logs_tenant_trace_id ON public.sys_api_audit_logs USING btree (tenant_id, trace_id);
CREATE INDEX IF NOT EXISTS idx_sys_api_audit_logs_tenant_user_created_at ON public.sys_api_audit_logs USING btree (tenant_id, user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_api_audit_logs_tenant_username_created_at ON public.sys_api_audit_logs USING btree (tenant_id, username, created_at);
CREATE UNIQUE INDEX IF NOT EXISTS uidx_sys_api_audit_logs_tenant_log_hash ON public.sys_api_audit_logs USING btree (tenant_id, log_hash);
CREATE UNIQUE INDEX IF NOT EXISTS uidx_sys_api_audit_logs_tenant_request_id ON public.sys_api_audit_logs USING btree (tenant_id, request_id);
CREATE INDEX IF NOT EXISTS idx_sys_api_res_created_at ON public.sys_apis USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_sys_api_res_created_by_created_at ON public.sys_apis USING btree (created_by, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_api_res_module ON public.sys_apis USING btree (module);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_api_res_module_path_method_scope ON public.sys_apis USING btree (module, path, method, scope);
CREATE INDEX IF NOT EXISTS idx_sys_api_res_path_method ON public.sys_apis USING btree (path, method);
CREATE INDEX IF NOT EXISTS idx_sys_api_res_scope ON public.sys_apis USING btree (scope);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_access_type ON public.sys_data_access_audit_logs USING btree (access_type);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_access_type_success_created_at ON public.sys_data_access_audit_logs USING btree (access_type, success, created_at);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_created_at ON public.sys_data_access_audit_logs USING btree (created_at);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_data_masked ON public.sys_data_access_audit_logs USING btree (data_masked);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_data_source_table_name_data_id ON public.sys_data_access_audit_logs USING btree (data_source, table_name, data_id);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_ip_address ON public.sys_data_access_audit_logs USING btree (ip_address);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_ip_address_created_at ON public.sys_data_access_audit_logs USING btree (ip_address, created_at);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_request_id ON public.sys_data_access_audit_logs USING btree (request_id);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_sql_digest ON public.sys_data_access_audit_logs USING btree (sql_digest);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_tenant_id ON public.sys_data_access_audit_logs USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_tenant_id_created_at ON public.sys_data_access_audit_logs USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_tenant_id_user_id_created_at ON public.sys_data_access_audit_logs USING btree (tenant_id, user_id, created_at);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_trace_id ON public.sys_data_access_audit_logs USING btree (trace_id);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_user_id ON public.sys_data_access_audit_logs USING btree (user_id);
CREATE INDEX IF NOT EXISTS dataaccessauditlog_username ON public.sys_data_access_audit_logs USING btree (username);
CREATE INDEX IF NOT EXISTS idx_sys_dict_entries_entry_value ON public.sys_dict_entries USING btree (entry_value);
CREATE INDEX IF NOT EXISTS idx_sys_dict_entries_numeric_value ON public.sys_dict_entries USING btree (numeric_value);
CREATE INDEX IF NOT EXISTS idx_sys_dict_entries_tenant_entry_value ON public.sys_dict_entries USING btree (tenant_id, entry_value);
CREATE INDEX IF NOT EXISTS idx_sys_dict_entries_tenant_id ON public.sys_dict_entries USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS idx_sys_dict_entries_tenant_type ON public.sys_dict_entries USING btree (tenant_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_sys_dict_entries_tenant_type_value ON public.sys_dict_entries USING btree (tenant_id, id, entry_value);
CREATE INDEX IF NOT EXISTS idx_sys_dict_entry_i18n_language_code ON public.sys_dict_entry_i18n USING btree (language_code);
CREATE INDEX IF NOT EXISTS idx_sys_dict_types_is_enabled ON public.sys_dict_types USING btree (is_enabled);
CREATE INDEX IF NOT EXISTS idx_sys_dict_types_sort_order ON public.sys_dict_types USING btree (sort_order);
CREATE INDEX IF NOT EXISTS idx_sys_dict_types_tenant_id ON public.sys_dict_types USING btree (tenant_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_sys_dict_types_tenant_type_code ON public.sys_dict_types USING btree (tenant_id, type_code);
CREATE INDEX IF NOT EXISTS idx_sys_languages_created_at ON public.sys_languages USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_sys_languages_is_default ON public.sys_languages USING btree (is_default);
CREATE INDEX IF NOT EXISTS idx_sys_languages_is_enabled ON public.sys_languages USING btree (is_enabled);
CREATE INDEX IF NOT EXISTS idx_sys_languages_language_code ON public.sys_languages USING btree (language_code);
CREATE INDEX IF NOT EXISTS idx_sys_languages_language_name ON public.sys_languages USING btree (language_name);
CREATE INDEX IF NOT EXISTS idx_sys_languages_native_name ON public.sys_languages USING btree (native_name);
CREATE INDEX IF NOT EXISTS idx_sys_languages_sort_order ON public.sys_languages USING btree (sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS uix_sys_languages_language_code ON public.sys_languages USING btree (language_code);
CREATE INDEX IF NOT EXISTS loginauditlog_action_type ON public.sys_login_audit_logs USING btree (action_type);
CREATE INDEX IF NOT EXISTS loginauditlog_created_at ON public.sys_login_audit_logs USING btree (created_at);
CREATE INDEX IF NOT EXISTS loginauditlog_ip_address ON public.sys_login_audit_logs USING btree (ip_address);
CREATE INDEX IF NOT EXISTS loginauditlog_ip_address_created_at ON public.sys_login_audit_logs USING btree (ip_address, created_at);
CREATE INDEX IF NOT EXISTS loginauditlog_request_id ON public.sys_login_audit_logs USING btree (request_id);
CREATE INDEX IF NOT EXISTS loginauditlog_session_id ON public.sys_login_audit_logs USING btree (session_id);
CREATE INDEX IF NOT EXISTS loginauditlog_status ON public.sys_login_audit_logs USING btree (status);
CREATE INDEX IF NOT EXISTS loginauditlog_tenant_id ON public.sys_login_audit_logs USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS loginauditlog_tenant_id_created_at ON public.sys_login_audit_logs USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS loginauditlog_user_id ON public.sys_login_audit_logs USING btree (user_id);
CREATE INDEX IF NOT EXISTS loginauditlog_username ON public.sys_login_audit_logs USING btree (username);
CREATE INDEX IF NOT EXISTS idx_sys_login_policy_tenant_type_method ON public.sys_login_policies USING btree (tenant_id, type, method);
CREATE INDEX IF NOT EXISTS idx_sys_login_policy_tenant_value ON public.sys_login_policies USING btree (tenant_id, value);
CREATE UNIQUE INDEX IF NOT EXISTS uidx_sys_login_policy_tenant_target_type_method ON public.sys_login_policies USING btree (tenant_id, target_id, type, method);
CREATE INDEX IF NOT EXISTS idx_mou_assigned_at ON public.sys_membership_org_units USING btree (assigned_at);
CREATE INDEX IF NOT EXISTS idx_mou_assigned_by ON public.sys_membership_org_units USING btree (assigned_by);
CREATE INDEX IF NOT EXISTS idx_mou_created_at ON public.sys_membership_org_units USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_mou_created_by_created_at ON public.sys_membership_org_units USING btree (created_by, created_at);
CREATE INDEX IF NOT EXISTS idx_mou_end_at ON public.sys_membership_org_units USING btree (end_at);
CREATE INDEX IF NOT EXISTS idx_mou_is_primary ON public.sys_membership_org_units USING btree (is_primary);
CREATE INDEX IF NOT EXISTS idx_mou_membership_id ON public.sys_membership_org_units USING btree (membership_id);
CREATE INDEX IF NOT EXISTS idx_mou_org_unit_id ON public.sys_membership_org_units USING btree (org_unit_id);
CREATE INDEX IF NOT EXISTS idx_mou_position_id ON public.sys_membership_org_units USING btree (position_id);
CREATE INDEX IF NOT EXISTS idx_mou_role_id ON public.sys_membership_org_units USING btree (role_id);
CREATE INDEX IF NOT EXISTS idx_mou_start_at ON public.sys_membership_org_units USING btree (start_at);
CREATE INDEX IF NOT EXISTS idx_mou_status ON public.sys_membership_org_units USING btree (status);
CREATE INDEX IF NOT EXISTS idx_mou_tenant_created_at ON public.sys_membership_org_units USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_mou_tenant_membership ON public.sys_membership_org_units USING btree (tenant_id, membership_id);
CREATE INDEX IF NOT EXISTS idx_mou_tenant_org_unit ON public.sys_membership_org_units USING btree (tenant_id, org_unit_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_mou_tenant_mem_org_pos_role ON public.sys_membership_org_units USING btree (tenant_id, membership_id, org_unit_id, position_id, role_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_mou_tenant_membership_primary ON public.sys_membership_org_units USING btree (tenant_id, membership_id, is_primary);
CREATE INDEX IF NOT EXISTS idx_mp_assigned_by ON public.sys_membership_positions USING btree (assigned_by);
CREATE INDEX IF NOT EXISTS idx_mp_is_primary ON public.sys_membership_positions USING btree (is_primary);
CREATE INDEX IF NOT EXISTS idx_mp_membership_id ON public.sys_membership_positions USING btree (membership_id);
CREATE INDEX IF NOT EXISTS idx_mp_position_id ON public.sys_membership_positions USING btree (position_id);
CREATE INDEX IF NOT EXISTS idx_mp_status ON public.sys_membership_positions USING btree (status);
CREATE INDEX IF NOT EXISTS idx_mp_tenant_assigned_by ON public.sys_membership_positions USING btree (tenant_id, assigned_by);
CREATE INDEX IF NOT EXISTS idx_mp_tenant_id ON public.sys_membership_positions USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS idx_mp_tenant_membership ON public.sys_membership_positions USING btree (tenant_id, membership_id);
CREATE INDEX IF NOT EXISTS idx_mp_tenant_membership_primary ON public.sys_membership_positions USING btree (tenant_id, membership_id, is_primary);
CREATE INDEX IF NOT EXISTS idx_mp_tenant_position ON public.sys_membership_positions USING btree (tenant_id, position_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_mp_tenant_membership_pos ON public.sys_membership_positions USING btree (tenant_id, membership_id, position_id);
CREATE INDEX IF NOT EXISTS idx_mr_assigned_by ON public.sys_membership_roles USING btree (assigned_by);
CREATE INDEX IF NOT EXISTS idx_mr_is_primary ON public.sys_membership_roles USING btree (is_primary);
CREATE INDEX IF NOT EXISTS idx_mr_membership_id ON public.sys_membership_roles USING btree (membership_id);
CREATE INDEX IF NOT EXISTS idx_mr_role_id ON public.sys_membership_roles USING btree (role_id);
CREATE INDEX IF NOT EXISTS idx_mr_status ON public.sys_membership_roles USING btree (status);
CREATE INDEX IF NOT EXISTS idx_mr_tenant_assigned_by ON public.sys_membership_roles USING btree (tenant_id, assigned_by);
CREATE INDEX IF NOT EXISTS idx_mr_tenant_id ON public.sys_membership_roles USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS idx_mr_tenant_membership ON public.sys_membership_roles USING btree (tenant_id, membership_id);
CREATE INDEX IF NOT EXISTS idx_mr_tenant_membership_primary ON public.sys_membership_roles USING btree (tenant_id, membership_id, is_primary);
CREATE INDEX IF NOT EXISTS idx_mr_tenant_role ON public.sys_membership_roles USING btree (tenant_id, role_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_mr_tenant_membership_role ON public.sys_membership_roles USING btree (tenant_id, membership_id, role_id);
CREATE INDEX IF NOT EXISTS idx_sys_membership_assigned_by ON public.sys_memberships USING btree (assigned_by);
CREATE INDEX IF NOT EXISTS idx_sys_membership_created_at ON public.sys_memberships USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_sys_membership_created_by_created_at ON public.sys_memberships USING btree (created_by, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_membership_end_at ON public.sys_memberships USING btree (end_at);
CREATE INDEX IF NOT EXISTS idx_sys_membership_org_unit_id ON public.sys_memberships USING btree (org_unit_id);
CREATE INDEX IF NOT EXISTS idx_sys_membership_position_id ON public.sys_memberships USING btree (position_id);
CREATE INDEX IF NOT EXISTS idx_sys_membership_role_id ON public.sys_memberships USING btree (role_id);
CREATE INDEX IF NOT EXISTS idx_sys_membership_start_at ON public.sys_memberships USING btree (start_at);
CREATE INDEX IF NOT EXISTS idx_sys_membership_status ON public.sys_memberships USING btree (status);
CREATE INDEX IF NOT EXISTS idx_sys_membership_tenant_id ON public.sys_memberships USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS idx_sys_membership_tenant_user_status_start_at ON public.sys_memberships USING btree (tenant_id, user_id, status, start_at);
CREATE INDEX IF NOT EXISTS idx_sys_membership_user_id ON public.sys_memberships USING btree (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_sys_membership_tenant_user ON public.sys_memberships USING btree (tenant_id, user_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_sys_membership_tenant_user_primary ON public.sys_memberships USING btree (tenant_id, user_id, is_primary);
CREATE INDEX IF NOT EXISTS idx_sys_menu_alias ON public.sys_menus USING btree (alias);
CREATE INDEX IF NOT EXISTS idx_sys_menu_component ON public.sys_menus USING btree (component);
CREATE INDEX IF NOT EXISTS idx_sys_menu_created_at ON public.sys_menus USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_sys_menu_created_by_created_at ON public.sys_menus USING btree (created_by, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_menu_parent ON public.sys_menus USING btree (parent_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_menu_parent_name ON public.sys_menus USING btree (parent_id, name);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_menu_parent_path ON public.sys_menus USING btree (parent_id, path);
CREATE INDEX IF NOT EXISTS idx_sys_menu_path ON public.sys_menus USING btree (path);
CREATE INDEX IF NOT EXISTS idx_sys_menu_status ON public.sys_menus USING btree (status);
CREATE INDEX IF NOT EXISTS idx_sys_menu_type ON public.sys_menus USING btree (type);
CREATE INDEX IF NOT EXISTS operationauditlog_action ON public.sys_operation_audit_logs USING btree (action);
CREATE INDEX IF NOT EXISTS operationauditlog_action_success_created_at ON public.sys_operation_audit_logs USING btree (action, success, created_at);
CREATE INDEX IF NOT EXISTS operationauditlog_created_at ON public.sys_operation_audit_logs USING btree (created_at);
CREATE INDEX IF NOT EXISTS operationauditlog_ip_address ON public.sys_operation_audit_logs USING btree (ip_address);
CREATE INDEX IF NOT EXISTS operationauditlog_ip_address_created_at ON public.sys_operation_audit_logs USING btree (ip_address, created_at);
CREATE INDEX IF NOT EXISTS operationauditlog_log_hash ON public.sys_operation_audit_logs USING btree (log_hash);
CREATE INDEX IF NOT EXISTS operationauditlog_request_id ON public.sys_operation_audit_logs USING btree (request_id);
CREATE INDEX IF NOT EXISTS operationauditlog_resource_type_resource_id ON public.sys_operation_audit_logs USING btree (resource_type, resource_id);
CREATE INDEX IF NOT EXISTS operationauditlog_sensitive_level ON public.sys_operation_audit_logs USING btree (sensitive_level);
CREATE INDEX IF NOT EXISTS operationauditlog_success ON public.sys_operation_audit_logs USING btree (success);
CREATE INDEX IF NOT EXISTS operationauditlog_tenant_id ON public.sys_operation_audit_logs USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS operationauditlog_tenant_id_created_at ON public.sys_operation_audit_logs USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS operationauditlog_tenant_id_user_id_created_at ON public.sys_operation_audit_logs USING btree (tenant_id, user_id, created_at);
CREATE INDEX IF NOT EXISTS operationauditlog_trace_id ON public.sys_operation_audit_logs USING btree (trace_id);
CREATE INDEX IF NOT EXISTS operationauditlog_user_id ON public.sys_operation_audit_logs USING btree (user_id);
CREATE INDEX IF NOT EXISTS operationauditlog_username ON public.sys_operation_audit_logs USING btree (username);
CREATE INDEX IF NOT EXISTS idx_org_contact_user_id ON public.sys_org_units USING btree (contact_user_id);
CREATE INDEX IF NOT EXISTS idx_org_created_at ON public.sys_org_units USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_org_created_by_created_at ON public.sys_org_units USING btree (created_by, created_at);
CREATE INDEX IF NOT EXISTS idx_org_end_at ON public.sys_org_units USING btree (end_at);
CREATE INDEX IF NOT EXISTS idx_org_external_id ON public.sys_org_units USING btree (external_id);
CREATE INDEX IF NOT EXISTS idx_org_is_legal_entity ON public.sys_org_units USING btree (is_legal_entity);
CREATE INDEX IF NOT EXISTS idx_org_leader_id ON public.sys_org_units USING btree (leader_id);
CREATE INDEX IF NOT EXISTS idx_org_parent_id ON public.sys_org_units USING btree (parent_id);
CREATE INDEX IF NOT EXISTS idx_org_start_at ON public.sys_org_units USING btree (start_at);
CREATE INDEX IF NOT EXISTS idx_org_tenant_created_at ON public.sys_org_units USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_org_tenant_id ON public.sys_org_units USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS idx_org_tenant_path ON public.sys_org_units USING btree (tenant_id, path);
CREATE INDEX IF NOT EXISTS idx_org_type ON public.sys_org_units USING btree (type);
CREATE UNIQUE INDEX IF NOT EXISTS uix_org_tenant_code ON public.sys_org_units USING btree (tenant_id, code);
CREATE UNIQUE INDEX IF NOT EXISTS uix_org_tenant_parent_name ON public.sys_org_units USING btree (tenant_id, parent_id, name);
CREATE UNIQUE INDEX IF NOT EXISTS uix_org_tenant_parent_path ON public.sys_org_units USING btree (tenant_id, parent_id, path);
CREATE INDEX IF NOT EXISTS idx_perm_api_api_id ON public.sys_permission_apis USING btree (api_id);
CREATE INDEX IF NOT EXISTS idx_perm_api_permission_id ON public.sys_permission_apis USING btree (permission_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_perm_api_permission_api_id ON public.sys_permission_apis USING btree (permission_id, api_id);
CREATE INDEX IF NOT EXISTS idx_permission_audit_target ON public.sys_permission_audit_logs USING btree (target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_permission_audit_tenant_action_created_at ON public.sys_permission_audit_logs USING btree (tenant_id, action, created_at);
CREATE INDEX IF NOT EXISTS idx_permission_audit_tenant_created_at ON public.sys_permission_audit_logs USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_permission_audit_tenant_ip ON public.sys_permission_audit_logs USING btree (tenant_id, ip_address);
CREATE INDEX IF NOT EXISTS idx_permission_audit_tenant_operator_created_at ON public.sys_permission_audit_logs USING btree (tenant_id, operator_id, created_at);
CREATE INDEX IF NOT EXISTS idx_permission_audit_tenant_target_created_at ON public.sys_permission_audit_logs USING btree (tenant_id, target_type, target_id, created_at);
CREATE INDEX IF NOT EXISTS idx_perm_group_module ON public.sys_permission_groups USING btree (module);
CREATE INDEX IF NOT EXISTS idx_perm_group_name ON public.sys_permission_groups USING btree (name);
CREATE INDEX IF NOT EXISTS idx_perm_group_parent_id ON public.sys_permission_groups USING btree (parent_id);
CREATE INDEX IF NOT EXISTS idx_perm_menu_menu_id ON public.sys_permission_menus USING btree (menu_id);
CREATE INDEX IF NOT EXISTS idx_perm_menu_permission_id ON public.sys_permission_menus USING btree (permission_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_perm_menu_permission_menu_id ON public.sys_permission_menus USING btree (permission_id, menu_id);
CREATE INDEX IF NOT EXISTS idx_perm_policy_engine ON public.sys_permission_policies USING btree (policy_engine);
CREATE INDEX IF NOT EXISTS idx_perm_policy_perm ON public.sys_permission_policies USING btree (permission_id);
CREATE INDEX IF NOT EXISTS idx_perm_policy_perm_version ON public.sys_permission_policies USING btree (permission_id, version);
CREATE INDEX IF NOT EXISTS idx_perm_policy_version ON public.sys_permission_policies USING btree (version);
CREATE INDEX IF NOT EXISTS idx_perm_group_id ON public.sys_permissions USING btree (group_id);
CREATE INDEX IF NOT EXISTS idx_perm_name ON public.sys_permissions USING btree (name);
CREATE UNIQUE INDEX IF NOT EXISTS uix_perm_code ON public.sys_permissions USING btree (code);
CREATE INDEX IF NOT EXISTS idx_policy_eval_ip_address_created_at ON public.sys_policy_evaluation_logs USING btree (ip_address, created_at);
CREATE INDEX IF NOT EXISTS idx_policy_eval_log_hash ON public.sys_policy_evaluation_logs USING btree (log_hash);
CREATE INDEX IF NOT EXISTS idx_policy_eval_request_method ON public.sys_policy_evaluation_logs USING btree (request_method);
CREATE INDEX IF NOT EXISTS idx_policy_eval_request_path ON public.sys_policy_evaluation_logs USING btree (request_path);
CREATE INDEX IF NOT EXISTS idx_policy_eval_tenant_created_at ON public.sys_policy_evaluation_logs USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_policy_eval_tenant_membership_created_at ON public.sys_policy_evaluation_logs USING btree (tenant_id, membership_id, created_at);
CREATE INDEX IF NOT EXISTS idx_policy_eval_tenant_permission_result_created_at ON public.sys_policy_evaluation_logs USING btree (tenant_id, permission_id, result, created_at);
CREATE INDEX IF NOT EXISTS idx_policy_eval_tenant_policy_created_at ON public.sys_policy_evaluation_logs USING btree (tenant_id, policy_id, created_at);
CREATE INDEX IF NOT EXISTS idx_policy_eval_tenant_user_permission_created_at ON public.sys_policy_evaluation_logs USING btree (tenant_id, user_id, permission_id, created_at);
CREATE INDEX IF NOT EXISTS idx_policy_eval_trace_id ON public.sys_policy_evaluation_logs USING btree (trace_id);
CREATE INDEX IF NOT EXISTS idx_sys_positions_code ON public.sys_positions USING btree (code);
CREATE INDEX IF NOT EXISTS idx_sys_positions_created_at ON public.sys_positions USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_sys_positions_end_at ON public.sys_positions USING btree (end_at);
CREATE INDEX IF NOT EXISTS idx_sys_positions_headcount ON public.sys_positions USING btree (headcount);
CREATE INDEX IF NOT EXISTS idx_sys_positions_is_key ON public.sys_positions USING btree (is_key_position);
CREATE INDEX IF NOT EXISTS idx_sys_positions_level ON public.sys_positions USING btree (level);
CREATE INDEX IF NOT EXISTS idx_sys_positions_name ON public.sys_positions USING btree (name);
CREATE INDEX IF NOT EXISTS idx_sys_positions_org_unit_id ON public.sys_positions USING btree (org_unit_id);
CREATE INDEX IF NOT EXISTS idx_sys_positions_reports_to ON public.sys_positions USING btree (reports_to_position_id);
CREATE INDEX IF NOT EXISTS idx_sys_positions_sort_order ON public.sys_positions USING btree (sort_order);
CREATE INDEX IF NOT EXISTS idx_sys_positions_start_at ON public.sys_positions USING btree (start_at);
CREATE INDEX IF NOT EXISTS idx_sys_positions_status ON public.sys_positions USING btree (status);
CREATE INDEX IF NOT EXISTS idx_sys_positions_tenant_id ON public.sys_positions USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS idx_sys_positions_tenant_is_key ON public.sys_positions USING btree (tenant_id, is_key_position);
CREATE INDEX IF NOT EXISTS idx_sys_positions_tenant_name ON public.sys_positions USING btree (tenant_id, name);
CREATE INDEX IF NOT EXISTS idx_sys_positions_tenant_org_unit_id ON public.sys_positions USING btree (tenant_id, org_unit_id);
CREATE INDEX IF NOT EXISTS idx_sys_positions_tenant_reports_to ON public.sys_positions USING btree (tenant_id, reports_to_position_id);
CREATE INDEX IF NOT EXISTS idx_sys_positions_tenant_type ON public.sys_positions USING btree (tenant_id, type);
CREATE INDEX IF NOT EXISTS idx_sys_positions_type ON public.sys_positions USING btree (type);
CREATE UNIQUE INDEX IF NOT EXISTS uix_sys_positions_tenant_code ON public.sys_positions USING btree (tenant_id, code);
CREATE INDEX IF NOT EXISTS idx_role_metadata_last_synced_at ON public.sys_role_metadata USING btree (tenant_id, last_synced_at);
CREATE INDEX IF NOT EXISTS idx_role_metadata_last_synced_version ON public.sys_role_metadata USING btree (tenant_id, last_synced_version);
CREATE INDEX IF NOT EXISTS idx_role_metadata_scope_role ON public.sys_role_metadata USING btree (tenant_id, scope, role_id);
CREATE INDEX IF NOT EXISTS idx_role_metadata_template_lookup ON public.sys_role_metadata USING btree (tenant_id, is_template, template_for);
CREATE UNIQUE INDEX IF NOT EXISTS idx_role_metadata_tenant_role ON public.sys_role_metadata USING btree (tenant_id, role_id);
CREATE INDEX IF NOT EXISTS idx_rp_created_at ON public.sys_role_permissions USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_rp_created_by ON public.sys_role_permissions USING btree (created_by);
CREATE INDEX IF NOT EXISTS idx_rp_permission_id ON public.sys_role_permissions USING btree (permission_id);
CREATE INDEX IF NOT EXISTS idx_rp_role_id ON public.sys_role_permissions USING btree (role_id);
CREATE INDEX IF NOT EXISTS idx_rp_tenant_id ON public.sys_role_permissions USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS idx_rp_tenant_permission ON public.sys_role_permissions USING btree (tenant_id, permission_id);
CREATE INDEX IF NOT EXISTS idx_rp_tenant_role ON public.sys_role_permissions USING btree (tenant_id, role_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_rp_role_permission ON public.sys_role_permissions USING btree (role_id, permission_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_rp_tenant_role_permission ON public.sys_role_permissions USING btree (tenant_id, role_id, permission_id);
CREATE INDEX IF NOT EXISTS idx_sys_roles_code ON public.sys_roles USING btree (code);
CREATE INDEX IF NOT EXISTS idx_sys_roles_created_at ON public.sys_roles USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_sys_roles_created_by ON public.sys_roles USING btree (created_by);
CREATE INDEX IF NOT EXISTS idx_sys_roles_is_protected ON public.sys_roles USING btree (is_protected);
CREATE INDEX IF NOT EXISTS idx_sys_roles_name ON public.sys_roles USING btree (name);
CREATE INDEX IF NOT EXISTS idx_sys_roles_sort_order ON public.sys_roles USING btree (sort_order);
CREATE INDEX IF NOT EXISTS idx_sys_roles_status ON public.sys_roles USING btree (status);
CREATE INDEX IF NOT EXISTS idx_sys_roles_tenant_id ON public.sys_roles USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS idx_sys_roles_tenant_name ON public.sys_roles USING btree (tenant_id, name);
CREATE UNIQUE INDEX IF NOT EXISTS uix_sys_roles_tenant_code ON public.sys_roles USING btree (tenant_id, code);
CREATE INDEX IF NOT EXISTS idx_sys_task_tenant_created_at ON public.sys_tasks USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_task_tenant_created_by_created_at ON public.sys_tasks USING btree (tenant_id, created_by, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_task_tenant_enable_created_at ON public.sys_tasks USING btree (tenant_id, enable, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_task_tenant_type ON public.sys_tasks USING btree (tenant_id, type);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_task_tenant_type_name ON public.sys_tasks USING btree (tenant_id, type_name);
CREATE INDEX IF NOT EXISTS idx_sys_tenant_admin_user_id ON public.sys_tenants USING btree (admin_user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_tenant_code ON public.sys_tenants USING btree (code);
CREATE INDEX IF NOT EXISTS idx_sys_tenant_created_at ON public.sys_tenants USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_sys_tenant_created_by_created_at ON public.sys_tenants USING btree (created_by, created_at);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_tenant_domain ON public.sys_tenants USING btree (domain);
CREATE INDEX IF NOT EXISTS idx_sys_tenant_expired_at ON public.sys_tenants USING btree (expired_at);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_tenant_name ON public.sys_tenants USING btree (name);
CREATE INDEX IF NOT EXISTS idx_sys_tenant_status_audit_status ON public.sys_tenants USING btree (status, audit_status);
CREATE INDEX IF NOT EXISTS idx_sys_tenant_subscription_at ON public.sys_tenants USING btree (subscription_at);
CREATE INDEX IF NOT EXISTS idx_sys_tenant_type_expired_at ON public.sys_tenants USING btree (type, expired_at);
CREATE INDEX IF NOT EXISTS idx_sys_user_cred_tenant_activate_expires_at ON public.sys_user_credentials USING btree (tenant_id, activate_token_expires_at);
CREATE INDEX IF NOT EXISTS idx_sys_user_cred_tenant_identifier ON public.sys_user_credentials USING btree (tenant_id, identifier);
CREATE INDEX IF NOT EXISTS idx_sys_user_cred_tenant_is_primary ON public.sys_user_credentials USING btree (tenant_id, is_primary);
CREATE INDEX IF NOT EXISTS idx_sys_user_cred_tenant_provider ON public.sys_user_credentials USING btree (tenant_id, provider);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_user_cred_tenant_provider_account ON public.sys_user_credentials USING btree (tenant_id, provider, provider_account_id);
CREATE INDEX IF NOT EXISTS idx_sys_user_cred_tenant_status_created_at ON public.sys_user_credentials USING btree (tenant_id, status, created_at);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_user_cred_tenant_uid_identity_identifier ON public.sys_user_credentials USING btree (tenant_id, user_id, identity_type, identifier);
CREATE INDEX IF NOT EXISTS idx_sys_user_cred_tenant_user_id ON public.sys_user_credentials USING btree (tenant_id, user_id);
CREATE INDEX IF NOT EXISTS idx_uou_assigned_at ON public.sys_user_org_units USING btree (assigned_at);
CREATE INDEX IF NOT EXISTS idx_uou_assigned_by ON public.sys_user_org_units USING btree (assigned_by);
CREATE INDEX IF NOT EXISTS idx_uou_created_at ON public.sys_user_org_units USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_uou_created_by_created_at ON public.sys_user_org_units USING btree (created_by, created_at);
CREATE INDEX IF NOT EXISTS idx_uou_end_at ON public.sys_user_org_units USING btree (end_at);
CREATE INDEX IF NOT EXISTS idx_uou_is_primary ON public.sys_user_org_units USING btree (is_primary);
CREATE INDEX IF NOT EXISTS idx_uou_org_unit_id ON public.sys_user_org_units USING btree (org_unit_id);
CREATE INDEX IF NOT EXISTS idx_uou_position_id ON public.sys_user_org_units USING btree (position_id);
CREATE INDEX IF NOT EXISTS idx_uou_start_at ON public.sys_user_org_units USING btree (start_at);
CREATE INDEX IF NOT EXISTS idx_uou_status ON public.sys_user_org_units USING btree (status);
CREATE INDEX IF NOT EXISTS idx_uou_tenant_created_at ON public.sys_user_org_units USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_uou_tenant_org_unit ON public.sys_user_org_units USING btree (tenant_id, org_unit_id);
CREATE INDEX IF NOT EXISTS idx_uou_tenant_user ON public.sys_user_org_units USING btree (tenant_id, user_id);
CREATE INDEX IF NOT EXISTS idx_uou_user_id ON public.sys_user_org_units USING btree (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_uou_tenant_user_org_pos ON public.sys_user_org_units USING btree (tenant_id, user_id, org_unit_id, position_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_uou_tenant_user_primary ON public.sys_user_org_units USING btree (tenant_id, user_id, is_primary);
CREATE INDEX IF NOT EXISTS idx_up_assigned_by ON public.sys_user_positions USING btree (assigned_by);
CREATE INDEX IF NOT EXISTS idx_up_is_primary ON public.sys_user_positions USING btree (is_primary);
CREATE INDEX IF NOT EXISTS idx_up_position_id ON public.sys_user_positions USING btree (position_id);
CREATE INDEX IF NOT EXISTS idx_up_status ON public.sys_user_positions USING btree (status);
CREATE INDEX IF NOT EXISTS idx_up_tenant_assigned_by ON public.sys_user_positions USING btree (tenant_id, assigned_by);
CREATE INDEX IF NOT EXISTS idx_up_tenant_id ON public.sys_user_positions USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS idx_up_tenant_position ON public.sys_user_positions USING btree (tenant_id, position_id);
CREATE INDEX IF NOT EXISTS idx_up_tenant_user ON public.sys_user_positions USING btree (tenant_id, user_id);
CREATE INDEX IF NOT EXISTS idx_up_tenant_user_primary ON public.sys_user_positions USING btree (tenant_id, user_id, is_primary);
CREATE INDEX IF NOT EXISTS idx_up_user_id ON public.sys_user_positions USING btree (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_up_tenant_user_pos ON public.sys_user_positions USING btree (tenant_id, user_id, position_id);
CREATE INDEX IF NOT EXISTS idx_ur_assigned_by ON public.sys_user_roles USING btree (assigned_by);
CREATE INDEX IF NOT EXISTS idx_ur_is_primary ON public.sys_user_roles USING btree (is_primary);
CREATE INDEX IF NOT EXISTS idx_ur_role_id ON public.sys_user_roles USING btree (role_id);
CREATE INDEX IF NOT EXISTS idx_ur_status ON public.sys_user_roles USING btree (status);
CREATE INDEX IF NOT EXISTS idx_ur_tenant_assigned_by ON public.sys_user_roles USING btree (tenant_id, assigned_by);
CREATE INDEX IF NOT EXISTS idx_ur_tenant_id ON public.sys_user_roles USING btree (tenant_id);
CREATE INDEX IF NOT EXISTS idx_ur_tenant_role ON public.sys_user_roles USING btree (tenant_id, role_id);
CREATE INDEX IF NOT EXISTS idx_ur_tenant_user ON public.sys_user_roles USING btree (tenant_id, user_id);
CREATE INDEX IF NOT EXISTS idx_ur_tenant_user_primary ON public.sys_user_roles USING btree (tenant_id, user_id, is_primary);
CREATE INDEX IF NOT EXISTS idx_ur_user_id ON public.sys_user_roles USING btree (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS uix_ur_tenant_user_role ON public.sys_user_roles USING btree (tenant_id, user_id, role_id);
CREATE INDEX IF NOT EXISTS idx_sys_user_tenant_created_at ON public.sys_users USING btree (tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_sys_user_tenant_created_by ON public.sys_users USING btree (tenant_id, created_by);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_user_tenant_email ON public.sys_users USING btree (tenant_id, email);
CREATE INDEX IF NOT EXISTS idx_sys_user_tenant_last_login_at ON public.sys_users USING btree (tenant_id, last_login_at);
CREATE INDEX IF NOT EXISTS idx_sys_user_tenant_last_login_ip ON public.sys_users USING btree (tenant_id, last_login_ip);
CREATE INDEX IF NOT EXISTS idx_sys_user_tenant_mobile ON public.sys_users USING btree (tenant_id, mobile);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_user_tenant_username ON public.sys_users USING btree (tenant_id, username);
CREATE INDEX IF NOT EXISTS tagtranslation_full_path ON public.tag_translations USING btree (full_path);
CREATE INDEX IF NOT EXISTS tagtranslation_language_code ON public.tag_translations USING btree (language_code);
CREATE INDEX IF NOT EXISTS tagtranslation_language_code_slug ON public.tag_translations USING btree (language_code, slug);
CREATE INDEX IF NOT EXISTS tagtranslation_slug ON public.tag_translations USING btree (slug);
CREATE INDEX IF NOT EXISTS tagtranslation_tag_id ON public.tag_translations USING btree (tag_id);
CREATE INDEX IF NOT EXISTS tagtranslation_tag_id_language_code ON public.tag_translations USING btree (tag_id, language_code);
CREATE INDEX IF NOT EXISTS tag_group ON public.tags USING btree ("group");
CREATE INDEX IF NOT EXISTS tag_is_featured ON public.tags USING btree (is_featured);
CREATE INDEX IF NOT EXISTS tag_status ON public.tags USING btree (status);
CREATE INDEX IF NOT EXISTS tag_status_group ON public.tags USING btree (status, "group");
CREATE INDEX IF NOT EXISTS tag_status_is_featured ON public.tags USING btree (status, is_featured);
