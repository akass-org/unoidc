import { useState } from 'react'
import { useNavigate, Link } from 'react-router-dom'
import { ArrowRight, Shield, User, Mail, Lock, Eye, EyeOff } from 'lucide-react'
import { getErrorMessage } from '#src/api/client'
import { authApi } from '#src/api/auth'
import { LoginPageWrapper } from '#src/components/LoginLayout'
import { ThemeToggle } from '#src/components/ThemeToggle'
import { useUIConfigStore } from '#src/stores/theme'
import { useSessionStore } from '#src/stores/session'

export function RegisterPage() {
  const navigate = useNavigate()
  const { brandName } = useUIConfigStore()
  const { setUser } = useSessionStore()

  const [formData, setFormData] = useState({
    username: '',
    email: '',
    displayName: '',
    password: '',
    confirmPassword: '',
  })
  const [showPassword, setShowPassword] = useState(false)
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError('')

    if (formData.password !== formData.confirmPassword) {
      setError('两次输入的密码不一致')
      return
    }

    setLoading(true)
    try {
      // 注册
      await authApi.register({
        username: formData.username,
        email: formData.email,
        display_name: formData.displayName,
        password: formData.password,
      })

      // 自动登录
      const result = await authApi.login(formData.username, formData.password) as { user: unknown }
      setUser(result.user as { id: string; username: string; email: string; display_name: string; picture?: string; is_admin: boolean })

      // 跳转到个人资料页
      navigate('/profile')
    } catch (err: unknown) {
      setError(getErrorMessage(err))
    } finally {
      setLoading(false)
    }
  }

  return (
    <LoginPageWrapper>
      {/* Header */}
      <div className="flex items-center justify-between mb-10">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-primary-600 text-white">
            <Shield className="w-5 h-5" />
          </div>
          <span className="text-lg font-semibold tracking-tight text-slate-900 dark:text-white">
            {brandName}
          </span>
        </div>
        <ThemeToggle />
      </div>

      {/* Title */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight mb-2 text-slate-900 dark:text-white">
          创建账户
        </h1>
        <p className="text-slate-500 dark:text-slate-400">
          填写以下信息完成账户注册
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
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1.5">
              用户名
            </label>
            <div className="relative">
              <User className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
              <input
                type="text"
                value={formData.username}
                onChange={(e) => setFormData({ ...formData, username: e.target.value })}
                required
                autoFocus
                className="input-field pl-9"
                placeholder="用户名"
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1.5">
              显示名称
            </label>
            <div className="relative">
              <User className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
              <input
                type="text"
                value={formData.displayName}
                onChange={(e) => setFormData({ ...formData, displayName: e.target.value })}
                required
                className="input-field pl-9"
                placeholder="显示名称"
              />
            </div>
          </div>
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1.5">
            邮箱
          </label>
          <div className="relative">
            <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
            <input
              type="email"
              value={formData.email}
              onChange={(e) => setFormData({ ...formData, email: e.target.value })}
              required
              className="input-field pl-9"
              placeholder="your@email.com"
            />
          </div>
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1.5">
            密码
          </label>
          <div className="relative">
            <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
            <input
              type={showPassword ? 'text' : 'password'}
              value={formData.password}
              onChange={(e) => setFormData({ ...formData, password: e.target.value })}
              required
              minLength={8}
              className="input-field pl-9 pr-10"
              placeholder="至少8位字符"
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

        <div>
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1.5">
            确认密码
          </label>
          <div className="relative">
            <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
            <input
              type={showPassword ? 'text' : 'password'}
              value={formData.confirmPassword}
              onChange={(e) => setFormData({ ...formData, confirmPassword: e.target.value })}
              required
              className="input-field pl-9"
              placeholder="再次输入密码"
            />
          </div>
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
              注册中...
            </span>
          ) : (
            <span className="flex items-center gap-2">
              创建账户
              <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-0.5" />
            </span>
          )}
        </button>
      </form>

      {/* Footer */}
      <div className="mt-8 pt-6 border-t border-slate-200 dark:border-slate-700/50 text-center">
        <p className="text-sm text-slate-500 dark:text-slate-400">
          已有账户？
          <Link
            to="/login"
            className="ml-1 text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300 font-medium transition-colors"
          >
            立即登录
          </Link>
        </p>
      </div>
    </LoginPageWrapper>
  )
}
