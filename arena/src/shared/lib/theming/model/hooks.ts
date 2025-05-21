import {useDispatch, useSelector} from "react-redux";
import { useMemo } from "react";

import * as selectors from "./selectors";
import * as actions from "./actions";
import {Theme} from "../lib/typings";

export const useTheme = () => {
  const theme = useSelector(selectors.theme);
  const dispatch = useDispatch();
  
  // 不再需要计算effectiveTheme，因为没有system选项
  
  const toggleTheme = () => {
    // 只在 light 和 dark 之间切换
    const newTheme: Theme = theme === "light" ? "dark" : "light";
    dispatch(actions.setTheme(newTheme));
  };
  
  const setTheme = (theme: Theme) => {
    dispatch(actions.setTheme(theme));
  };
  
  return { 
    mode: theme,
    toggleTheme,
    setTheme
  };
};
