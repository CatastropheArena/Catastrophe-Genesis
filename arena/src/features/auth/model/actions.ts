import {createAction, createAsyncThunk} from "@reduxjs/toolkit";
import axios, {AxiosError} from "axios";

import {
  authApi,
  CheckGameEntryData,
  GetCredentialsResponse,
  SessionTokenRequest,
  SessionTokenResponse,
  SignInData,
  SignInResponse,
  SignUpData,
  SignUpResponse,
  VerifyUsernameData,
  VerifyUsernameResponse,
} from "@shared/api/auth";
import {HTTPD} from "@shared/lib/request";
import { Credentials } from "@shared/api/common";

const prefix = "auth";

export type FetchCredentialsPayload = GetCredentialsResponse;

export const fetchCredentials = createAsyncThunk<FetchCredentialsPayload, void>(
  `${prefix}/fetchCredentials`,
  async (_, {rejectWithValue}) => {
    const {data} = await authApi.getCredentials();
    if (data.success) {
      return {
        credentials: {
          id: data.credentials.profile.id,
          username: data.credentials.profile.id,
          avatar: data.credentials.profile.avatar,
          rating: data.credentials.profile.rating,
        } as Credentials,
      };
    }else{
      return rejectWithValue(data.error);
    }
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


// 在 actions.ts 中添加
export const sessionKeyAuth = createAsyncThunk(
  `${prefix}/sessionKeyAuth`,
  async (data: SessionTokenRequest, { rejectWithValue }) => {
    try {
      const response = await authApi.getSessionToken(data);
      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        const value = (error as AxiosError<HTTPD>).response!.data
          .message as string;
        return rejectWithValue(value);
      }
      return rejectWithValue("钱包认证失败");
    }
  }
);