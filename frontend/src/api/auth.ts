import { api } from './client'

// TODO: 实现认证 API
export const authApi = {
  login: (username: string, password: string) =>
    api.post('api/v1/auth/login', { json: { username, password } }).json(),

  register: (data: Record<string, string>) =>
    api.post('api/v1/auth/register', { json: data }).json(),

  logout: () =>
    api.post('api/v1/auth/logout').json(),

  forgotPassword: (email: string) =>
    api.post('api/v1/auth/forgot-password', { json: { email } }).json(),

  resetPassword: (token: string, password: string) =>
    api.post('api/v1/auth/reset-password', { json: { token, password } }).json(),

  getSession: () =>
    api.get('api/v1/auth/session').json(),
}
