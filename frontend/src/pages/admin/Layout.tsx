import { Outlet, NavLink, Link } from 'react-router-dom'
import { useSessionStore } from '#src/stores/session'
import { ThemeToggle } from '#src/components/ThemeToggle'
import { useUIConfigStore } from '#src/stores/theme'

const navItems = [
  { to: '/admin/users', label: '用户管理', icon: '👥' },
  { to: '/admin/groups', label: '用户组', icon: '🏷️' },
  { to: '/admin/clients', label: 'Client 管理', icon: '🔐' },
  { to: '/admin/audit-logs', label: '审计日志', icon: '📋' },
  { to: '/admin/settings', label: '系统设置', icon: '⚙️' },
]

export function AdminLayout() {
  const { user } = useSessionStore()
  const { brandName } = useUIConfigStore()

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
      {/* Header */}
      <header className="sticky top-0 z-50 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-8">
              <Link to="/admin/users" className="flex items-center gap-2">
                <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                  <span className="text-white text-sm font-bold">🔐</span>
                </div>
                <span className="font-bold text-gray-900 dark:text-white">{brandName}</span>
              </Link>
              <span className="text-sm text-gray-500 dark:text-gray-400">管理后台</span>
            </div>
            <div className="flex items-center gap-4">
              <ThemeToggle />
              <div className="flex items-center gap-3 pl-4 border-l border-gray-200 dark:border-gray-700">
                <Link
                  to="/profile"
                  className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                >
                  返回前台
                </Link>
                <div className="w-8 h-8 rounded-full bg-gradient-to-br from-green-400 to-blue-500 flex items-center justify-center text-white text-sm font-medium">
                  {user?.display_name?.charAt(0) || user?.username?.charAt(0) || '?'}
                </div>
              </div>
            </div>
          </div>
        </div>
      </header>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="flex flex-col lg:flex-row gap-8">
          {/* Sidebar */}
          <aside className="lg:w-64 flex-shrink-0">
            <nav className="space-y-1">
              {navItems.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  className={({ isActive }) =>
                    `flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-colors ${
                      isActive
                        ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300'
                        : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'
                    }`
                  }
                >
                  <span>{item.icon}</span>
                  {item.label}
                </NavLink>
              ))}
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
