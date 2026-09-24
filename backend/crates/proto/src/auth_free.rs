/// The auth-free operations: the two deployments' whitelist registrations
/// (the admin BFF's and the app BFF's), pinned 1:1 as
/// (service full name, proto method name) pairs. The union is safe: the
/// two BFF surfaces own disjoint route sets, so each deployment's split
/// is unaffected by the other's entries.
///
/// The app BFF's hand-mounted C-side register route (`POST /app/v1/register`,
/// not declared in the app proto) is NOT in this table — it mounts as a
/// manual route in the app service, mirroring the reference's hand
/// registration. Consumed by the build script's generator config (the
/// public/gated mount split) and re-exported here for the corpus guard
/// test.
pub const AUTH_FREE: &[(&str, &str)] = &[
    // ── admin BFF（app/admin/service rest_server.go AddWhiteList）──
    ("admin.service.v1.AuthenticationService", "Login"),
    ("admin.service.v1.AuthenticationService", "GenerateCaptcha"),
    ("admin.service.v1.AuthenticationService", "VerifyCaptcha"),
    ("admin.service.v1.AuthenticationService", "RefreshToken"),
    // ── app BFF（app/app/service rest_server.go AddWhiteList）──
    ("app.service.v1.AuthenticationService", "Login"),
    ("app.service.v1.AuthenticationService", "RefreshToken"),
    ("app.service.v1.NavigationService", "List"),
    // CommentService.Create：游客评论策略在操作内自行执行（可选认证 +
    // enable_comments/allow_guest_comments 开关），白名单仅放行游客请求。
    ("app.service.v1.CommentService", "Create"),
    // SiteService.GetSiteByDomain：公开站点配置（渲染必需字段）。
    ("app.service.v1.SiteService", "GetSiteByDomain"),
    ("app.service.v1.PageService", "List"),
    ("app.service.v1.PageService", "Get"),
    ("app.service.v1.PostService", "List"),
    ("app.service.v1.PostService", "Get"),
    // PostService.SearchPosts：公开全文搜索，匿名可见性与列表/详情一致。
    ("app.service.v1.PostService", "SearchPosts"),
    ("app.service.v1.CategoryService", "List"),
    ("app.service.v1.CategoryService", "Get"),
    ("app.service.v1.CommentService", "List"),
    ("app.service.v1.CommentService", "Get"),
    ("app.service.v1.TagService", "List"),
    ("app.service.v1.TagService", "Get"),
    // InteractionService.GetCounts：公开计数（点赞数），仅按 tenant 隔离。
    ("app.service.v1.InteractionService", "GetCounts"),
];
