import {useDispatch, useSelector} from "react-redux";
import { useMemo } from "react";

import * as selectors from "./selectors";
import * as actions from "./actions";
import {Theme} from "../lib/typings";

export const useTheme = () => {
  const theme = useSelector(selectors.theme);
  const dispatch = useDispatch();
  
  // 计算实际应用的主题（考虑系统主题）
  const effectiveTheme = useMemo(() => {
    if (theme === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return theme;
  }, [theme]);
  
  const toggleTheme = () => {
    // 在 light、dark 和 system 之间循环切换
    let newTheme: Theme;
    
    switch (theme) {
      case "light":
        newTheme = "dark";
        break;
      case "dark":
        newTheme = "system";
        break;
      case "system":
      default:
        newTheme = "light";
        break;
    }
    
    dispatch(actions.setTheme(newTheme));
  };
  
  const setTheme = (theme: Theme) => {
    dispatch(actions.setTheme(theme));
  };
  
  return { 
    mode: theme,
    effectiveTheme, // 添加有效主题
    toggleTheme,
    setTheme
  };
};
