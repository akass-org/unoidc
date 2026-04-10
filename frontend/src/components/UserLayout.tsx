import { Link, useLocation, Outlet } from 'react-router-dom'
import { 
  User, 
  AppWindow,
  ClipboardList
} from 'lucide-react'
import { LayoutHeader } from './LayoutHeader'

const navItems = [
  { path: '/profile', label: '个人资料', icon: User },
  { path: '/my-apps', label: '我的应用', icon: AppWindow },
  { path: '/my-audit-logs', label: '我的审计日志', icon: ClipboardList },
]

export function UserLayout() {
  const location = useLocation()

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-black page-content">
      <LayoutHeader isAdminPortal={false} />

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
                      flex items-center gap-2.5 px-3 py-2 rounded-md text-sm nav-item
                      ${isActive
                        ? 'bg-black text-white dark:bg-white dark:text-black font-bold'
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
          <main className="flex-1 min-w-0 page-content">
            <Outlet />
          </main>
        </div>
      </div>
    </div>
  )
}
