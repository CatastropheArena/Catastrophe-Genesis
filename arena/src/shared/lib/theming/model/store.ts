import {createReducer, PayloadAction} from "@reduxjs/toolkit";

import * as actions from "./actions";
import {Theme} from "../lib/typings";

export interface ThemingReducer {
  theme: Theme;
}

export const store = createReducer<ThemingReducer>(
  {
    theme: (localStorage.getItem("theme") as Theme) || window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light',
  },
  {
    [actions.setTheme.type]: (
      state,
      {payload}: PayloadAction<actions.SetThemePayload>,
    ) => {
      state.theme = payload;
      localStorage.setItem("theme", payload);
    },
  },
);
