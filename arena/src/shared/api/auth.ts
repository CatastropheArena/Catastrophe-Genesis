import {AxiosPromise} from "axios";

import {request} from "@shared/lib/request";
import {Credentials, FormVerification} from "./common";

export interface GetCredentialsResponse {
  credentials: Credentials;
}

export interface SessionUser {
  user_address: string;
  session_vk: string;
  exp: number;
  profile: Profile;
}

/// 获取用户Profile响应结构
export interface GetUserCredentialsResponse {
    success: boolean,
    credentials: SessionUser,
    error: string,
}

const getCredentials = (): AxiosPromise<GetUserCredentialsResponse> =>
  request({url: "/auth/credentials"});

export interface SignUpData {
  username: string;
  password: string;
}

export interface SignUpResponse {
  credentials: Credentials;
}

const signUp = (data: SignUpData): AxiosPromise<SignUpResponse> =>
  request({url: "/auth/register", method: "POST", data});

export interface SignInData {
  username: string;
  password: string;
}

export interface SignInResponse {
  credentials: Credentials;
}

const signIn = (data: SignInData): AxiosPromise<SignInResponse> =>
  request({url: "/auth/login", method: "POST", data});

export interface VerifyUsernameData {
  username: string;
}

export type VerifyUsernameResponse = FormVerification;

const verifyUsername = (
  data: VerifyUsernameData,
): AxiosPromise<VerifyUsernameResponse> =>
  request({url: "/auth/verify/username", method: "POST", data});

/// 证书数据结构
export type Certificate = {
  user: string;
  session_vk: string;
  creation_time: number;
  ttl_min: number;
  signature: string;
};
/// 会话令牌请求结构
export interface SessionTokenRequest {
  ptb: string;
  enc_key: string;
  enc_verification_key: string;
  request_signature: string;
  certificate: Certificate;
}
/// Profile数据结构
export interface Profile {
    id: string,
    avatar: string,
    rating: number,
    played: number,
    won: number,
    lost: number,
}
/// 会话令牌响应结构
export interface SessionTokenResponse {
  auth_token: string;
  expires_at: number;
  profile: Profile;
}


export interface CheckGameEntryData {
  address: string;
}

export interface CheckGameEntryResponse {
  hasGameEntry: boolean;
  passportId?: string;
  }

const checkGameEntry = (data: CheckGameEntryData): AxiosPromise<CheckGameEntryResponse> =>
  request({url: "/auth/check-game-entry", method: "POST", data});


const getSessionToken = (data: SessionTokenRequest): AxiosPromise<SessionTokenResponse> =>
  request({url: "/auth/session_token", method: "POST", data});

export interface LogoutResponse {
  success: boolean;
  message: string;
}

const signOut = (): Promise<LogoutResponse> =>
  request({ url: "/auth/session_logout", method: "POST", withCredentials: true }).then(res => res.data);


export const authApi = {
  getCredentials,
  signUp,
  signIn,
  signOut,
  verifyUsername,
  getSessionToken,
  checkGameEntry,
};
