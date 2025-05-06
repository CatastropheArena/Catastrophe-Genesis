import {AxiosPromise} from "axios";

import {request} from "@shared/lib/request";
import { getAllowlistedKeyServers, SealClient, SessionKey } from '@mysten/seal';
import {Credentials, FormVerification} from "./common";

export interface GetCredentialsResponse {
  credentials: Credentials;
}

const getCredentials = (): AxiosPromise<GetCredentialsResponse> =>
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

export interface SessionKeyAuthData {
  signature: string;
  sessionKey: string;
  address: string;
  timestamp: number;
  ttlMin: number;
}

export interface SessionKeyAuthResponse {
  credentials: Credentials;
  hasGameEntry: boolean;
  isNewUser: boolean;
}

const sessionKeyAuth = (data: SessionKeyAuthData): AxiosPromise<SessionKeyAuthResponse> =>
  request({url: "/auth/session-key", method: "POST", data});

export interface CheckGameEntryData {
  address: string;
}

export interface CheckGameEntryResponse {
  hasGameEntry: boolean;
  passportId?: string;
}

const checkGameEntry = (data: CheckGameEntryData): AxiosPromise<CheckGameEntryResponse> =>
  request({url: "/auth/check-game-entry", method: "POST", data});

export interface VerifyUsernameData {
  username: string;
}

export type VerifyUsernameResponse = FormVerification;

const verifyUsername = (
  data: VerifyUsernameData,
): AxiosPromise<VerifyUsernameResponse> =>
  request({url: "/auth/verify/username", method: "POST", data});

export const authApi = {
  getCredentials,
  signUp,
  signIn,
  verifyUsername,
  sessionKeyAuth,
  checkGameEntry
};
