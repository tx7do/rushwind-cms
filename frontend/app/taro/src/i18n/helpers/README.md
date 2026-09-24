# i18n Helpers - Taro 多语言支持

本目录提供 Taro 项目的多语言支持功能。

## 📚 目录结构

```
helpers/
├── useI18n.ts          # 国际化 Hook
├── useI18nRouter.ts    # 多语言路由 Hook
└── index.ts            # 统一导出
```

## ✨ 特性

- ✅ 基于 `react-i18next` 实现
- ✅ 支持多语言切换
- ✅ 自动保存到 localStorage
- ✅ 与 Redux store 同步
- ✅ 类型安全

## 🚀 快速开始

### 1. useI18n Hook

```tsx
import {useI18n} from '@/i18n';

export function MyComponent() {
    const {t, locale, changeLocale} = useI18n('common');
    
    return (
        <div>
            <h1>{t('welcome')}</h1>
            <p>当前语言：{locale}</p>
            <button onClick={() => changeLocale('en-US')}>
                Switch to English
            </button>
        </div>
    );
}
```

### 2. useI18nRouter Hook

```tsx
import {useI18nRouter} from '@/i18n';

export function NavigationExample() {
    const router = useI18nRouter();
    
    return (
        <button onClick={() => router.push('/pages/category/index')}>
            查看分类
        </button>
    );
}
```

## 📋 API

### useI18n

```typescript
function useI18n(namespace?: string) {
    return {
        t: (key: string) => string,      // 翻译函数
        locale: string,                   // 当前语言
        changeLocale: (locale: string) => void,  // 切换语言
    };
}
```

### useI18nRouter

```typescript
function useI18nRouter() {
    return {
        push: (path: string, params?: object) => void,      // 导航到页面
        replace: (path: string, params?: object) => void,   // 替换当前页面
        back: (delta?: number) => void,                     // 返回上一页
        refresh: () => void,                                 // 刷新页面
        localizedPath: (path: string) => string,            // 获取本地化路径
    };
}
```

## 💡 使用示例

### 完整的页面示例

```tsx
import {View, Text} from '@tarojs/components';
import {useI18n} from '@/i18n';
import {useI18nRouter} from '@/i18n';

export default function Index() {
    const {t, locale, changeLocale} = useI18n('common');
    const router = useI18nRouter();
    
    return (
        <View>
            <Text>{t('welcome')}</Text>
            <Text>当前语言：{locale}</Text>
            
            <View onClick={() => changeLocale('en-US')}>
                <Text>Switch to English</Text>
            </View>
            
            <View onClick={() => router.push('/pages/category/index?id=1')}>
                <Text>查看分类</Text>
            </View>
        </View>
    );
}
```

## ⚠️ 注意事项

### 1. 消息文件

确保在 `src/i18n/messages/` 目录下有对应的语言文件：

```
messages/
├── zh-CN/
│   └── common.json
└── en-US/
    └── common.json
```

### 2. 初始化配置

在 `src/app.ts` 中已经初始化了 i18n：

```tsx
import './i18n';
```

### 3. localStorage

语言偏好会保存到 localStorage，键名为 `locale`。

## 🔗 相关文件

- `../index.ts` - 主入口和初始化
- `../config.ts` - 语言配置
- `../messages/` - 翻译文件
