import { api } from './client'
export const adminApi = {
  getUsers: () => api.get('api/v1/admin/users').json(),
  createUser: (data: Record<string, unknown>) => api.post('api/v1/admin/users', { json: data }).json(),
  updateUser: (id: string, data: Record<string, unknown>) => api.patch(`api/v1/admin/users/${id}`, { json: data }).json(),
  resetUserPassword: (id: string) => api.post(`api/v1/admin/users/${id}/reset-password`).json(),

  getGroups: () => api.get('api/v1/admin/groups').json(),
  createGroup: (data: Record<string, unknown>) => api.post('api/v1/admin/groups', { json: data }).json(),
  updateGroup: (id: string, data: Record<string, unknown>) => api.patch(`api/v1/admin/groups/${id}`, { json: data }).json(),
  deleteGroup: (id: string) => api.delete(`api/v1/admin/groups/${id}`).json(),

  getClients: () => api.get('api/v1/admin/clients').json(),
  createClient: (data: Record<string, unknown>) => api.post('api/v1/admin/clients', { json: data }).json(),
  updateClient: (id: string, data: Record<string, unknown>) => api.patch(`api/v1/admin/clients/${id}`, { json: data }).json(),
  deleteClient: (id: string) => api.delete(`api/v1/admin/clients/${id}`).json(),
  resetClientSecret: (id: string) => api.post(`api/v1/admin/clients/${id}/reset-secret`).json(),

  getAuditLogs: () => api.get('api/v1/admin/audit-logs').json(),
  getSettings: () => api.get('api/v1/admin/settings').json(),
  updateSettings: (data: Record<string, unknown>) => api.patch('api/v1/admin/settings', { json: data }).json(),
  rotateKey: () => api.post('api/v1/admin/keys/rotate').json(),
}
