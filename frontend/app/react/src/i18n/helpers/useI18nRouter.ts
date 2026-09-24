'use client';

import {useRouter as useNextRouter, usePathname} from 'next/navigation';
import {routing} from '@/app/[locale]/routing';

/**
 * 多语言路由 Hook - 封装 next-intl 的路由功能
 *
 * @description
 * 自动处理语言前缀的路由导航，无需手动拼接 locale
 *
 * @example
 * ```typescript
 * const router = useI18nRouter();
 *
 * // 自动添加当前语言前缀
 * router.push('/category/1');        // /zh-CN/category/1
 * router.push('/post/123');          // /zh-CN/post/123
 * router.push('/settings/profile');  // /zh-CN/settings/profile
 *
 * // 获取本地化路径
 * const localizedPath = router.localizedPath('/category/1');
 *
 * // 替换当前历史记录
 * router.replace('/new-path');
 *
 * // 前进/后退
 * router.back();
 * router.forward();
 * ```
 */
export function useI18nRouter() {
    const router = useNextRouter();
    const pathname = usePathname();

    /**
     * 获取当前语言
     */
    const getCurrentLocale = (): string => {
        const segments = pathname.split('/');
        return segments[1] || routing.defaultLocale;
    };

    /**
     * 导航到指定路径（自动添加语言前缀）
     */
    const push = (path: string, options?: { scroll?: boolean }) => {
        const locale = getCurrentLocale();
        const localizedPath = `/${locale}${path}`;
        if (pathname === localizedPath) {
            // 如果当前路径已经是本地化路径，则刷新页面
            router.refresh();
            return;
        }
        router.push(localizedPath, options);
    };

    /**
     * 替换当前路径（自动添加语言前缀）
     */
    const replace = (path: string, options?: { scroll?: boolean }) => {
        const locale = getCurrentLocale();
        const localizedPath = `/${locale}${path}`;
        router.replace(localizedPath, options);
    };

    const replaceWithoutLocale = (path: string, options?: { scroll?: boolean }) => {
        router.replace(path, options);
    }

    /**
     * 获取本地化路径（不执行导航）
     */
    const getLocalizedPath = (path: string): string => {
        const locale = getCurrentLocale();
        return `/${locale}${path}`;
    };

    /**
     * 返回上一页
     */
    const back = () => {
        router.back();
    };

    /**
     * 前进
     */
    const forward = () => {
        router.forward();
    };

    /**
     * 刷新当前页面
     */
    const refresh = () => {
        router.refresh();
    };

    return {
        push,
        replace,
        replaceWithoutLocale,
        back,
        forward,
        refresh,
        localizedPath: getLocalizedPath,
        /**
         * @deprecated 使用 localizedPath 代替
         */
        getLocalizedPath,
    };
}

/**
 * 原始 Next.js Router（不使用多语言前缀）
 *
 * @warning 仅在特殊场景下使用（如外部链接、API 调用等）
 */
export function useRouter() {
    return useNextRouter();
}
