import React, { useEffect, useMemo } from "react";
import {ThemeProvider} from "@mui/material";
import {useSelector} from "react-redux";

import {themes} from "./lib/themes";
import {palette} from "./lib/palette";
import {model} from "./model";

interface ThemingProviderProps {
  children: React.ReactNode;
}

export const ThemingProvider: React.FC<ThemingProviderProps> = ({children}) => {
  const currentTheme = useSelector(model.selectors.theme);

  // 计算实际应用的主题模式
  const effectiveTheme = useMemo(() => {
    if (currentTheme === 'system') {
      // 如果是系统主题，检查系统偏好
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return currentTheme;
  }, [currentTheme]);

  // 获取对应的 MUI 主题
  const theme = themes[effectiveTheme];

  // 调整调色板
  palette.adjust(theme.palette);

  // 监听系统主题变化
  useEffect(() => {
    const darkModeMediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    
    const handleDarkModeChange = (e: MediaQueryListEvent) => {
      if (currentTheme === 'system') {
        // 只有当设置为"system"时才响应系统主题变化
        document.documentElement.classList.toggle('dark', e.matches);
      }
    };

    // 添加系统主题变化的监听器
    darkModeMediaQuery.addEventListener('change', handleDarkModeChange);
    
    return () => {
      darkModeMediaQuery.removeEventListener('change', handleDarkModeChange);
    };
  }, [currentTheme]);

  // 应用主题到 HTML 元素，以便 Tailwind 的暗色模式可以正常工作
  useEffect(() => {
    // 保存主题偏好到 localStorage
    localStorage.setItem('theme', currentTheme);
    
    // 更新 HTML 的类名
    if (currentTheme === 'system') {
      // 如果是系统主题，根据系统偏好设置
      const isDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
      document.documentElement.classList.toggle('dark', isDarkMode);
    } else {
      // 直接使用用户设置的主题
      document.documentElement.classList.toggle('dark', currentTheme === 'dark');
    }
  }, [currentTheme]);

  return <ThemeProvider theme={theme}>{children}</ThemeProvider>;
};
