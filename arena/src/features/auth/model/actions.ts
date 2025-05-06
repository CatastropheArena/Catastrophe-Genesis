import {createAction, createAsyncThunk} from "@reduxjs/toolkit";
import axios, {AxiosError} from "axios";

import {
  authApi,
  GetCredentialsResponse,
  SignInData,
  SignInResponse,
  SignUpData,
  SignUpResponse,
  VerifyUsernameData,
  VerifyUsernameResponse,
} from "@shared/api/auth";
import {HTTPD} from "@shared/lib/request";

const prefix = "auth";

export type FetchCredentialsPayload = GetCredentialsResponse;

export const fetchCredentials = createAsyncThunk<FetchCredentialsPayload, void>(
  `${prefix}/fetchCredentials`,
  async () => {
    const {data} = await authApi.getCredentials();

    return data;
  },
);

export type SignUpPayload = SignUpResponse;
export type SignUpOptions = SignUpData;

export const signUp = createAsyncThunk<SignUpPayload, SignUpOptions>(
  `${prefix}/signUp`,
  async (options, {rejectWithValue}) => {
    try {
      const {data} = await authApi.signUp(options);

      return data;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        const value = (error as AxiosError<HTTPD>).response!.data
          .message[0] as string;

        return rejectWithValue(value);
      }

      return rejectWithValue(null);
    }
  },
);

export type SignInPayload = SignInResponse;
export type SignInOptions = SignInData;

export const signIn = createAsyncThunk<SignInPayload, SignInOptions>(
  `${prefix}/signIn`,
  async (options, {rejectWithValue}) => {
    try {
      const {data} = await authApi.signIn(options);

      return data;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        const value = (error as AxiosError<HTTPD>).response!.data
          .message as string;

        return rejectWithValue(value);
      }

      return rejectWithValue(null);
    }
  },
);

export type VerifyUsernamePayload = VerifyUsernameResponse;
export type VerifyUsernameOptions = VerifyUsernameData;

export const verifyUsername = createAsyncThunk<
  VerifyUsernamePayload,
  VerifyUsernameOptions
>(`${prefix}/verifyUsername`, async (options) => {
  const {data} = await authApi.verifyUsername(options);

  return data;
});

export interface SetAreCredentialsFetchingPayload {
  areFetching: boolean;
}

export const setAreCredentialsFetching =
  createAction<SetAreCredentialsFetchingPayload>(
    `${prefix}/setAreCredentialsFetching`,
  );

// 新增 SessionKey 认证方法
export const sessionKeyAuth = createAsyncThunk(
  "auth/sessionKeyAuth",
  async (data: authApi.SessionKeyAuthData, { rejectWithValue }) => {
    try {
      const response = await authApi.sessionKeyAuth(data);
      return response.data;
    } catch (error) {
      return rejectWithValue(
        error?.response?.data?.message || "钱包认证失败",
      );
    }
  },
);

// 新增 检查游戏通行证方法
export const checkGameEntry = createAsyncThunk(
  "auth/checkGameEntry",
  async (data: authApi.CheckGameEntryData, { rejectWithValue }) => {
    try {
      const response = await authApi.checkGameEntry(data);
      return response.data;
    } catch (error) {
      return rejectWithValue(
        error?.response?.data?.message || "检查游戏通行证失败",
      );
    }
  },
);
