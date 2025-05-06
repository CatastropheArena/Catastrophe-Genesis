import { createEffect, createEvent, createStore, sample } from 'effector';
import { SessionKey } from '../lib/session-key';
import { authApi } from '@shared/api/auth';
import { requestSigner } from '@shared/lib/wallet';
import { persist } from 'effector-storage/local';

// 合约包ID，实际应用中应从配置中获取
const PACKAGE_ID = '0x123456'; // 替换为实际的包ID

// 会话有效期（分钟）
const SESSION_TTL_MIN = 10;

// Events
export const checkAuthStatus = createEvent();
export const logout = createEvent();
export const loginWithSessionKey = createEvent();
export const createGameEntryRequested = createEvent();
export const gameEntryCreated = createEvent();

// Effects
export const sessionKeyAuthFx = createEffect(async () => {
  try {
    // 1. 连接钱包
    await requestSigner.connect();
    const address = requestSigner.getAddress();
    
    if (!address) {
      throw new Error('无法获取钱包地址');
    }
    
    // 2. 创建SessionKey
    const sessionKey = new SessionKey({
      address,
      packageId: PACKAGE_ID,
      ttlMin: SESSION_TTL_MIN,
    });
    
    // 3. 请求用户签名
    await sessionKey.requestSignature();
    
    // 4. 获取证书
    const certificate = sessionKey.getCertificate();
    
    // 5. 发送到服务器进行验证
    const response = await authApi.sessionKeyAuth({
      signature: certificate.signature,
      sessionKey: certificate.sessionKey,
      address: certificate.user,
      timestamp: certificate.creationTime,
      ttlMin: certificate.ttlMin,
    });
    
    return {
      ...response.data,
      sessionKey
    };
  } catch (error) {
    console.error('SessionKey登录失败:', error);
    throw error;
  }
});

export const checkGameEntryFx = createEffect(async (address: string) => {
  try {
    const response = await authApi.checkGameEntry({ address });
    return response.data;
  } catch (error) {
    console.error('检查游戏通行证失败:', error);
    throw error;
  }
});

// Stores
export const $isAuthenticated = createStore(false);
export const $currentUser = createStore<{
  address: string;
  accessToken: string;
  refreshToken: string;
  hasGameEntry: boolean;
} | null>(null);

export const $sessionKey = createStore<SessionKey | null>(null);
export const $isAuthLoading = sessionKeyAuthFx.pending;

// 持久化用户状态
persist({ store: $currentUser, key: 'currentUser' });

// Samples and reactions
sample({
  clock: loginWithSessionKey,
  target: sessionKeyAuthFx,
});

sample({
  clock: sessionKeyAuthFx.doneData,
  fn: (data) => {
    return {
      address: data.sessionKey.getAddress(),
      accessToken: data.credentials.accessToken,
      refreshToken: data.credentials.refreshToken,
      hasGameEntry: data.hasGameEntry,
    };
  },
  target: $currentUser,
});

sample({
  clock: sessionKeyAuthFx.doneData,
  fn: ({ sessionKey }) => sessionKey,
  target: $sessionKey,
});

sample({
  clock: sessionKeyAuthFx.doneData,
  fn: () => true,
  target: $isAuthenticated,
});

sample({
  clock: logout,
  fn: () => null,
  target: $currentUser,
});

sample({
  clock: logout,
  fn: () => null,
  target: $sessionKey,
});

sample({
  clock: logout,
  fn: () => false,
  target: $isAuthenticated,
});

// 检查认证状态
sample({
  clock: checkAuthStatus,
  source: $currentUser,
  filter: Boolean,
  fn: (user) => user.address,
  target: checkGameEntryFx,
});

// 用户登录后自动检查认证状态
sample({
  clock: sessionKeyAuthFx.done,
  target: checkAuthStatus,
}); 