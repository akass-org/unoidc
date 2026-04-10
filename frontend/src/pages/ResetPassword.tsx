import { useState } from 'react'
import { useSearchParams, Link } from 'react-router-dom'
import { 
  Shield, 
  Lock, 
  CheckCircle, 
  ArrowLeft,
  Eye,
  EyeOff
} from 'lucide-react'
import { authApi } from '#src/api/auth'
import { LoginPageWrapper } from '#src/components/LoginLayout'
import { ThemeToggle } from '#src/components/ThemeToggle'
import { useUIConfigStore } from '#src/stores/theme'
import { Button, Input } from '#src/components/ui'
import { getErrorMessage } from '#src/api/client'

export function ResetPasswordPage() {
  const { brandName } = useUIConfigStore()
  const [searchParams] = useSearchParams()
  const token = searchParams.get('token')

  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [loading, setLoading] = useState(false)
  const [success, setSuccess] = useState(false)
  const [error, setError] = useState('')

  // Validate token
  if (!token) {
    return (
      <LoginPageWrapper>
        <div className="text-center py-6 page-content">
          <div className="inline-flex items-center justify-center w-12 h-12 mb-4 rounded-full bg-red-500/[0.08] text-red-400 border border-red-500/[0.16]">
            <Lock className="w-6 h-6" />
          </div>
          <h2 className="text-lg font-bold text-gray-900 dark:text-white mb-1">无效的链接</h2>
          <p className="text-sm text-gray-500 mb-5">
            重置密码链接无效或已过期，请重新申请。
          </p>
          <Link to="/forgot-password">
            <Button variant="secondary" size="sm">
              重新申请
            </Button>
          </Link>
        </div>
      </LoginPageWrapper>
    )
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    if (password.length < 8) {
      setError('密码至少需要8个字符')
      return
    }

    if (password !== confirmPassword) {
      setError('两次输入的密码不一致')
      return
    }

    setLoading(true)
    try {
      await authApi.resetPassword(token, password)
      setSuccess(true)
    } catch (err) {
      setError(getErrorMessage(err))
    } finally {
      setLoading(false)
    }
  }

  if (success) {
    return (
      <LoginPageWrapper>
        <div className="text-center py-6 page-content">
          <div className="inline-flex items-center justify-center w-12 h-12 mb-4 rounded-full bg-emerald-500/[0.08] text-emerald-400 border border-emerald-500/[0.16]">
            <CheckCircle className="w-6 h-6" />
          </div>
          <h2 className="text-lg font-bold text-gray-900 dark:text-white mb-1">密码重置成功</h2>
          <p className="text-sm text-gray-500 mb-5">
            您的密码已成功重置，请使用新密码登录。
          </p>
          <Link to="/login">
            <Button size="sm">
              前往登录
            </Button>
          </Link>
        </div>
      </LoginPageWrapper>
    )
  }

  return (
    <LoginPageWrapper>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
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
      <div className="mb-5">
        <h1 className="text-xl font-bold text-gray-900 dark:text-white mb-0.5">
          重置密码
        </h1>
        <p className="text-sm text-gray-500 dark:text-gray-500">
          请输入您的新密码
        </p>
      </div>

      {/* Error */}
      {error && (
        <div className="mb-4 p-2.5 rounded-lg bg-red-500/[0.08] border border-red-500/[0.16]">
          <p className="text-sm text-red-400">{error}</p>
        </div>
      )}

      {/* Form */}
      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="relative">
          <Input
            label="新密码"
            type={showPassword ? 'text' : 'password'}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="至少8位字符"
            required
            minLength={8}
          />
          <button
            type="button"
            onClick={() => setShowPassword(!showPassword)}
            className="absolute right-3 top-[34px] text-gray-500 hover:text-gray-700 dark:text-gray-600 dark:hover:text-gray-400 transition-colors"
          >
            {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
          </button>
        </div>

        <Input
          label="确认新密码"
          type={showPassword ? 'text' : 'password'}
          value={confirmPassword}
          onChange={(e) => setConfirmPassword(e.target.value)}
          placeholder="再次输入新密码"
          required
        />

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
              处理中...
            </span>
          ) : (
            '重置密码'
          )}
        </button>
      </form>

      {/* Footer */}
      <div className="mt-5 pt-5 border-t border-gray-200 dark:border-white/[0.06] text-center">
        <Link
          to="/login"
          className="inline-flex items-center gap-1.5 text-sm text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors"
        >
          <ArrowLeft className="w-4 h-4" />
          返回登录
        </Link>
      </div>
    </LoginPageWrapper>
  )
}
