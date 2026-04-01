import { Outlet, NavLink, Link, useNavigate } from 'react-router-dom'
import { 
  Users, 
  Tags, 
  Shield, 
  ClipboardList, 
  Settings,
  LogOut,
  LayoutDashboard
} from 'lucide-react'
import { useSessionStore } from '#src/stores/session'
import { useUIConfigStore } from '#src/stores/theme'
import { ThemeToggle } from '#src/components/ThemeToggle'
import { Avatar } from '#src/components/ui'

const navItems = [
  { to: '/admin/users', label: '用户管理', icon: Users },
  { to: '/admin/groups', label: '用户组', icon: Tags },
  { to: '/admin/clients', label: '应用管理', icon: Shield },
  { to: '/admin/audit-logs', label: '审计日志', icon: ClipboardList },
  { to: '/admin/settings', label: '系统设置', icon: Settings },
]

export function AdminLayout() {
  const { user, logout } = useSessionStore()
  const { brandName } = useUIConfigStore()
  const navigate = useNavigate()

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
            <div className="flex items-center gap-6">
              <Link to="/admin/users" className="flex items-center gap-2.5">
                <div className="w-7 h-7 rounded-md bg-black dark:bg-white flex items-center justify-center">
                  <LayoutDashboard className="w-4 h-4 text-white dark:text-black" />
                </div>
                <div className="flex items-center gap-2">
                  <span className="font-medium text-sm text-gray-900 dark:text-white">{brandName}</span>
                  <span className="text-[10px] text-gray-500 dark:text-gray-600 px-1.5 py-0.5 bg-gray-100 dark:bg-white/[0.04] rounded">管理</span>
                </div>
              </Link>
            </div>
            
            <div className="flex items-center gap-1">
              <ThemeToggle />
              
              <div className="flex items-center gap-2 pl-3 ml-2 border-l border-gray-200 dark:border-white/[0.06]">
                <Link
                  to="/profile"
                  className="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors"
                >
                  前台
                </Link>
                
                <div className="flex items-center gap-2">
                  <Avatar 
                    name={user?.display_name || user?.username || '?'} 
                    size="sm" 
                  />
                  <span className="hidden sm:block text-xs text-gray-500 dark:text-gray-400">
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
