//! Generated sea-orm entity modules — one per table of the
//! golden DDL (sql/schema.sql). DO NOT EDIT; regenerate via
//! scripts/gen-entities.py when the schema is re-dumped.

#![allow(clippy::all)]
#![allow(missing_docs)]

pub mod categories {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "categories")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub sort_order: Option<i64>,
        pub path: Option<String>,
        pub tenant_id: Option<i64>,
        pub status: Option<String>,
        pub is_nav: Option<bool>,
        pub icon: Option<String>,
        pub code: Option<String>,
        pub thumbnail: Option<String>,
        pub post_count: Option<i64>,
        pub direct_post_count: Option<i64>,
        pub depth: Option<i32>,
        pub custom_fields: Option<Json>,
        pub content_model_id: Option<i64>,
        pub parent_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod category_translations {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "category_translations")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub seo: Option<Json>,
        pub tenant_id: Option<i64>,
        pub category_id: Option<i64>,
        pub language_code: Option<String>,
        pub name: Option<String>,
        pub slug: Option<String>,
        pub description: Option<String>,
        pub cover_image: Option<String>,
        pub full_path: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod comment_likes {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "comment_likes")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub user_id: Option<i64>,
        pub comment_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod comments {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "comments")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub content_type: Option<String>,
        pub object_id: Option<i64>,
        pub content: Option<String>,
        pub author_id: Option<i64>,
        pub author_name: Option<String>,
        pub author_email: Option<String>,
        pub author_url: Option<String>,
        pub author_type: Option<String>,
        pub status: Option<String>,
        pub ip_address: Option<String>,
        pub location: Option<String>,
        pub user_agent: Option<String>,
        pub detected_language: Option<String>,
        pub is_spam: Option<bool>,
        pub is_sticky: Option<bool>,
        pub reply_to_id: Option<i64>,
        pub parent_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod content_model_translations {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "content_model_translations")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub content_model_id: Option<i64>,
        pub language_code: Option<String>,
        pub name: Option<String>,
        pub description: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod content_models {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "content_models")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub sort_order: Option<i64>,
        pub tenant_id: Option<i64>,
        pub name: Option<String>,
        pub code: Option<String>,
        pub description: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod field_definition_translations {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "field_definition_translations")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub field_definition_id: Option<i64>,
        pub language_code: Option<String>,
        pub label: Option<String>,
        pub description: Option<String>,
        pub placeholder: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod field_definitions {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "field_definitions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub sort_order: Option<i64>,
        pub tenant_id: Option<i64>,
        pub content_model_id: Option<i64>,
        pub name: Option<String>,
        pub r#type: Option<String>,
        pub label: Option<String>,
        pub description: Option<String>,
        pub placeholder: Option<String>,
        pub is_required: Option<bool>,
        pub validation_regex: Option<String>,
        pub options: Option<Json>,
        pub relation_config: Option<Json>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod files {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "files")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub remark: Option<String>,
        pub tenant_id: Option<i64>,
        pub provider: Option<String>,
        pub bucket_name: Option<String>,
        pub file_directory: Option<String>,
        pub file_guid: Option<String>,
        pub save_file_name: Option<String>,
        pub file_name: Option<String>,
        pub extension: Option<String>,
        pub size: Option<i64>,
        pub size_format: Option<String>,
        pub link_url: Option<String>,
        pub content_hash: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod interaction_counters {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "interaction_counters")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub target_type: Option<i16>,
        pub target_id: Option<i64>,
        pub metric: Option<i16>,
        pub count: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod internal_message_categories {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "internal_message_categories")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub is_enabled: Option<bool>,
        pub sort_order: Option<i64>,
        pub path: Option<String>,
        pub remark: Option<String>,
        pub tenant_id: Option<i64>,
        pub name: Option<String>,
        pub code: Option<String>,
        pub icon_url: Option<String>,
        pub depth: Option<i32>,
        pub parent_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod internal_message_recipients {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "internal_message_recipients")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub message_id: Option<i64>,
        pub recipient_user_id: Option<i64>,
        pub status: Option<String>,
        pub received_at: Option<DateTimeWithTimeZone>,
        pub read_at: Option<DateTimeWithTimeZone>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod internal_messages {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "internal_messages")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub title: Option<String>,
        pub content: Option<String>,
        pub sender_id: i64,
        pub category_id: Option<i64>,
        pub status: Option<String>,
        pub r#type: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod media_assets {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "media_assets")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub filename: Option<String>,
        pub r#type: Option<String>,
        pub mime_type: Option<String>,
        pub size: Option<i64>,
        pub storage_path: Option<String>,
        pub url: Option<String>,
        pub width: Option<i64>,
        pub height: Option<i64>,
        pub duration: Option<i64>,
        pub alt_text: Option<String>,
        pub title: Option<String>,
        pub caption: Option<String>,
        pub processing_status: Option<String>,
        pub processing_error: Option<String>,
        pub file_hash: Option<String>,
        pub folder_id: Option<i64>,
        pub file_id: Option<i64>,
        pub reference_count: Option<i64>,
        pub is_private: Option<bool>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod media_variants {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "media_variants")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub media_id: i64,
        pub file_id: i64,
        pub variant_name: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod navigation_items {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "navigation_items")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub sort_order: Option<i64>,
        pub tenant_id: Option<i64>,
        pub link_type: Option<String>,
        pub navigation_id: Option<i64>,
        pub title: Option<String>,
        pub url: Option<String>,
        pub object_id: Option<i64>,
        pub icon: Option<String>,
        pub description: Option<String>,
        pub is_open_new_tab: Option<bool>,
        pub is_invalid: Option<bool>,
        pub required_permission: Option<String>,
        pub parent_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod navigations {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "navigations")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub name: Option<String>,
        pub location: Option<String>,
        pub locale: Option<String>,
        pub is_active: Option<bool>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod page_translations {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "page_translations")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub seo: Option<Json>,
        pub tenant_id: Option<i64>,
        pub page_id: Option<i64>,
        pub language_code: Option<String>,
        pub title: Option<String>,
        pub slug: Option<String>,
        pub cover_image: Option<String>,
        pub full_path: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod pages {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "pages")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub sort_order: Option<i64>,
        pub path: Option<String>,
        pub editor_type: Option<String>,
        pub tenant_id: Option<i64>,
        pub status: Option<String>,
        pub r#type: Option<String>,
        pub slug: Option<String>,
        pub author_id: Option<i64>,
        pub author_name: Option<String>,
        pub disallow_comment: Option<bool>,
        pub redirect_url: Option<String>,
        pub show_in_navigation: Option<bool>,
        pub template: Option<String>,
        pub is_custom_template: Option<bool>,
        pub thumbnail: Option<String>,
        pub custom_fields: Option<Json>,
        pub content_model_id: Option<i64>,
        pub depth: Option<i32>,
        pub parent_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod post_categories {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "post_categories")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub post_id: i64,
        pub category_id: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod post_likes {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "post_likes")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub user_id: Option<i64>,
        pub post_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod post_tags {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "post_tags")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub post_id: i64,
        pub tag_id: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod post_translations {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "post_translations")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub seo: Option<Json>,
        pub tenant_id: Option<i64>,
        pub post_id: Option<i64>,
        pub language_code: Option<String>,
        pub title: Option<String>,
        pub slug: Option<String>,
        pub summary: Option<String>,
        pub content: Option<String>,
        pub original_content: Option<String>,
        pub full_path: Option<String>,
        pub word_count: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod post_watches {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "post_watches")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub user_id: Option<i64>,
        pub post_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod posts {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "posts")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub sort_order: Option<i64>,
        pub editor_type: Option<String>,
        pub tenant_id: Option<i64>,
        pub status: Option<String>,
        pub code: Option<String>,
        pub disallow_comment: Option<bool>,
        pub in_progress: Option<bool>,
        pub auto_summary: Option<bool>,
        pub is_featured: Option<bool>,
        pub author_id: Option<i64>,
        pub author_name: Option<String>,
        pub thumbnail: Option<String>,
        pub password_hash: Option<String>,
        pub custom_fields: Option<Json>,
        pub publish_time: Option<DateTimeWithTimeZone>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod section_translations {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "section_translations")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub section_id: Option<i64>,
        pub language_code: Option<String>,
        pub content: Option<Json>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sections {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sections")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub sort_order: Option<i64>,
        pub tenant_id: Option<i64>,
        pub page_id: Option<i64>,
        pub r#type: Option<String>,
        pub name: Option<String>,
        pub config: Option<Json>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod site_settings {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "site_settings")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub site_id: Option<i64>,
        pub locale: Option<String>,
        pub group: Option<String>,
        pub key: Option<String>,
        pub value: Option<String>,
        pub r#type: Option<String>,
        pub label: Option<String>,
        pub description: Option<String>,
        pub placeholder: Option<String>,
        pub options: Option<Json>,
        pub is_required: Option<bool>,
        pub validation_regex: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sites {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sites")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub name: Option<String>,
        pub slug: Option<String>,
        pub domain: Option<String>,
        pub alternate_domains: Option<Json>,
        pub is_default: Option<bool>,
        pub status: Option<String>,
        pub default_locale: Option<String>,
        pub template: Option<String>,
        pub theme: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_api_audit_logs {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_api_audit_logs")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub user_id: Option<i64>,
        pub username: Option<String>,
        pub ip_address: Option<String>,
        pub geo_location: Option<Json>,
        pub device_info: Option<Json>,
        pub referer: Option<String>,
        pub app_version: Option<String>,
        pub http_method: Option<String>,
        pub path: Option<String>,
        pub request_uri: Option<String>,
        pub api_module: Option<String>,
        pub api_operation: Option<String>,
        pub api_description: Option<String>,
        pub request_id: Option<String>,
        pub trace_id: Option<String>,
        pub span_id: Option<String>,
        pub latency_ms: Option<i64>,
        pub success: Option<bool>,
        pub status_code: Option<i64>,
        pub reason: Option<String>,
        pub request_header: Option<String>,
        pub request_body: Option<String>,
        pub response: Option<String>,
        pub log_hash: Option<String>,
        pub signature: Option<Vec<u8>>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_apis {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_apis")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub status: String,
        pub tenant_id: Option<i64>,
        pub description: Option<String>,
        pub module: Option<String>,
        pub module_description: Option<String>,
        pub operation: Option<String>,
        pub path: Option<String>,
        pub method: Option<String>,
        pub scope: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_data_access_audit_logs {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_data_access_audit_logs")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub user_id: Option<i64>,
        pub username: Option<String>,
        pub ip_address: Option<String>,
        pub geo_location: Option<Json>,
        pub device_info: Option<Json>,
        pub request_id: Option<String>,
        pub trace_id: Option<String>,
        pub data_source: Option<String>,
        pub table_name: Option<String>,
        pub data_id: Option<String>,
        pub access_type: Option<String>,
        pub sql_digest: Option<String>,
        pub sql_text: Option<String>,
        pub affected_rows: Option<i64>,
        pub latency_ms: Option<i64>,
        pub success: Option<bool>,
        pub sensitive_level: Option<String>,
        pub data_masked: Option<bool>,
        pub masking_rules: Option<String>,
        pub business_purpose: Option<String>,
        pub data_category: Option<String>,
        pub db_user: Option<String>,
        pub log_hash: Option<String>,
        pub signature: Option<Vec<u8>>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_dict_entries {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_dict_entries")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub sort_order: Option<i64>,
        pub is_enabled: Option<bool>,
        pub tenant_id: Option<i64>,
        pub entry_value: String,
        pub numeric_value: Option<i32>,
        pub type_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_dict_types {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_dict_types")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub is_enabled: Option<bool>,
        pub sort_order: Option<i64>,
        pub tenant_id: Option<i64>,
        pub type_code: Option<String>,
        pub type_name: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_languages {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_languages")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub sort_order: Option<i64>,
        pub is_enabled: Option<bool>,
        pub language_code: Option<String>,
        pub language_name: Option<String>,
        pub native_name: Option<String>,
        pub is_default: Option<bool>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_login_audit_logs {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_login_audit_logs")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub user_id: Option<i64>,
        pub username: Option<String>,
        pub ip_address: Option<String>,
        pub geo_location: Option<Json>,
        pub session_id: Option<String>,
        pub device_info: Option<Json>,
        pub request_id: Option<String>,
        pub trace_id: Option<String>,
        pub action_type: Option<String>,
        pub status: Option<String>,
        pub login_method: Option<String>,
        pub failure_reason: Option<String>,
        pub mfa_status: Option<String>,
        pub risk_score: Option<i64>,
        pub risk_level: Option<String>,
        pub risk_factors: Option<Json>,
        pub log_hash: Option<String>,
        pub signature: Option<Vec<u8>>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_login_policies {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_login_policies")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub target_id: Option<i64>,
        pub value: Option<String>,
        pub reason: Option<String>,
        pub r#type: Option<String>,
        pub method: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_membership_org_units {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_membership_org_units")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub remark: Option<String>,
        pub membership_id: i64,
        pub org_unit_id: i64,
        pub position_id: Option<i64>,
        pub role_id: Option<i64>,
        pub start_at: Option<DateTimeWithTimeZone>,
        pub end_at: Option<DateTimeWithTimeZone>,
        pub assigned_at: Option<DateTimeWithTimeZone>,
        pub assigned_by: Option<i64>,
        pub is_primary: bool,
        pub status: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_membership_positions {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_membership_positions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub remark: Option<String>,
        pub membership_id: i64,
        pub position_id: i64,
        pub is_primary: Option<bool>,
        pub start_at: Option<DateTimeWithTimeZone>,
        pub end_at: Option<DateTimeWithTimeZone>,
        pub assigned_at: Option<DateTimeWithTimeZone>,
        pub assigned_by: Option<i64>,
        pub status: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_membership_roles {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_membership_roles")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub membership_id: i64,
        pub role_id: i64,
        pub start_at: Option<DateTimeWithTimeZone>,
        pub end_at: Option<DateTimeWithTimeZone>,
        pub assigned_at: Option<DateTimeWithTimeZone>,
        pub assigned_by: Option<i64>,
        pub is_primary: bool,
        pub status: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_memberships {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_memberships")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub remark: Option<String>,
        pub user_id: i64,
        pub org_unit_id: Option<i64>,
        pub position_id: Option<i64>,
        pub role_id: Option<i64>,
        pub is_primary: bool,
        pub start_at: Option<DateTimeWithTimeZone>,
        pub end_at: Option<DateTimeWithTimeZone>,
        pub assigned_at: Option<DateTimeWithTimeZone>,
        pub assigned_by: Option<i64>,
        pub joined_at: Option<DateTimeWithTimeZone>,
        pub status: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_menus {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_menus")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub remark: Option<String>,
        pub status: String,
        pub tenant_id: Option<i64>,
        pub r#type: Option<String>,
        pub path: Option<String>,
        pub redirect: Option<String>,
        pub alias: Option<String>,
        pub name: Option<String>,
        pub component: Option<String>,
        pub meta: Option<Json>,
        pub parent_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_operation_audit_logs {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_operation_audit_logs")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub user_id: Option<i64>,
        pub username: Option<String>,
        pub resource_type: Option<String>,
        pub resource_id: Option<String>,
        pub action: Option<String>,
        pub before_data: Option<Json>,
        pub after_data: Option<Json>,
        pub sensitive_level: Option<String>,
        pub request_id: Option<String>,
        pub trace_id: Option<String>,
        pub success: Option<bool>,
        pub failure_reason: Option<String>,
        pub ip_address: Option<String>,
        pub geo_location: Option<Json>,
        pub device_info: Option<Json>,
        pub log_hash: Option<String>,
        pub signature: Option<Vec<u8>>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_org_units {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_org_units")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub status: String,
        pub sort_order: Option<i64>,
        pub tenant_id: Option<i64>,
        pub remark: Option<String>,
        pub description: Option<String>,
        pub path: Option<String>,
        pub name: String,
        pub code: Option<String>,
        pub leader_id: Option<i64>,
        pub r#type: String,
        pub business_scopes: Option<Json>,
        pub external_id: Option<String>,
        pub is_legal_entity: Option<bool>,
        pub registration_number: Option<String>,
        pub tax_id: Option<String>,
        pub legal_entity_org_id: Option<i64>,
        pub address: Option<String>,
        pub phone: Option<String>,
        pub email: Option<String>,
        pub timezone: Option<String>,
        pub country: Option<String>,
        pub latitude: Option<f64>,
        pub longitude: Option<f64>,
        pub start_at: Option<DateTimeWithTimeZone>,
        pub end_at: Option<DateTimeWithTimeZone>,
        pub contact_user_id: Option<i64>,
        pub permission_tags: Option<Json>,
        pub parent_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_permission_apis {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_permission_apis")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub permission_id: i64,
        pub api_id: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_permission_audit_logs {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_permission_audit_logs")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub operator_id: Option<i64>,
        pub target_type: Option<String>,
        pub target_id: Option<String>,
        pub action: Option<String>,
        pub old_value: Option<Json>,
        pub new_value: Option<Json>,
        pub ip_address: String,
        pub request_id: String,
        pub reason: String,
        pub log_hash: Option<String>,
        pub signature: Option<Vec<u8>>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_permission_groups {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_permission_groups")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub description: Option<String>,
        pub status: String,
        pub sort_order: Option<i64>,
        pub path: Option<String>,
        pub tenant_id: Option<i64>,
        pub name: String,
        pub module: Option<String>,
        pub parent_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_permission_menus {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_permission_menus")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub permission_id: i64,
        pub menu_id: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_permission_policies {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_permission_policies")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub status: String,
        pub tenant_id: Option<i64>,
        pub permission_id: i64,
        pub policy_engine: String,
        pub definition: Option<Json>,
        pub version: i64,
        pub eval_order: i64,
        pub cache_ttl: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_permissions {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_permissions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub status: String,
        pub description: Option<String>,
        pub tenant_id: Option<i64>,
        pub name: String,
        pub code: String,
        pub group_id: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_policy_evaluation_logs {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_policy_evaluation_logs")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub user_id: i64,
        pub membership_id: i64,
        pub permission_id: i64,
        pub policy_id: Option<i64>,
        pub request_path: Option<String>,
        pub request_method: Option<String>,
        pub result: bool,
        pub effect_details: Option<String>,
        pub scope_sql: Option<String>,
        pub ip_address: Option<String>,
        pub trace_id: Option<String>,
        pub evaluation_context: Option<String>,
        pub log_hash: Option<String>,
        pub signature: Option<Vec<u8>>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_positions {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_positions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub sort_order: Option<i64>,
        pub remark: Option<String>,
        pub tenant_id: Option<i64>,
        pub status: String,
        pub name: String,
        pub code: String,
        pub org_unit_id: i64,
        pub reports_to_position_id: Option<i64>,
        pub description: Option<String>,
        pub job_family: Option<String>,
        pub job_grade: Option<String>,
        pub level: Option<i32>,
        pub headcount: i64,
        pub is_key_position: bool,
        pub r#type: String,
        pub start_at: Option<DateTimeWithTimeZone>,
        pub end_at: Option<DateTimeWithTimeZone>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_role_metadata {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_role_metadata")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub role_id: Option<i64>,
        pub is_template: Option<bool>,
        pub template_for: Option<String>,
        pub template_version: Option<i32>,
        pub last_synced_version: Option<i32>,
        pub last_synced_at: Option<DateTimeWithTimeZone>,
        pub sync_policy: Option<String>,
        pub scope: Option<String>,
        pub custom_overrides: Json,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_role_permissions {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_role_permissions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub status: String,
        pub role_id: i64,
        pub permission_id: i64,
        pub effect: Option<String>,
        pub priority: Option<i32>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_roles {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_roles")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub remark: Option<String>,
        pub description: Option<String>,
        pub sort_order: Option<i64>,
        pub tenant_id: Option<i64>,
        pub status: String,
        pub name: Option<String>,
        pub code: Option<String>,
        pub is_protected: bool,
        pub r#type: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_tasks {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_tasks")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub remark: Option<String>,
        pub tenant_id: Option<i64>,
        pub r#type: Option<String>,
        pub type_name: Option<String>,
        pub task_payload: Option<Json>,
        pub cron_spec: Option<String>,
        pub task_options: Option<Json>,
        pub enable: Option<bool>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_tenants {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_tenants")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub remark: Option<String>,
        pub name: Option<String>,
        pub code: Option<String>,
        pub logo_url: Option<String>,
        pub domain: Option<String>,
        pub industry: Option<String>,
        pub admin_user_id: Option<i64>,
        pub status: Option<String>,
        pub r#type: Option<String>,
        pub audit_status: Option<String>,
        pub subscription_at: Option<DateTimeWithTimeZone>,
        pub unsubscribe_at: Option<DateTimeWithTimeZone>,
        pub subscription_plan: Option<String>,
        pub expired_at: Option<DateTimeWithTimeZone>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_user_credentials {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_user_credentials")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub tenant_id: Option<i64>,
        pub user_id: Option<i64>,
        pub identity_type: Option<String>,
        pub identifier: Option<String>,
        pub credential_type: Option<String>,
        pub credential: Option<String>,
        pub is_primary: Option<bool>,
        pub status: Option<String>,
        pub extra_info: Option<Json>,
        pub provider: Option<String>,
        pub provider_account_id: Option<String>,
        pub activate_token_hash: Option<String>,
        pub activate_token_expires_at: Option<DateTimeWithTimeZone>,
        pub activate_token_used_at: Option<DateTimeWithTimeZone>,
        pub reset_token_hash: Option<String>,
        pub reset_token_expires_at: Option<DateTimeWithTimeZone>,
        pub reset_token_used_at: Option<DateTimeWithTimeZone>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_user_org_units {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_user_org_units")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub remark: Option<String>,
        pub user_id: i64,
        pub org_unit_id: i64,
        pub position_id: Option<i64>,
        pub start_at: Option<DateTimeWithTimeZone>,
        pub end_at: Option<DateTimeWithTimeZone>,
        pub assigned_at: Option<DateTimeWithTimeZone>,
        pub assigned_by: Option<i64>,
        pub is_primary: bool,
        pub status: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_user_positions {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_user_positions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub remark: Option<String>,
        pub user_id: i64,
        pub position_id: i64,
        pub is_primary: Option<bool>,
        pub start_at: Option<DateTimeWithTimeZone>,
        pub end_at: Option<DateTimeWithTimeZone>,
        pub assigned_at: Option<DateTimeWithTimeZone>,
        pub assigned_by: Option<i64>,
        pub status: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_user_roles {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_user_roles")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub tenant_id: Option<i64>,
        pub user_id: i64,
        pub role_id: i64,
        pub start_at: Option<DateTimeWithTimeZone>,
        pub end_at: Option<DateTimeWithTimeZone>,
        pub assigned_at: Option<DateTimeWithTimeZone>,
        pub assigned_by: Option<i64>,
        pub is_primary: bool,
        pub status: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod sys_users {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "sys_users")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub remark: Option<String>,
        pub tenant_id: Option<i64>,
        pub username: Option<String>,
        pub nickname: Option<String>,
        pub realname: Option<String>,
        pub email: Option<String>,
        pub mobile: Option<String>,
        pub telephone: Option<String>,
        pub avatar: Option<String>,
        pub address: Option<String>,
        pub region: Option<String>,
        pub description: Option<String>,
        pub gender: Option<String>,
        pub last_login_at: Option<DateTimeWithTimeZone>,
        pub last_login_ip: Option<String>,
        pub locked_until: Option<DateTimeWithTimeZone>,
        pub status: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod tag_translations {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "tag_translations")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub seo: Option<Json>,
        pub tenant_id: Option<i64>,
        pub tag_id: Option<i64>,
        pub language_code: Option<String>,
        pub name: Option<String>,
        pub slug: Option<String>,
        pub description: Option<String>,
        pub cover_image: Option<String>,
        pub full_path: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod tags {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "tags")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
        pub deleted_at: Option<DateTimeWithTimeZone>,
        pub created_by: Option<i64>,
        pub updated_by: Option<i64>,
        pub deleted_by: Option<i64>,
        pub sort_order: Option<i64>,
        pub tenant_id: Option<i64>,
        pub status: Option<String>,
        pub color: Option<String>,
        pub icon: Option<String>,
        pub group: Option<String>,
        pub code: Option<String>,
        pub is_featured: Option<bool>,
        pub post_count: Option<i64>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
