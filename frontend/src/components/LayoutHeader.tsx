import { Link, useNavigate } from 'react-router-dom'
import { LogOut, Settings, Monitor, LayoutDashboard } from 'lucide-react'
import { useSessionStore } from '#src/stores/session'
import { useUIConfigStore } from '#src/stores/theme'
import { ThemeToggle } from './ThemeToggle'
import { PortalSwitchButton } from './PortalSwitchButton'
import { Avatar } from '#src/components/ui'

interface LayoutHeaderProps {
  isAdminPortal: boolean
}

export function LayoutHeader({ isAdminPortal }: LayoutHeaderProps) {
  const { user, logout } = useSessionStore()
  const { brandName } = useUIConfigStore()
  const navigate = useNavigate()

  const handleLogout = async () => {
    await logout()
    navigate('/login')
  }

  return (
    <header className="sticky top-0 z-40 bg-white/80 dark:bg-black/80 backdrop-blur-md border-b border-gray-200 dark:border-white/[0.06]">
      <div className="max-w-6xl mx-auto px-4 sm:px-6">
        <div className="flex items-center justify-between h-14">
          {/* Logo */}
          {isAdminPortal ? (
            <Link to="/admin/users" className="flex items-center gap-2.5">
              <div className="w-7 h-7 rounded-md bg-black dark:bg-white flex items-center justify-center">
                <LayoutDashboard className="w-4 h-4 text-white dark:text-black" />
              </div>
              <div className="flex items-center gap-2">
                <span className="font-medium text-sm text-gray-900 dark:text-white">
                  {brandName}
                </span>
                <span className="text-[10px] text-gray-500 dark:text-gray-600 px-1.5 py-0.5 bg-gray-100 dark:bg-white/[0.04] rounded">
                  管理
                </span>
              </div>
            </Link>
          ) : (
            <Link to="/profile" className="flex items-center gap-2.5">
              <div className="w-7 h-7 rounded-md bg-black dark:bg-white flex items-center justify-center">
                <Settings className="w-4 h-4 text-white dark:text-black" />
              </div>
              <span className="font-medium text-sm text-gray-900 dark:text-white">
                {brandName}
              </span>
            </Link>
          )}

          {/* Right Side */}
          <div className="flex items-center gap-1">
            <ThemeToggle />

            {/* User Menu */}
            <div className="flex items-center gap-2 pl-3 ml-2 border-l border-gray-200 dark:border-white/[0.06]">
              {isAdminPortal ? (
                <PortalSwitchButton to="/profile" label="前台" icon={Monitor} />
              ) : (
                user?.is_admin && (
                  <PortalSwitchButton to="/admin/users" label="管理" icon={Settings} />
                )
              )}

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
  )
}
