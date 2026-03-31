import { Link, useLocation, Outlet } from 'react-router-dom'
import { useSessionStore } from '#src/stores/session'
import { ThemeToggle } from '#src/components/ThemeToggle'
import { useUIConfigStore } from '#src/stores/theme'

const navItems = [
  { path: '/profile', label: '个人资料', icon: '👤' },
  { path: '/my-apps', label: '我的应用', icon: '📱' },
]

const adminNavItems = [
  { path: '/admin/users', label: '管理后台', icon: '⚙️' },
]

export function UserLayout() {
  const location = useLocation()
  const { user, logout } = useSessionStore()
  const { brandName } = useUIConfigStore()

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
      {/* Header */}
      <header className="sticky top-0 z-50 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            {/* Logo */}
            <Link to="/profile" className="flex items-center gap-2">
              <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                <span className="text-white text-sm font-bold">🔐</span>
              </div>
              <span className="font-bold text-gray-900 dark:text-white">
                {brandName}
              </span>
            </Link>

            {/* Right Side */}
            <div className="flex items-center gap-4">
              <ThemeToggle />

              {/* User Menu */}
              <div className="flex items-center gap-3 pl-4 border-l border-gray-200 dark:border-gray-700">
                <div className="text-right hidden sm:block">
                  <p className="text-sm font-medium text-gray-900 dark:text-white">
                    {user?.display_name || user?.username}
                  </p>
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    {user?.email}
                  </p>
                </div>
                <div className="w-9 h-9 rounded-full bg-gradient-to-br from-green-400 to-blue-500 flex items-center justify-center text-white font-medium">
                  {user?.display_name?.charAt(0) || user?.username?.charAt(0) || '?'}
                </div>
                <button
                  onClick={logout}
                  className="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                  title="退出登录"
                >
                  🚪
                </button>
              </div>
            </div>
          </div>
        </div>
      </header>

      <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="flex flex-col lg:flex-row gap-8">
          {/* Sidebar */}
          <aside className="lg:w-64 flex-shrink-0">
            <nav className="space-y-1">
              {navItems.map((item) => (
                <Link
                  key={item.path}
                  to={item.path}
                  className={`
                    flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-colors
                    ${location.pathname === item.path
                      ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300'
                      : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'
                    }
                  `}
                >
                  <span>{item.icon}</span>
                  {item.label}
                </Link>
              ))}

              {user?.is_admin && (
                <>
                  <div className="pt-4 mt-4 border-t border-gray-200 dark:border-gray-700">
                    <p className="px-4 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
                      管理
                    </p>
                    {adminNavItems.map((item) => (
                      <Link
                        key={item.path}
                        to={item.path}
                        className="
                          flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium
                          text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800
                          transition-colors
                        "
                      >
                        <span>{item.icon}</span>
                        {item.label}
                      </Link>
                    ))}
                  </div>
                </>
              )}
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
