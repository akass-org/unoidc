import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Shield, CheckCircle, ArrowLeft } from 'lucide-react'
import { authApi } from '#src/api/auth'
import { LoginPageWrapper } from '#src/components/LoginLayout'
import { ThemeToggle } from '#src/components/ThemeToggle'
import { useUIConfigStore } from '#src/stores/theme'
import { Input } from '#src/components/ui'
import { getErrorMessage } from '#src/api/client'

export function ForgotPasswordPage() {
  const { brandName } = useUIConfigStore()

  const [email, setEmail] = useState('')
  const [submitted, setSubmitted] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError('')
    setLoading(true)
    try {
      await authApi.forgotPassword(email)
      setSubmitted(true)
    } catch (err) {
      setError(getErrorMessage(err))
    } finally {
      setLoading(false)
    }
  }

  return (
    <LoginPageWrapper>
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-2.5">
          <div className="flex items-center justify-center w-8 h-8 rounded-md bg-black dark:bg-white">
            <Shield className="w-4 h-4 text-white dark:text-black" />
          </div>
          <span className="text-sm font-medium text-gray-900 dark:text-white">
            {brandName}
          </span>
        </div>
        <ThemeToggle />
      </div>

      {/* Content */}
      {submitted ? (
        <div className="text-center py-4">
          <div className="inline-flex items-center justify-center w-14 h-14 mb-5 rounded-full bg-emerald-500/[0.08] text-emerald-400 border border-emerald-500/[0.16]">
            <CheckCircle className="w-7 h-7" />
          </div>
          <h2 className="text-lg font-medium text-gray-900 dark:text-white mb-2">邮件已发送</h2>
          <p className="text-sm text-gray-500 mb-1">
            如果该邮箱已注册，重置密码链接已发送至您的邮箱。
          </p>
          <p className="text-xs text-gray-500 dark:text-gray-600 mb-6">
            请检查您的收件箱，链接有效期为30分钟。
          </p>
          <Link
            to="/login"
            className="inline-flex items-center gap-1.5 text-sm text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            返回登录
          </Link>
        </div>
      ) : (
        <>
          {/* Title */}
          <div className="mb-6">
            <h1 className="text-xl font-semibold text-gray-900 dark:text-white mb-1">
              找回密码
            </h1>
            <p className="text-sm text-gray-500 dark:text-gray-500">
              输入您的邮箱地址，我们将发送重置链接
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
              label="邮箱地址"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="your@email.com"
              required
              autoFocus
            />

            <button
              type="submit"
              disabled={loading}
              style={{ backgroundColor: '#ffffff', color: '#000000' }}
              className="w-full py-3 px-4 font-medium text-sm rounded-md hover:bg-gray-100 transition-colors disabled:opacity-50 disabled:cursor-not-allowed border border-white"
            >
              {loading ? (
                <span className="flex items-center justify-center gap-2">
                  <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                  </svg>
                  发送中...
                </span>
              ) : (
                '发送重置链接'
              )}
            </button>
          </form>

          {/* Footer */}
          <div className="mt-6 pt-6 border-t border-gray-200 dark:border-white/[0.06] text-center">
            <Link
              to="/login"
              className="inline-flex items-center gap-1.5 text-sm text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors"
            >
              <ArrowLeft className="w-4 h-4" />
              返回登录
            </Link>
          </div>
        </>
      )}
    </LoginPageWrapper>
  )
}
