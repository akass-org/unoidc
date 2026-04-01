import { Link, useLocation, Outlet, useNavigate } from 'react-router-dom'
import { 
  User, 
  AppWindow, 
  LogOut, 
  Shield, 
  Settings
} from 'lucide-react'
import { useSessionStore } from '#src/stores/session'
import { useUIConfigStore } from '#src/stores/theme'
import { ThemeToggle } from './ThemeToggle'
import { Avatar } from '#src/components/ui'

const navItems = [
  { path: '/profile', label: '个人资料', icon: User },
  { path: '/my-apps', label: '我的应用', icon: AppWindow },
]

export function UserLayout() {
  const location = useLocation()
  const navigate = useNavigate()
  const { user, logout } = useSessionStore()
  const { brandName } = useUIConfigStore()

  const handleLogout = async () => {
    await logout()
    navigate('/login')
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-black">
      {/* Header */}
      <header className="sticky top-0 z-40 bg-white/80 dark:bg-black/80 backdrop-blur-md border-b border-gray-200 dark:border-white/[0.06]">
        <div className="max-w-6xl mx-auto px-4 sm:px-6">
          <div className="flex items-center justify-between h-14">
            {/* Logo */}
            <Link to="/profile" className="flex items-center gap-2.5">
              <div className="w-7 h-7 rounded-md bg-black dark:bg-white flex items-center justify-center">
                <Shield className="w-4 h-4 text-white dark:text-black" />
              </div>
              <span className="font-medium text-sm text-gray-900 dark:text-white">
                {brandName}
              </span>
            </Link>

            {/* Right Side */}
            <div className="flex items-center gap-1">
              <ThemeToggle />

              {/* User Menu */}
              <div className="flex items-center gap-3 pl-3 ml-3 border-l border-gray-200 dark:border-white/[0.06]">
                <Link
                  to="/admin/users"
                  className="flex items-center gap-1.5 px-2 py-1.5 text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors"
                >
                  <Settings className="w-3.5 h-3.5" />
                  管理
                </Link>
                
                <div className="flex items-center gap-2">
                  <Avatar 
                    name={user?.display_name || user?.username || '?'} 
                    size="sm"
                  />
                  <span className="hidden sm:block text-xs text-gray-500 dark:text-gray-400 max-w-[100px] truncate">
                    {user?.display_name || user?.username}
                  </span>
                </div>

                <button
                  onClick={handleLogout}
                  className="p-1.5 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-white/[0.04] rounded-md transition-colors"
                  title="退出登录"
                >
                  <LogOut className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </header>

      <div className="max-w-6xl mx-auto px-4 sm:px-6 py-8">
        <div className="flex flex-col lg:flex-row gap-8">
          {/* Sidebar */}
          <aside className="lg:w-48 flex-shrink-0">
            <nav className="space-y-0.5 sticky top-20">
              {navItems.map((item) => {
                const Icon = item.icon
                const isActive = location.pathname === item.path
                
                return (
                  <Link
                    key={item.path}
                    to={item.path}
                    className={`
                      flex items-center gap-2.5 px-3 py-2 rounded-md text-sm transition-all
                      ${isActive
                        ? 'bg-black text-white dark:bg-white dark:text-black font-medium'
                        : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-white/[0.04]'
                      }
                    `}
                  >
                    <Icon className={`w-4 h-4 ${isActive ? 'text-white dark:text-black' : ''}`} />
                    {item.label}
                  </Link>
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
