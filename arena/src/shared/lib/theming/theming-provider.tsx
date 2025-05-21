import React, { useEffect } from "react";
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

  // 获取对应的 MUI 主题
  const theme = themes[currentTheme];

  // 调整调色板
  palette.adjust(theme.palette);

  // 应用主题到 HTML 元素，以便 Tailwind 的暗色模式可以正常工作
  useEffect(() => {
    // 保存主题偏好到 localStorage
    localStorage.setItem('theme', currentTheme);
    
    // 更新 HTML 的类名
    document.documentElement.classList.toggle('dark', currentTheme === 'dark');
  }, [currentTheme]);

  return <ThemeProvider theme={theme}>{children}</ThemeProvider>;
};
