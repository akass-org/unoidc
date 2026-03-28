import type { RouteObject } from 'react-router-dom'
import { LoginPage } from './pages/Login'
import { RegisterPage } from './pages/Register'
import { ForgotPasswordPage } from './pages/ForgotPassword'
import { AuthorizePage } from './pages/Authorize'
import { ProfilePage } from './pages/Profile'
import { MyAppsPage } from './pages/MyApps'
import { AdminLayout } from './pages/admin/Layout'
import { AdminUsers } from './pages/admin/Users'
import { AdminGroups } from './pages/admin/Groups'
import { AdminClients } from './pages/admin/Clients'
import { AdminAuditLogs } from './pages/admin/AuditLogs'
import { AdminSettings } from './pages/admin/Settings'

export const routes: RouteObject[] = [
  // 公开页面
  { path: '/login', element: <LoginPage /> },
  { path: '/register', element: <RegisterPage /> },
  { path: '/forgot-password', element: <ForgotPasswordPage /> },

  // OIDC 授权
  { path: '/authorize', element: <AuthorizePage /> },

  // 用户自助
  { path: '/profile', element: <ProfilePage /> },
  { path: '/my-apps', element: <MyAppsPage /> },

  // 管理后台
  {
    path: '/admin',
    element: <AdminLayout />,
    children: [
      { path: 'users', element: <AdminUsers /> },
      { path: 'groups', element: <AdminGroups /> },
      { path: 'clients', element: <AdminClients /> },
      { path: 'audit-logs', element: <AdminAuditLogs /> },
      { path: 'settings', element: <AdminSettings /> },
    ],
  },
]
