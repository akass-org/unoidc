import { Outlet, NavLink } from 'react-router-dom'
import { 
  Users, 
  Tags, 
  Shield, 
  ClipboardList, 
  Settings
} from 'lucide-react'
import { LayoutHeader } from '#src/components/LayoutHeader'

const navItems = [
  { to: '/admin/users', label: '用户管理', icon: Users },
  { to: '/admin/groups', label: '用户组', icon: Tags },
  { to: '/admin/clients', label: '应用管理', icon: Shield },
  { to: '/admin/audit-logs', label: '审计日志', icon: ClipboardList },
  { to: '/admin/settings', label: '系统设置', icon: Settings },
]

export function AdminLayout() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-black">
      <LayoutHeader isAdminPortal={true} />

      <div className="max-w-6xl mx-auto px-4 sm:px-6 py-8">
        <div className="flex flex-col lg:flex-row gap-8">
          {/* Sidebar */}
          <aside className="lg:w-48 flex-shrink-0">
            <nav className="space-y-0.5 sticky top-20">
              {navItems.map((item) => {
                const Icon = item.icon
                return (
                  <NavLink
                    key={item.to}
                    to={item.to}
                    className={({ isActive }) =>
                      `flex items-center gap-2.5 px-3 py-2 rounded-md text-sm transition-all ${
                        isActive
                          ? 'bg-black text-white dark:bg-white dark:text-black font-medium'
                          : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-white/[0.04]'
                      }`
                    }
                  >
                    <Icon className="w-4 h-4" />
                    {item.label}
                  </NavLink>
                )
              })}
            </nav>
          </aside>

          {/* Main Content */}
          <main className="flex-1 min-w-0">
            <Outlet />
          </main>
        </div>
      </div>
    </div>
  )
}
