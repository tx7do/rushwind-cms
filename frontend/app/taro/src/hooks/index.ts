import {useState, useCallback} from 'react';
import Taro from '@tarojs/taro';

interface UseLoadingOptions {
    initialValue?: boolean;
    duration?: number;
}

/**
 * 加载状态 Hook
 */
export function useLoading(options: UseLoadingOptions = {}) {
    const {initialValue = false, duration} = options;
    const [loading, setLoading] = useState(initialValue);

    const startLoading = useCallback(async () => {
        setLoading(true);
        if (duration) {
            await new Promise((resolve) => setTimeout(resolve, duration));
            setLoading(false);
        }
    }, [duration]);

    const stopLoading = useCallback(() => {
        setLoading(false);
    }, []);

    const withLoading = useCallback(
        async <T extends (...args: any[]) => Promise<any>>(fn: T, ...args: Parameters<T>) => {
            setLoading(true);
            try {
                return await fn(...args);
            } finally {
                setLoading(false);
            }
        },
        [],
    );

    return {
        loading,
        startLoading,
        stopLoading,
        withLoading,
    };
}

/**
 * 显示 Toast 提示
 */
export function useToast() {
    const showToast = useCallback((title: string, icon: 'success' | 'error' | 'none' = 'none') => {
        Taro.showToast({
            title,
            icon,
            duration: 2000,
        });
    }, []);

    return {showToast};
}

/**
 * 显示 Loading 提示
 */
export function useLoadingToast() {
    const showLoading = useCallback((title: string) => {
        Taro.showLoading({title});
    }, []);

    const hideLoading = useCallback(() => {
        Taro.hideLoading();
    }, []);

    return {showLoading, hideLoading};
}
