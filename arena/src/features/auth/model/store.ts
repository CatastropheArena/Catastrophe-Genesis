import {createReducer, PayloadAction} from "@reduxjs/toolkit";

import * as actions from "./actions";

export interface AuthState {
  isAuthenticated: boolean;
  areCredentialsFetching: boolean;
}

export const store = createReducer<AuthState>(
  {
    isAuthenticated: false,
    areCredentialsFetching: false,
  },
  {
    [actions.fetchCredentials.fulfilled.type]: (state) => {
      state.isAuthenticated = true;
    },
    [actions.signIn.fulfilled.type]: (state) => {
      console.log('signIn fulfilled, setting isAuthenticated to true');
      state.isAuthenticated = true;
    },
    [actions.signIn.rejected.type]: (state) => {
      console.log('sessionKeyAuth rejected, setting isAuthenticated to false');
      state.isAuthenticated = false;
    },
    [actions.signOut.fulfilled.type]: (state) => {
      console.log('signOut fulfilled, setting isAuthenticated to false');
      state.isAuthenticated = false;
    },
    [actions.setAreCredentialsFetching.type]: (
      state,
      {payload}: PayloadAction<actions.SetAreCredentialsFetchingPayload>,
    ) => {
      state.areCredentialsFetching = payload.areFetching;
    },
  },
);
