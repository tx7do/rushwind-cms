-- 系统默认种子（对位 go-wind-cms pkg/constants/default_data.go 的空库引导）：
-- 权限组 / 权限 / 角色 / 角色元数据 / 平台管理员 / 用户角色 / 成员关系 / 语言。
-- 幂等：以「表空才插」为约定（bootstrap 只在空库执行 schema → seed → demo）。

INSERT INTO public.sys_permission_groups (id, name, path, module, sort_order, status, created_at)
VALUES
    (1, '系统管理', '/', 'sys', 1, 'ON', now()),
    (2, '系统权限', '/1/2/', 'sys', 1, 'ON', now()),
    (3, '租户管理', '/1/3/', 'sys', 2, 'ON', now()),
    (4, '审计管理', '/1/4/', 'sys', 3, 'ON', now()),
    (5, '安全策略', '/1/5/', 'sys', 4, 'ON', now());
SELECT setval('sys_permission_groups_id_seq', (SELECT MAX(id) FROM public.sys_permission_groups));

INSERT INTO public.sys_permissions (id, group_id, name, description, code, status, created_at)
VALUES
    (1, 2, '访问后台', '允许用户访问系统后台管理界面', 'sys:access_backend', 'ON', now()),
    (2, 2, '平台管理员权限', '拥有系统所有功能的操作权限，可管理租户、用户、角色及所有资源', 'sys:platform_admin', 'ON', now()),
    (3, 3, '租户管理员权限', '拥有租户内所有功能的操作权限，可管理用户、角色及租户内所有资源', 'sys:tenant_manager', 'ON', now()),
    (4, 3, '管理租户', '允许创建/修改/删除租户', 'sys:manage_tenants', 'ON', now()),
    (5, 4, '查看审计日志', '允许查看系统操作日志', 'sys:audit_logs', 'ON', now()),
    (6, 2, '访问应用端', '允许用户访问 C 端应用（前台登录与注册用户默认权限）', 'sys:access_app', 'ON', now());
SELECT setval('sys_permissions_id_seq', (SELECT MAX(id) FROM public.sys_permissions));

INSERT INTO public.sys_roles (id, name, code, status, description, is_protected, "type", sort_order, created_at)
VALUES
    (1, '平台管理员', 'platform:admin', 'ON', '拥有系统所有功能的操作权限，可管理租户、用户、角色及所有资源', true, 'SYSTEM', 1, now()),
    (2, '租户管理员模板', 'template:tenant:manager', 'ON', '租户管理员角色，拥有租户内所有功能的操作权限，可管理用户、角色及租户内所有资源', true, 'TEMPLATE', 2, now()),
    (3, '租户普通用户', 'tenant:user', 'ON', '租户普通用户角色（C 端注册默认角色）', true, 'SYSTEM', 3, now());
SELECT setval('sys_roles_id_seq', (SELECT MAX(id) FROM public.sys_roles));

INSERT INTO public.sys_role_metadata (id, role_id, is_template, template_for, template_version, scope, sync_policy, custom_overrides, created_at)
VALUES
    (1, 1, false, NULL, 1, 'PLATFORM', 'AUTO', '{}'::jsonb, now()),
    (2, 2, true, 'tenant:manager', 1, 'TENANT', 'AUTO', '{}'::jsonb, now());
SELECT setval('sys_role_metadata_id_seq', (SELECT MAX(id) FROM public.sys_role_metadata));

INSERT INTO public.sys_role_permissions (id, tenant_id, role_id, permission_id, status, created_at)
VALUES
    (1, 0, 1, 1, 'ON', now()),
    (2, 0, 1, 2, 'ON', now()),
    (3, 0, 1, 4, 'ON', now()),
    (4, 0, 1, 6, 'ON', now()),
    (5, 0, 2, 1, 'ON', now()),
    (6, 0, 2, 3, 'ON', now()),
    (7, 0, 3, 6, 'ON', now());

-- 平台管理员（admin/admin，密码哈希与 demo 数据同源：bcrypt("admin")）
INSERT INTO public.sys_users (id, tenant_id, username, nickname, realname, email, region, created_at)
VALUES (1, 0, 'admin', '鹳狸猿', '喵个咪', 'admin@gmail.com', '中国', now());
SELECT setval('sys_role_permissions_id_seq', (SELECT MAX(id) FROM public.sys_role_permissions));
SELECT setval('sys_users_id_seq', (SELECT MAX(id) FROM public.sys_users));

INSERT INTO public.sys_user_credentials (id, tenant_id, user_id, identity_type, identifier, credential_type, credential, is_primary, status, created_at)
VALUES (1, 0, 1, 'USERNAME', 'admin', 'PASSWORD_HASH', '$2a$10$yajZDX20Y40FkG0Bu4N19eXNqRizez/S9fK63.JxGkfLq.RoNKR/a', true, 'ENABLED', now());
SELECT setval('sys_user_credentials_id_seq', (SELECT MAX(id) FROM public.sys_user_credentials));

INSERT INTO public.sys_user_roles (id, tenant_id, user_id, role_id, is_primary, status, created_at)
VALUES (1, 0, 1, 1, true, 'ACTIVE', now());
SELECT setval('sys_user_roles_id_seq', (SELECT MAX(id) FROM public.sys_user_roles));

INSERT INTO public.sys_memberships (id, tenant_id, user_id, is_primary, status, created_at)
VALUES (1, 0, 1, true, 'ACTIVE', now());
SELECT setval('sys_memberships_id_seq', (SELECT MAX(id) FROM public.sys_memberships));

INSERT INTO public.sys_languages (id, language_code, language_name, native_name, is_default, is_enabled, sort_order, created_at)
VALUES
    (1,'zh-CN', '中文（简体）', '简体中文', true, true, 0, now()),
    (2,'zh-TW', '中文（繁体）', '繁體中文', false, true, 100, now()),
    (3,'en-US', '英语', 'English', false, true, 1, now()),
    (4,'ja-JP', '日语', '日本語', false, true, 100, now()),
    (5,'ko-KR', '韩语', '한국어', false, true, 100, now()),
    (6,'es-ES', '西班牙语', 'Español', false, true, 100, now()),
    (7,'fr-FR', '法语', 'Français', false, true, 100, now());
SELECT setval('sys_languages_id_seq', (SELECT MAX(id) FROM public.sys_languages));
