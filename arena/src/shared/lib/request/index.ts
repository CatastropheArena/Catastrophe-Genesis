import axios, { InternalAxiosRequestConfig, AxiosHeaders } from "axios";
import { useAuthStore } from "src/components/auth";

// 定义 SDK 版本常量
const PACKAGE_VERSION = "1.0.0"; // 请根据实际版本号修改

export const request = axios.create({
  baseURL: import.meta.env.VITE_BACKEND_URL,
  withCredentials: true,
  headers: {
    'Content-Type': 'application/json',
    'Client-Sdk-Type': 'typescript',
    'Client-Sdk-Version': PACKAGE_VERSION,
  },
});

// 添加请求拦截器来动态设置 Request-Id 和 Authorization
request.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  if (!config.headers) {
    config.headers = new AxiosHeaders();
  }
  // 生成唯一的请求 ID
  const requestId = crypto.randomUUID();
  config.headers['Request-Id'] = requestId;

  // 添加 auth token
  const token = useAuthStore.getState().token;
  if (token) {
    config.headers['Authorization'] = `Bearer ${token}`;
  }

  return config;
});

export interface HTTPD {
  error: string;
  message: string | string[];
  statusCode: number;
}
