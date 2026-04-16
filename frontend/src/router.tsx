import type { RouteObject } from "react-router-dom";
import { Navigate } from "react-router-dom";
import { ProtectedRoute } from "./components/ProtectedRoute";
import { LoginPage } from "./pages/Login";
import { RegisterPage } from "./pages/Register";
import { ForgotPasswordPage } from "./pages/ForgotPassword";
import { ResetPasswordPage } from "./pages/ResetPassword";
import { AuthorizePage } from "./pages/Authorize";
import { ProfilePage } from "./pages/Profile";
import { MyAppsPage } from "./pages/MyApps";
import { MyAuditLogsPage } from "./pages/MyAuditLogs";
import { UserLayout } from "./components/UserLayout";
import { AdminLayout } from "./pages/admin/Layout";
import { AdminUsers } from "./pages/admin/Users";
import { AdminGroups } from "./pages/admin/Groups";
import { AdminClients } from "./pages/admin/Clients";
import { AdminAuditLogs } from "./pages/admin/AuditLogs";
import { AdminSettings } from "./pages/admin/Settings";

export const routes: RouteObject[] = [
  // 默认重定向
  { path: "/", element: <Navigate to="/login" replace /> },

  // 公开页面
  { path: "/login", element: <LoginPage /> },
  { path: "/register", element: <RegisterPage /> },
  { path: "/forgot-password", element: <ForgotPasswordPage /> },
  { path: "/reset-password", element: <ResetPasswordPage /> },

  // OIDC 授权（需要登录）
  {
    path: "/authorize",
    element: (
      <ProtectedRoute>
        <AuthorizePage />
      </ProtectedRoute>
    ),
  },
  {
    path: "/oauth/authorize",
    element: (
      <ProtectedRoute>
        <AuthorizePage />
      </ProtectedRoute>
    ),
  },

  // 用户自助（需要登录）
  {
    element: (
      <ProtectedRoute>
        <UserLayout />
      </ProtectedRoute>
    ),
    children: [
      { path: "/profile", element: <ProfilePage /> },
      { path: "/my-apps", element: <MyAppsPage /> },
      { path: "/my-audit-logs", element: <MyAuditLogsPage /> },
    ],
  },

  // 管理后台（需要管理员权限）
  {
    path: "/admin",
    element: (
      <ProtectedRoute requireAdmin>
        <AdminLayout />
      </ProtectedRoute>
    ),
    children: [
      { path: "users", element: <AdminUsers /> },
      { path: "groups", element: <AdminGroups /> },
      { path: "clients", element: <AdminClients /> },
      { path: "audit-logs", element: <AdminAuditLogs /> },
      { path: "settings", element: <AdminSettings /> },
    ],
  },
];
