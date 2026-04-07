import { Navigate, useLocation } from 'react-router-dom'
import { useSessionStore } from '#src/stores/session'
import { useEffect, useState } from 'react'

interface ProtectedRouteProps {
  children: React.ReactNode
  requireAdmin?: boolean
}

export function ProtectedRoute({ children, requireAdmin = false }: ProtectedRouteProps) {
  const { user, loading, checkSession } = useSessionStore()
  const location = useLocation()
  const [checked, setChecked] = useState(false)

  useEffect(() => {
    const verify = async () => {
      // 路由进入时总是向后端复核一次会话，避免依赖陈旧本地状态
      if (!checked) {
        await checkSession()
        setChecked(true)
      }
    }
    verify()
  }, [checked, checkSession])

  if (loading || !checked) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="w-8 h-8 border border-gray-300 dark:border-white/20 border-t-gray-900 dark:border-t-white rounded-full animate-spin" />
      </div>
    )
  }

  if (!user) {
    return <Navigate to="/login" state={{ from: location }} replace />
  }

  if (requireAdmin && !user.is_admin) {
    return <Navigate to="/profile" replace />
  }

  return <>{children}</>
}
