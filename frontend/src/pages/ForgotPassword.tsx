import { useState } from 'react'
import { Link } from 'react-router-dom'
import { ArrowRight, Shield, Mail, CheckCircle, ArrowLeft } from 'lucide-react'
import { authApi } from '#src/api/auth'
import { LoginPageWrapper } from '#src/components/LoginLayout'
import { ThemeToggle } from '#src/components/ThemeToggle'
import { useUIConfigStore } from '#src/stores/theme'

export function ForgotPasswordPage() {
  const { brandName } = useUIConfigStore()

  const [email, setEmail] = useState('')
  const [submitted, setSubmitted] = useState(false)
  const [loading, setLoading] = useState(false)

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setLoading(true)
    try {
      await authApi.forgotPassword(email)
      setSubmitted(true)
    } catch {
      // 即使失败也显示成功，避免泄露邮箱是否存在
      setSubmitted(true)
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

      {/* Content */}
      {submitted ? (
        <div className="text-center py-4">
          <div className="inline-flex items-center justify-center w-16 h-16 mb-6 rounded-full bg-success-50 dark:bg-success-900/20 text-success-600 dark:text-success-400">
            <CheckCircle className="w-8 h-8" />
          </div>
          <h2 className="text-2xl font-bold tracking-tight mb-3 text-slate-900 dark:text-white">
            邮件已发送
          </h2>
          <p className="text-slate-500 dark:text-slate-400 mb-2">
            如果该邮箱已注册，重置密码链接已发送至您的邮箱。
          </p>
          <p className="text-sm text-slate-400 dark:text-slate-500 mb-8">
            请检查您的收件箱，链接有效期为30分钟。
          </p>
          <Link
            to="/login"
            className="inline-flex items-center gap-2 text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300 font-medium transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            返回登录
          </Link>
        </div>
      ) : (
        <>
          {/* Title */}
          <div className="mb-8">
            <h1 className="text-2xl font-bold tracking-tight mb-2 text-slate-900 dark:text-white">
              找回密码
            </h1>
            <p className="text-slate-500 dark:text-slate-400">
              输入您的邮箱地址，我们将发送重置链接
            </p>
          </div>

          {/* Form */}
          <form onSubmit={handleSubmit} className="space-y-5">
            <div>
              <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1.5">
                邮箱地址
              </label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                  autoFocus
                  className="input-field pl-9"
                  placeholder="your@email.com"
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
                  发送中...
                </span>
              ) : (
                <span className="flex items-center gap-2">
                  发送重置链接
                  <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-0.5" />
                </span>
              )}
            </button>
          </form>

          {/* Footer */}
          <div className="mt-8 pt-6 border-t border-slate-200 dark:border-slate-700/50 text-center">
            <Link
              to="/login"
              className="inline-flex items-center gap-2 text-sm text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300 transition-colors"
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