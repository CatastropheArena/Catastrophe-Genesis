import { createAction, createAsyncThunk } from "@reduxjs/toolkit";
import axios, { AxiosError } from "axios";

import {
  authApi,
  FetchCredentialsResponse,
  LogoutResponse,
  SessionTokenRequest,
  SessionTokenResponse,
} from "@shared/api/auth";
import { HTTPD } from "@shared/lib/request";
import { Credentials } from "@shared/api/common";

const prefix = "auth";

export interface SetAreCredentialsFetchingPayload {
  areFetching: boolean;
}

export const setAreCredentialsFetching =
  createAction<SetAreCredentialsFetchingPayload>(
    `${prefix}/setAreCredentialsFetching`
  );

export const fetchCredentials = createAsyncThunk<
  FetchCredentialsResponse,
  void
>(`${prefix}/fetchCredentials`, async (_, { rejectWithValue }) => {
  const { data } = await authApi.authCredentials();
  if (data.success) {
    return {
      credentials: {
        id: data.credentials.profile.id,
        username: data.credentials.profile.id,
        avatar: data.credentials.profile.avatar,
        rating: data.credentials.profile.rating,
      } as Credentials,
    };
  } else {
    return rejectWithValue(data.error);
  }
});

export const signIn = createAsyncThunk<
  SessionTokenResponse,
  SessionTokenRequest
>(`${prefix}/signIn`, async (data, { rejectWithValue }) => {
  try {
    const response = await authApi.authSessionToken(data);
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      const value = (error as AxiosError<HTTPD>).response!.data
        .message as string;
      return rejectWithValue(value);
    }
    return rejectWithValue("登录认证失败");
  }
});

export const signOut = createAsyncThunk<LogoutResponse, void>(
  `${prefix}/signOut`,
  async (_, { rejectWithValue }) => {
    try {
      const response = await authApi.authSessionLogout();
      return response;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        const value = (error as AxiosError<HTTPD>).response!.data
          .message as string;
        return rejectWithValue(value);
      }
      return rejectWithValue("退出登录失败");
    }
  }
);
