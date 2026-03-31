import { useState } from 'react'
import { useNavigate, useSearchParams, Link } from 'react-router-dom'
import { Eye, EyeOff, ArrowRight, Shield } from 'lucide-react'
import { useSessionStore } from '#src/stores/session'
import { useUIConfigStore } from '#src/stores/theme'
import { authApi } from '#src/api/auth'
import { LoginPageWrapper } from '#src/components/LoginLayout'
import { ThemeToggle } from '#src/components/ThemeToggle'

export function LoginPage() {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const { brandName, loginLayout } = useUIConfigStore()
  const { setUser } = useSessionStore()

  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const returnTo = searchParams.get('return_to') || '/profile'
  const isFullscreen = loginLayout === 'fullscreen'

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError('')
    setLoading(true)

    try {
      const result = await authApi.login(username, password) as { user: unknown }
      setUser(result.user as { id: string; username: string; email: string; display_name: string; picture?: string; is_admin: boolean })
      navigate(returnTo)
    } catch {
      setError('用户名或密码错误')
    } finally {
      setLoading(false)
    }
  }

  const content = (
    <>
      {/* Header */}
      <div className="flex items-center justify-between mb-10">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-primary-600 text-white">
            <Shield className="w-5 h-5" />
          </div>
          <span className={`text-lg font-semibold tracking-tight ${isFullscreen ? 'text-slate-900 dark:text-white' : 'text-slate-900 dark:text-white'}`}>
            {brandName}
          </span>
        </div>
        <ThemeToggle />
      </div>

      {/* Title */}
      <div className="mb-8">
        <h1 className={`text-2xl font-bold tracking-tight mb-2 ${isFullscreen ? 'text-slate-900 dark:text-white' : 'text-slate-900 dark:text-white'}`}>
          欢迎回来
        </h1>
        <p className={isFullscreen ? 'text-slate-500 dark:text-slate-400' : 'text-slate-500 dark:text-slate-400'}>
          请输入您的账户信息以继续
        </p>
      </div>

      {/* Error */}
      {error && (
        <div className="mb-6 p-3 rounded-lg bg-error-50 dark:bg-error-900/10 border border-error-100 dark:border-error-900/20">
          <p className="text-sm text-error-600 dark:text-error-400">{error}</p>
        </div>
      )}

      {/* Form */}
      <form onSubmit={handleSubmit} className="space-y-5">
        <div>
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1.5">
            用户名
          </label>
          <input
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            required
            autoFocus
            className="input-field"
            placeholder="请输入用户名"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1.5">
            密码
          </label>
          <div className="relative">
            <input
              type={showPassword ? 'text' : 'password'}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              className="input-field pr-10"
              placeholder="请输入密码"
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 transition-colors"
            >
              {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
            </button>
          </div>
        </div>

        <div className="flex items-center justify-between text-sm">
          <label className="flex items-center gap-2 text-slate-600 dark:text-slate-400 cursor-pointer">
            <input
              type="checkbox"
              className="w-4 h-4 rounded border-slate-300 text-primary-600 focus:ring-primary-500/20"
            />
            记住我
          </label>
          <Link
            to="/forgot-password"
            className="text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300 font-medium transition-colors"
          >
            忘记密码？
          </Link>
        </div>

        <button
          type="submit"
          disabled={loading}
          className="btn-primary w-full group"
        >
          {loading ? (
            <span className="flex items-center gap-2">
              <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
              登录中...
            </span>
          ) : (
            <span className="flex items-center gap-2">
              登录
              <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-0.5" />
            </span>
          )}
        </button>
      </form>

      {/* Footer */}
      <div className="mt-8 pt-6 border-t border-slate-200 dark:border-slate-700/50 text-center">
        <p className="text-sm text-slate-500 dark:text-slate-400">
          还没有账户？
          <Link
            to="/register"
            className="ml-1 text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300 font-medium transition-colors"
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
