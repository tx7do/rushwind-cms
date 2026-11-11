//! The identity constants of the reference's pkg/constants (role.go /
//! permission.go): the code namespaces that carry identity semantics by
//! exact code match, plus the reserved-code guards the CRUD faces
//! consult. One definition here — the faces used to carry per-file
//! copies of the same strings.

// ── role codes (role.go) ─────────────────────────────────────────────

/// TemplateRoleCodePrefix
pub const TEMPLATE_ROLE_CODE_PREFIX: &str = "template:";
/// PlatformRoleCodePrefix
pub const PLATFORM_ROLE_CODE_PREFIX: &str = "platform:";
/// TenantRoleCodePrefix
pub const TENANT_ROLE_CODE_PREFIX: &str = "tenant:";

/// PlatformAdminRoleCode
pub const PLATFORM_ADMIN_ROLE_CODE: &str = "platform:admin";
/// TenantAdminRoleCode
pub const TENANT_MANAGER_ROLE_CODE: &str = "tenant:manager";
/// TenantUserRoleCode（C 端注册默认角色）
pub const TENANT_USER_ROLE_CODE: &str = "tenant:user";
/// TenantAdminTemplateRoleCode
pub const TENANT_ADMIN_TEMPLATE_ROLE_CODE: &str = "template:tenant:manager";

/// IsPlatformRoleCode — the `platform:` namespace rides the token's
/// admin flag by exact code match.
pub fn is_platform_role_code(code: &str) -> bool {
    code.starts_with(PLATFORM_ROLE_CODE_PREFIX)
}

/// IsTemplateRoleCode — template codes query the instantiation path.
pub fn is_template_role_code(code: &str) -> bool {
    code.starts_with(TEMPLATE_ROLE_CODE_PREFIX)
}

/// ExtractRoleCodeFromTemplate — strip the template prefix.
pub fn extract_role_code_from_template(code: &str) -> &str {
    if is_template_role_code(code) {
        return &code[TEMPLATE_ROLE_CODE_PREFIX.len()..];
    }
    code
}

/// IsReservedRoleCode — 判断角色码是否为系统保留，不可通过角色 CRUD
/// API 创建/修改。platform: / template: 命名空间与内置租户管理员码在
/// 程序里承载身份判定语义（token 管理员标志按码值精确匹配、模板实例化
/// 按码值查询、鉴权策略按码值做主体），用户自建角色一旦占用即可伪装成
/// 内置身份越权。系统内的合法保留码角色只能由种子数据与租户开通等内部
/// 链路（绕过 service 层校验）产生。
pub fn is_reserved_role_code(code: &str) -> bool {
    code == TENANT_MANAGER_ROLE_CODE || is_platform_role_code(code) || is_template_role_code(code)
}

// ── permission codes (permission.go) ─────────────────────────────────

/// SystemPermissionCodePrefix
pub const SYSTEM_PERMISSION_CODE_PREFIX: &str = "sys:";

/// SystemAccessBackendPermissionCode — 登录后台的闸门码
pub const SYSTEM_ACCESS_BACKEND_PERMISSION_CODE: &str = "sys:access_backend";
/// SystemAccessAppPermissionCode — 登录 C 端的闸门码
pub const SYSTEM_ACCESS_APP_PERMISSION_CODE: &str = "sys:access_app";
/// SystemManageTenantsPermissionCode
pub const SYSTEM_MANAGE_TENANTS_PERMISSION_CODE: &str = "sys:manage_tenants";
/// SystemAuditLogsPermissionCode
pub const SYSTEM_AUDIT_LOGS_PERMISSION_CODE: &str = "sys:audit_logs";
/// SystemPlatformAdminPermissionCode
pub const SYSTEM_PLATFORM_ADMIN_PERMISSION_CODE: &str = "sys:platform_admin";
/// SystemTenantManagerPermissionCode
pub const SYSTEM_TENANT_MANAGER_PERMISSION_CODE: &str = "sys:tenant_manager";

/// IsReservedPermissionCode — 判断权限码是否为系统保留，不可通过权限
/// CRUD API 创建/修改。sys: 命名空间在服务端承载判定语义
/// （sys:access_backend 是登录后台的闸门码，登录链路按码值精确匹配），
/// 用户自建权限一旦占用即可伪造登录资格等权限语义。
pub fn is_reserved_permission_code(code: &str) -> bool {
    code.starts_with(SYSTEM_PERMISSION_CODE_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_role_codes_reject_the_identity_namespaces() {
        assert!(is_reserved_role_code("platform:admin"));
        assert!(is_reserved_role_code("platform:anything"));
        assert!(is_reserved_role_code("template:tenant:manager"));
        assert!(is_reserved_role_code("template:whatever"));
        assert!(is_reserved_role_code("tenant:manager"));
        // the built-in tenant/user codes are creatable namespaces
        assert!(!is_reserved_role_code("tenant:user"));
        assert!(!is_reserved_role_code("tenant:custom"));
        assert!(!is_reserved_role_code("editor"));
    }

    #[test]
    fn reserved_permission_codes_reject_the_sys_namespace() {
        assert!(is_reserved_permission_code("sys:access_backend"));
        assert!(is_reserved_permission_code("sys:anything"));
        assert!(!is_reserved_permission_code("task:create"));
        assert!(!is_reserved_permission_code(""));
    }

    #[test]
    fn template_extraction_strips_the_prefix_only() {
        assert_eq!(
            extract_role_code_from_template("template:tenant:manager"),
            "tenant:manager"
        );
        assert_eq!(
            extract_role_code_from_template("tenant:manager"),
            "tenant:manager"
        );
    }
}
