import { api } from "./client";
export const authApi = {
  login: (username: string, password: string) =>
    api.post("api/v1/auth/login", { json: { username, password } }).json(),

  register: (data: Record<string, string>) =>
    api.post("api/v1/auth/register", { json: data }).json(),

  logout: () => api.post("api/v1/auth/logout").json(),

  forgotPassword: (email: string) =>
    api.post("api/v1/auth/forgot-password", { json: { email } }).json(),

  resetPassword: (token: string, password: string) =>
    api.post("api/v1/auth/reset-password", { json: { token, password } }).json(),

  getSession: () => api.get("api/v1/auth/session").json(),

  // 获取公共配置（用于登录页）
  getPublicConfig: () =>
    api.get("api/v1/public/config").json<{
      brand_name: string;
      logo_url: string;
      login_background_url: string;
      login_layout: "split-left" | "split-right" | "centered" | "fullscreen";
      enable_password_login: boolean;
      enable_passkey_signup: boolean;
    }>(),
};
