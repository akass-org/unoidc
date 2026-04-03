import { api } from './client'

// TODO: 实现用户自助 API
export const meApi = {
  getProfile: () => api.get('api/v1/me').json(),
  updateProfile: (data: Record<string, unknown>) => api.patch('api/v1/me', { json: data }).json(),
  changePassword: (data: { current_password: string; new_password: string }) =>
    api.post('api/v1/me/password', { json: data }).json(),
  uploadAvatar: (file: File) => {
    const formData = new FormData()
    formData.append('avatar', file)
    return api.post('api/v1/me/avatar', { body: formData }).json()
  },
  getApps: () => api.get('api/v1/me/apps').json(),
  getAuditLogs: () => api.get('api/v1/me/audit-logs').json(),
  getConsents: () => api.get('api/v1/me/consents').json(),
  revokeConsent: (clientId: string) => api.delete(`api/v1/me/consents/${clientId}`).json(),
}
