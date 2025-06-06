import {AxiosPromise} from "axios";

import {request} from "@shared/lib/request";
import {Credentials} from "./common";

export interface FetchCredentialsResponse {
  credentials: Credentials;
}

export interface SessionUser {
  user_address: string;
  session_vk: string;
  exp: number;
  profile: Profile;
}

/// 获取用户Profile响应结构
export interface AuthCredentialsResponse {
    success: boolean,
    credentials: SessionUser,
    error: string,
}


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


export interface LogoutResponse {
  success: boolean;
  message: string;
}

const authCredentials = (): AxiosPromise<AuthCredentialsResponse> =>
  request({url: "/auth/credentials"});


const authSessionToken = (data: SessionTokenRequest): AxiosPromise<SessionTokenResponse> =>
  request({url: "/auth/session_token", method: "POST", data});


const authSessionLogout = (): Promise<LogoutResponse> =>
  request({ url: "/auth/session_logout", method: "POST", withCredentials: true }).then(res => res.data);


export const authApi = {
  authCredentials,
  authSessionLogout,
  authSessionToken,
};
