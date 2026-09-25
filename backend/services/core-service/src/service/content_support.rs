//! Shared machinery of the four content services: the enum-name
//! status tables (the EnumTypeConverter behavior) and the SEO-meta
//! JSON round-trip. Tests live beside the tables.

use proto::proto::content::service::v1 as contentv1;

pub(crate) fn status_to_i32(prefix: &str, name: Option<String>) -> Option<i32> {
    // The stored enum-value name → the proto value (name ↔ number table
    // per enum: derive by ordinal of the known members).
    let name = name?;
    Some(match (prefix, name.as_str()) {
        (_, "POST_STATUS_DRAFT") => 1,
        (_, "POST_STATUS_PUBLISHED") => 2,
        (_, "POST_STATUS_SCHEDULED") => 3,
        (_, "POST_STATUS_TRASHED") => 4,
        (_, "PAGE_STATUS_DRAFT") => 1,
        (_, "PAGE_STATUS_PUBLISHED") => 2,
        (_, "PAGE_STATUS_ARCHIVED") => 3,
        (_, "CATEGORY_STATUS_ACTIVE") => 1,
        (_, "CATEGORY_STATUS_HIDDEN") => 2,
        (_, "CATEGORY_STATUS_ARCHIVED") => 3,
        (_, "TAG_STATUS_ACTIVE") => 1,
        (_, "TAG_STATUS_HIDDEN") => 2,
        (_, "TAG_STATUS_ARCHIVED") => 3,
        // Legacy labels the seed layers used before the enum alignment.
        (_, "CATEGORY_STATUS_DRAFT") => 1,
        (_, "CATEGORY_STATUS_PUBLISHED") => 2,
        (_, "TAG_STATUS_NORMAL") => 1,
        (_, "TAG_STATUS_DISABLED") => 2,
        _ => 0,
    })
}

pub(crate) fn status_from_i32(prefix: &str, v: i32) -> Option<String> {
    Some(
        match (prefix, v) {
            ("post", 1) => "POST_STATUS_DRAFT",
            ("post", 2) => "POST_STATUS_PUBLISHED",
            ("post", 3) => "POST_STATUS_SCHEDULED",
            ("post", 4) => "POST_STATUS_TRASHED",
            ("page", 1) => "PAGE_STATUS_DRAFT",
            ("page", 2) => "PAGE_STATUS_PUBLISHED",
            ("page", 3) => "PAGE_STATUS_ARCHIVED",
            ("category", 1) => "CATEGORY_STATUS_ACTIVE",
            ("category", 2) => "CATEGORY_STATUS_HIDDEN",
            ("category", 3) => "CATEGORY_STATUS_ARCHIVED",
            ("tag", 1) => "TAG_STATUS_ACTIVE",
            ("tag", 2) => "TAG_STATUS_HIDDEN",
            ("tag", 3) => "TAG_STATUS_ARCHIVED",
            _ => return None,
        }
        .to_string(),
    )
}

pub(crate) fn seo_of(json: Option<sea_orm::JsonValue>) -> Option<contentv1::SeoMeta> {
    // Field-level mapping (the prost type carries no serde derives).
    let v = json?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).map(str::to_owned);
    Some(contentv1::SeoMeta {
        seo_title: s("seoTitle"),
        meta_keywords: s("metaKeywords"),
        meta_description: s("metaDescription"),
        og_title: s("ogTitle"),
        og_description: s("ogDescription"),
        og_image: s("ogImage"),
        ..Default::default()
    })
}

pub(crate) fn seo_to_json(seo: Option<contentv1::SeoMeta>) -> Option<sea_orm::JsonValue> {
    let seo = seo?;
    serde_json::json!({
        "seoTitle": seo.seo_title,
        "metaKeywords": seo.meta_keywords,
        "metaDescription": seo.meta_description,
        "ogTitle": seo.og_title,
        "ogDescription": seo.og_description,
        "ogImage": seo.og_image,
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── status_from_i32 ─────────────────────────────────────────────

    #[test]
    fn status_from_i32_maps_every_member_of_every_enum() {
        let table = [
            ("post", 1, "POST_STATUS_DRAFT"),
            ("post", 2, "POST_STATUS_PUBLISHED"),
            ("post", 3, "POST_STATUS_SCHEDULED"),
            ("post", 4, "POST_STATUS_TRASHED"),
            ("page", 1, "PAGE_STATUS_DRAFT"),
            ("page", 2, "PAGE_STATUS_PUBLISHED"),
            ("page", 3, "PAGE_STATUS_ARCHIVED"),
            ("category", 1, "CATEGORY_STATUS_ACTIVE"),
            ("category", 2, "CATEGORY_STATUS_HIDDEN"),
            ("category", 3, "CATEGORY_STATUS_ARCHIVED"),
            ("tag", 1, "TAG_STATUS_ACTIVE"),
            ("tag", 2, "TAG_STATUS_HIDDEN"),
            ("tag", 3, "TAG_STATUS_ARCHIVED"),
        ];
        for (prefix, v, want) in table {
            assert_eq!(
                status_from_i32(prefix, v).as_deref(),
                Some(want),
                "{prefix}/{v}"
            );
        }
    }

    #[test]
    fn status_from_i32_unknown_values_and_prefixes_yield_none() {
        // Out-of-range values.
        for (prefix, v) in [
            ("post", 0),
            ("post", 5),
            ("page", 0),
            ("page", 4),
            ("category", 0),
            ("category", 4),
            ("tag", 0),
            ("tag", 4),
        ] {
            assert_eq!(status_from_i32(prefix, v), None, "{prefix}/{v}");
        }
        // Unknown enum families never map.
        assert_eq!(status_from_i32("unknown", 1), None);
        assert_eq!(status_from_i32("", 1), None);
        assert_eq!(status_from_i32("POST", 1), None);
    }

    // ── status_to_i32 ───────────────────────────────────────────────

    #[test]
    fn status_to_i32_round_trips_the_from_i32_table() {
        for prefix in ["post", "page", "category", "tag"] {
            for v in 1..=4 {
                if let Some(name) = status_from_i32(prefix, v) {
                    assert_eq!(status_to_i32(prefix, Some(name)), Some(v), "{prefix}/{v}");
                }
            }
        }
    }

    #[test]
    fn status_to_i32_missing_name_is_none_and_unknown_name_is_zero() {
        assert_eq!(status_to_i32("post", None), None);
        // The unknown-name arm reads 0 (the proto's UNSPECIFIED), the
        // prefix never disambiguates — the stored name decides.
        assert_eq!(status_to_i32("post", Some("NOT_A_STATUS".into())), Some(0));
        assert_eq!(
            status_to_i32("tag", Some("POST_STATUS_DRAFT".into())),
            Some(1)
        );
    }
}
