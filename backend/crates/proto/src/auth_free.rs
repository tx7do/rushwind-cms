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
    // AuthenticationService.GetOidcAuthorizeUrl：OIDC 登录的前置步骤（登录页
    // 尚无 token），与 Login 同列免鉴权；state 防 CSRF 由 core 生成校验。
    ("admin.service.v1.AuthenticationService", "GetOidcAuthorizeUrl"),
    // AuthenticationService.ForgotPassword / ResetPasswordByCode：忘记密码两步
    //（发重置验证码 / 凭码重置），发生在登录之前、调用方必然无 token。
    // 防枚举由 core 的"查无此人静默 200"承担，验证码单次有效 10 分钟。
    ("admin.service.v1.AuthenticationService", "ForgotPassword"),
    ("admin.service.v1.AuthenticationService", "ResetPasswordByCode"),
    // AuthenticationService.GetTenantBranding：登录页按租户编号渲染 Logo/名称，
    // 登录前置步骤；只暴露展示字段，未命中返回 found=false 防枚举。
    ("admin.service.v1.AuthenticationService", "GetTenantBranding"),
    // AccessKeyService.IssueToken：AK/SK 令牌交换（机器对机器），免鉴权——
    // 它本身即是认证；调用方凭 AccessKey 头无 bearer，须在白名单内。
    ("admin.service.v1.AccessKeyService", "IssueToken"),
    // MFAService.VerifyMFAChallenge：两步验证挑战校验（半登录态换正式令牌），
    // 调用方仅持一次性挑战凭据、无 bearer；凭据本身 5 分钟一次性消费即防护。
    ("admin.service.v1.MFAService", "VerifyMFAChallenge"),
    // ── app BFF（app/app/service rest_server.go AddWhiteList）──
    ("app.service.v1.AuthenticationService", "Login"),
    ("app.service.v1.AuthenticationService", "RefreshToken"),
    // GetOidcAuthorizeUrl：OIDC 登录的前置步骤（登录页尚无 token）。
    ("app.service.v1.AuthenticationService", "GetOidcAuthorizeUrl"),
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
    // PostService.SearchPostsSuggest：搜索框联想（标题前缀），公开度同 SearchPosts。
    ("app.service.v1.PostService", "SearchPostsSuggest"),
    // PostService/PageService.GetBySlug：公开 SEO 路由（/post/{slug}、/page/{slug}
    // 的数据源），仅返回已发布内容，匿名可访问。
    ("app.service.v1.PostService", "GetBySlug"),
    ("app.service.v1.PageService", "GetBySlug"),
    // PostService.GetPostContentModel：C 端渲染 custom_fields 的公开数据源，
    // 模型定义（字段名/标签/类型）非敏感；模型解析在 core 内部完成。
    ("app.service.v1.PostService", "GetPostContentModel"),
    // PostService.ResolveContentReferences：RELATION 字段引用 → 目标内容摘要
    // （公开关联卡片）。core 仅解析 PUBLISHED 目标，不可见引用静默跳过。
    ("app.service.v1.PostService", "ResolveContentReferences"),
    // MediaAssetService.Get：customFields IMAGE 引用（media_asset:<id>）换公开
    // URL 渲染用；图片本身随文章公开，租户隔离由 core viewer 保证。
    // GetContent（手工挂载非 proto 路由）与 SeoService（sitemap/robots，同为
    // 手工路由）不在本表——仅 proto 路由走生成器的白名单分类。
    ("app.service.v1.MediaAssetService", "Get"),
    // InteractionService.RecordView：公开浏览计数回传，仅按 tenant 隔离。
    ("app.service.v1.InteractionService", "RecordView"),
    // FormSubmissionService.Submit：页面 FORM 区块的访客提交，匿名可调用
    //（校验在 core：页面须已发布、区块须为该页面的 FORM 区块、字段集受限）。
    ("app.service.v1.FormSubmissionService", "Submit"),
    // RedirectService.ListEnabled：公开重定向查询（C 端路由中间件 3xx 匹配）。
    ("app.service.v1.RedirectService", "ListEnabled"),
    ("app.service.v1.CategoryService", "List"),
    ("app.service.v1.CategoryService", "Get"),
    ("app.service.v1.CommentService", "List"),
    ("app.service.v1.CommentService", "Get"),
    ("app.service.v1.TagService", "List"),
    ("app.service.v1.TagService", "Get"),
    // InteractionService.GetCounts：公开计数（点赞数），仅按 tenant 隔离。
    ("app.service.v1.InteractionService", "GetCounts"),
];
