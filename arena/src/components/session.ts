import { create } from 'zustand';
import { SessionKey } from '@mysten/seal';
import { UseMutateFunction } from "@tanstack/react-query";

// 定义 SessionStore 的类型
interface SessionStore {
  sessionKey: SessionKey | null;
  setSessionKey: (session: SessionKey) => void;
  clearSession: () => void;
  // 新增：检查 session 是否有效
  isValidSession: () => boolean;
  // 新增：获取有效的 session，如果无效则返回 null
  getValidSession: () => SessionKey | null;
  // 新增：续签 session 的方法
  renewSession: (
    address: string, 
    packageId: string, 
    signPersonalMessage: SignPersonalMessageFn,
    ttlMin?: number
  ) => Promise<SessionKey | null>;
}

// 创建全局 session store
export const useSessionStore = create<SessionStore>((set, get) => ({
  sessionKey: null,
  setSessionKey: (session) => set({ sessionKey: session }),
  clearSession: () => set({ sessionKey: null }),
  
  // 检查 session 是否有效
  isValidSession: () => {
    const session = get().sessionKey;
    if (!session) return false;
    
    try {
      // 检查 session 是否过期
      return !session.isExpired();
    } catch (error) {
      // 如果检查过程中出现错误，认为 session 无效
      return false;
    }
  },
  
  // 获取有效的 session
  getValidSession: () => {
    const session = get().sessionKey;
    if (!session) return null;
    
    try {
      // 如果 session 已过期，清除它并返回 null
      if (session.isExpired()) {
        get().clearSession();
        return null;
      }
      return session;
    } catch (error) {
      // 如果检查过程中出现错误，清除 session 并返回 null
      get().clearSession();
      return null;
    }
  },

  // 新增：续签 session 的方法
  renewSession: async (address: string, packageId: string, signPersonalMessage: SignPersonalMessageFn, ttlMin: number=10) => {
    try {
      console.log("renewSession", address, packageId, signPersonalMessage, ttlMin)
      const sessionKey = new SessionKey({
        address,
        packageId,
        ttlMin,
      });
      
      // 包装成 Promise 来获取签名结果
      const signedSessionKey = await new Promise<SessionKey>((resolve, reject) => {
        signPersonalMessage(
          { message: sessionKey.getPersonalMessage() },
          {
            onSuccess: async (result) => {
              try {
                console.log("onSuccess", result)
                await sessionKey.setPersonalMessageSignature(result.signature);
                set({ sessionKey });
                console.log("sessionKey", sessionKey)
                resolve(sessionKey);
              } catch (error) {
                reject(error);
              }
            },
            onError: (error) => reject(error)
          }
        );
      });
      console.log("signedSessionKey", signedSessionKey)
      return signedSessionKey;
    } catch (error) {
      console.error("Session renewal failed:", error);
      return null;
    }
  },
}));

// 导出签名函数类型
export type SignPersonalMessageFn = UseMutateFunction<
  any,
  any,
  any,
  any
>;