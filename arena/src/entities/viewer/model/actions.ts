import {createAction, createAsyncThunk} from "@reduxjs/toolkit";

import {Credentials, Profile, User} from "@shared/api/common";

import {
  GetUserResponse,
  GetMyFriendsResponse,
  GetMyMatchesResponse,
  GetMyOngoingMatchData,
  GetMyOngoingMatchResponse,
  GetMyStatsResponse,
  profileApi,
} from "@shared/api/profile";
import { authApi } from "@shared/api/auth";

const prefix = "viewer";

export type FetchProfilePayload = GetUserResponse;

export const fetchProfile = createAsyncThunk<
  FetchProfilePayload,
  void
>(`${prefix}/fetchProfile`, async (_, { rejectWithValue }) => {
  const { data } = await authApi.authCredentials();
  if (data.success) {
    return {
      user: {
        id: data.credentials.profile.id,
        username: data.credentials.profile.id,
        avatar: data.credentials.profile.avatar,
        rating: data.credentials.profile.rating,
        relationship: 0,
      } as Profile,
    };
  } else {
    return rejectWithValue(data.error);
  }
});

export type FetchMatchesPayload = GetMyMatchesResponse;
export type FetchMatchesOptions = void;

export const fetchMatches = createAsyncThunk<
  FetchMatchesPayload,
  FetchMatchesOptions
>(`${prefix}/fetchMatches`, async () => {
  const {data} = await profileApi.getMyMatches();

  return data;
});

export type FetchFriendsPayload = GetMyFriendsResponse;
export type FetchFriendsOptions = void;

export const fetchFriends = createAsyncThunk<
  FetchFriendsPayload,
  FetchFriendsOptions
>(`${prefix}/fetchFriends`, async () => {
  const {data} = await profileApi.getMyFriends();

  return data;
});

export type FetchStatsPayload = GetMyStatsResponse;
export type FetchStatsOptions = void;

export const fetchStats = createAsyncThunk<
  FetchStatsPayload,
  FetchStatsOptions
>(`${prefix}/fetchStats`, async () => {
  const {data} = await profileApi.getMyStats();

  return data;
});

export type FetchOngoingMatchPayload = GetMyOngoingMatchResponse;
export type FetchOngoingMatchOptions = GetMyOngoingMatchData;

export const fetchOngoingMatch = createAsyncThunk<
  FetchOngoingMatchPayload,
  FetchOngoingMatchOptions
>(`${prefix}/fetchOngoingMatch`, async () => {
  const {data} = await profileApi.getMyOngoingMatch();

  return data;
});

export interface SetCredentialsPayload {
  credentials: Credentials;
}

export const setCredentials = createAction<SetCredentialsPayload>(
  `${prefix}/setCredentials`,
);

export interface AddFriendPayload {
  friend: User;
}

export const addFriend = createAction<AddFriendPayload>(`${prefix}/addFriend`);

export interface RemoveFriendPayload {
  friendId: string;
}

export const removeFriend = createAction<RemoveFriendPayload>(
  `${prefix}/removeFriend`,
);
