import {defineRouting} from 'next-intl/routing';
import {DEFAULT_LANGUAGE, locales} from "@/i18n";

export const routing = defineRouting({
    // 支持的语言列表
    locales: locales,

    // 默认语言
    defaultLocale: DEFAULT_LANGUAGE,

    // URL 中始终包含语言前缀
    localePrefix: 'always',
});

export type SupportedLocale = (typeof routing.locales)[number];

export function getSupportedLocale(locale: string): SupportedLocale | undefined {
    return routing.locales.find(l => l === locale);
}

export function isSupportedLocale(locale: string): locale is SupportedLocale {
    return getSupportedLocale(locale) !== undefined;
}
