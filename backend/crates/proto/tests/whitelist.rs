//! Corpus guard for the auth-free split of the two BFF faces.
//!
//! The generator classifies every route binding via the auth-free table
//! (`proto::AUTH_FREE`, the two deployments' whitelist union). This test
//! pins BOTH directions against the live corpora:
//!
//! * every whitelisted pair still exists as a route binding in its owning
//!   face — a dropped proto method would silently widen the gated set (a
//!   parity break);
//! * nothing beyond the whitelist classifies public — a table typo would
//!   silently expose a protected endpoint (a security break);
//! * the public binding counts match, so `additional_bindings` of a
//!   whitelisted operation stay exempt with their primary binding.
//!
//! The app BFF's hand-mounted C-side register route (`POST /app/v1/register`)
//! is not a proto route and is absent here by design — it mounts manually
//! in the app service, mirroring the reference's hand registration.

use std::collections::BTreeSet;

/// The admin face's expected public pairs — the admin BFF's whitelist
/// registrations, verbatim.
const ADMIN_EXPECTED: &[(&str, &str)] = &[
    ("admin.service.v1.AuthenticationService", "Login"),
    ("admin.service.v1.AuthenticationService", "GenerateCaptcha"),
    ("admin.service.v1.AuthenticationService", "VerifyCaptcha"),
    ("admin.service.v1.AuthenticationService", "RefreshToken"),
];

/// The app face's expected public pairs — the app BFF's whitelist
/// registrations, verbatim.
const APP_EXPECTED: &[(&str, &str)] = &[
    ("app.service.v1.AuthenticationService", "Login"),
    ("app.service.v1.AuthenticationService", "RefreshToken"),
    ("app.service.v1.NavigationService", "List"),
    ("app.service.v1.CommentService", "Create"),
    ("app.service.v1.SiteService", "GetSiteByDomain"),
    ("app.service.v1.PageService", "List"),
    ("app.service.v1.PageService", "Get"),
    ("app.service.v1.PostService", "List"),
    ("app.service.v1.PostService", "Get"),
    ("app.service.v1.PostService", "SearchPosts"),
    ("app.service.v1.CategoryService", "List"),
    ("app.service.v1.CategoryService", "Get"),
    ("app.service.v1.CommentService", "List"),
    ("app.service.v1.CommentService", "Get"),
    ("app.service.v1.TagService", "List"),
    ("app.service.v1.TagService", "Get"),
    ("app.service.v1.InteractionService", "GetCounts"),
];

fn public_of(routes: &[proto::gen_admin::routes::RouteSpec]) -> (BTreeSet<(&str, &str)>, usize) {
    let mut public: BTreeSet<(&str, &str)> = BTreeSet::new();
    let mut bindings = 0usize;
    for route in routes {
        if proto::AUTH_FREE
            .iter()
            .any(|(s, m)| *s == route.service_fq && *m == route.method_name)
        {
            public.insert((route.service_fq, route.method_name));
            bindings += 1;
        }
    }
    (public, bindings)
}

fn public_of_app(routes: &[proto::gen_app::routes::RouteSpec]) -> (BTreeSet<(&str, &str)>, usize) {
    let mut public: BTreeSet<(&str, &str)> = BTreeSet::new();
    let mut bindings = 0usize;
    for route in routes {
        if proto::AUTH_FREE
            .iter()
            .any(|(s, m)| *s == route.service_fq && *m == route.method_name)
        {
            public.insert((route.service_fq, route.method_name));
            bindings += 1;
        }
    }
    (public, bindings)
}

#[test]
fn auth_free_routes_match_the_go_whitelists() {
    // The admin face.
    let (admin_public, admin_bindings) = public_of(proto::gen_admin::routes::ROUTES);
    let admin_expected: BTreeSet<(&str, &str)> = ADMIN_EXPECTED.iter().copied().collect();
    assert_eq!(
        admin_public, admin_expected,
        "the admin public route set diverged from the auth-free set"
    );
    assert_eq!(
        admin_bindings,
        ADMIN_EXPECTED.len(),
        "the admin public binding count diverged (additional_bindings drift)"
    );

    // The app face.
    let (app_public, app_bindings) = public_of_app(proto::gen_app::routes::ROUTES);
    let app_expected: BTreeSet<(&str, &str)> = APP_EXPECTED.iter().copied().collect();
    assert_eq!(
        app_public, app_expected,
        "the app public route set diverged from the auth-free set"
    );
    assert_eq!(
        app_bindings,
        APP_EXPECTED.len(),
        "the app public binding count diverged (additional_bindings drift)"
    );
}

/// The shadow set: bindings a first-match mux absorbs never reach (an
/// earlier same-method pattern route absorbs their paths), so the mounts
/// skip them. Pinned per face — contract drift that adds or removes a
/// shadow must land here consciously.
#[test]
fn shadowed_routes_match_the_registered_set() {
    let admin_observed: Vec<(usize, &str, &str)> = proto::gen_admin::routes::ROUTES
        .iter()
        .enumerate()
        .filter(|(_, r)| r.shadowed)
        .map(|(i, r)| (i, r.method, r.path))
        .collect();
    let app_observed: Vec<(usize, &str, &str)> = proto::gen_app::routes::ROUTES
        .iter()
        .enumerate()
        .filter(|(_, r)| r.shadowed)
        .map(|(i, r)| (i, r.method, r.path))
        .collect();
    // i_api.proto: /admin/v1/apis/{id} (GET, registered earlier in the same
    // file) absorbs the walk-route literal path — the same registration-order
    // shadow the reference's first-match mux exhibits.
    let admin_expected: Vec<(usize, &str, &str)> = vec![(9, "GET", "/admin/v1/apis/walk-route")];
    assert_eq!(
        admin_observed, admin_expected,
        "the admin shadow set diverged (registration-order or path-shape drift)"
    );
    assert!(
        app_observed.is_empty(),
        "unexpected app shadow set: {app_observed:?}"
    );
}
