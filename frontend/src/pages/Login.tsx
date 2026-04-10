import { useState } from 'react'
import { useNavigate, useSearchParams, Link } from 'react-router-dom'
import { Eye, EyeOff, Shield } from 'lucide-react'
import { useSessionStore } from '#src/stores/session'
import { useUIConfigStore } from '#src/stores/theme'
import { getErrorMessage } from '#src/api/client'
import { authApi } from '#src/api/auth'
import { LoginPageWrapper } from '#src/components/LoginLayout'
import { ThemeToggle } from '#src/components/ThemeToggle'
import { Input } from '#src/components/ui'

export function LoginPage() {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const { brandName } = useUIConfigStore()
  const { setUser } = useSessionStore()

  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const rawReturnTo = searchParams.get('return_to') || '/profile'
  const returnTo = (() => {
    try {
      // 仅允许站内路径或同源绝对 URL，拒绝协议相对 URL（//example.com）
      const url = rawReturnTo.startsWith('/')
        ? new URL(rawReturnTo, window.location.origin)
        : new URL(rawReturnTo)

      if (rawReturnTo.startsWith('//')) {
        return '/profile'
      }

      if (url.origin === window.location.origin) {
        return `${url.pathname}${url.search}`
      }
    } catch {
      // ignore invalid URL
    }

    return '/profile'
  })()

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError('')
    setLoading(true)

    try {
      await authApi.login(username, password)
      const session = await authApi.getSession() as { user: { id: string; username: string; email: string; display_name: string; picture?: string; is_admin: boolean } }
      setUser(session.user)
      navigate(returnTo)
    } catch (err: unknown) {
      setError(getErrorMessage(err, 'login'))
    } finally {
      setLoading(false)
    }
  }

  const content = (
    <>
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-2.5">
          <div className="flex items-center justify-center w-8 h-8 rounded-md bg-black dark:bg-white">
            <Shield className="w-4 h-4 text-white dark:text-black" />
          </div>
          <span className="text-sm font-bold text-gray-900 dark:text-white">
            {brandName}
          </span>
        </div>
        <ThemeToggle />
      </div>

      {/* Title */}
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-1">
          欢迎回来
        </h1>
        <p className="text-sm text-gray-500 dark:text-gray-500">
          请输入您的账户信息以继续
        </p>
      </div>

      {/* Error */}
      {error && (
        <div className="mb-5 p-3 rounded-lg bg-red-500/[0.08] border border-red-500/[0.16]">
          <p className="text-sm text-red-400">{error}</p>
        </div>
      )}

      {/* Form */}
      <form onSubmit={handleSubmit} className="space-y-4">
        <Input
          label="用户名"
          type="text"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          placeholder="请输入用户名"
          required
          autoFocus
        />

        <div className="relative">
          <Input
            label="密码"
            type={showPassword ? 'text' : 'password'}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="请输入密码"
            required
          />
          <button
            type="button"
            onClick={() => setShowPassword(!showPassword)}
            className="absolute right-3 top-[34px] text-gray-500 hover:text-gray-300 transition-colors"
          >
            {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
          </button>
        </div>

        <div className="flex items-center justify-between text-sm">
          <label className="flex items-center gap-2 text-gray-600 dark:text-gray-500 cursor-pointer">
            <input
              type="checkbox"
              className="w-4 h-4 rounded border-gray-300 dark:border-white/[0.12] bg-white dark:bg-white/[0.04] text-black dark:text-white focus:ring-black/20 dark:focus:ring-white/20"
            />
            <span className="text-xs">记住我</span>
          </label>
          <Link
            to="/forgot-password"
            className="text-xs text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white transition-colors"
          >
            忘记密码？
          </Link>
        </div>

        <button
          type="submit"
          disabled={loading}
          style={{ backgroundColor: '#ffffff', color: '#000000' }}
          className="w-full py-3 px-4 font-bold text-sm rounded-md hover:bg-gray-100 btn-transition disabled:opacity-50 disabled:cursor-not-allowed border border-white"
        >
          {loading ? (
            <span className="flex items-center justify-center gap-2">
              <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
              登录中...
            </span>
          ) : (
            '登录'
          )}
        </button>
      </form>

      {/* Footer */}
      <div className="mt-6 pt-6 border-t border-gray-200 dark:border-white/[0.06] text-center">
        <p className="text-sm text-gray-500">
          还没有账户？
          <Link
            to="/register"
            className="ml-1 text-gray-900 hover:underline dark:text-white transition-colors"
          >
            立即注册
          </Link>
        </p>
      </div>
    </>
  )

  return (
    <LoginPageWrapper>
      {content}
    </LoginPageWrapper>
  )
}
