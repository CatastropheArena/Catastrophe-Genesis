import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface AuthStore {
  token: string | null;
  setToken: (token: string) => void;
  clearToken: () => void;
  isValidToken: () => boolean;
}

export const useAuthStore = create<AuthStore>()(
  persist(
    (set, get) => ({
      token: null,
      setToken: (token) => set({ token }),
      clearToken: () => set({ token: null }),
      isValidToken: () => {
        const token = get().token;
        if (!token) return false;
        
        try {
          // 解析 JWT token 检查是否过期
          const payload = JSON.parse(atob(token.split('.')[1]));
          return payload.exp * 1000 > Date.now();
        } catch (error) {
          return false;
        }
      },
    }),
    {
      name: 'auth-storage',
    }
  )
); 