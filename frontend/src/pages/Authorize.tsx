import { useState, useEffect } from 'react'
import { useSearchParams, useNavigate } from 'react-router-dom'
import { useSessionStore } from '#src/stores/session'
import { ThemeToggle } from '#src/components/ThemeToggle'

interface ConsentRequest {
  client_id: string
  client_name: string
  scopes: string[]
  redirect_uri: string
}

const scopeLabels: Record<string, string> = {
  openid: '获取您的唯一标识',
  profile: '获取您的基本资料（姓名、头像）',
  email: '获取您的邮箱地址',
  groups: '获取您所属的群组',
  offline_access: '长期访问您的账户（刷新令牌）',
}

const scopeIcons: Record<string, string> = {
  openid: '🆔',
  profile: '👤',
  email: '📧',
  groups: '👥',
  offline_access: '🔑',
}

export function AuthorizePage() {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const { user, loading: sessionLoading } = useSessionStore()

  const [consentRequest, setConsentRequest] = useState<ConsentRequest | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [processing, setProcessing] = useState(false)

  const clientId = searchParams.get('client_id')
  const redirectUri = searchParams.get('redirect_uri')
  const state = searchParams.get('state')
  const codeChallenge = searchParams.get('code_challenge')

  useEffect(() => {
    // 检查登录状态
    if (!sessionLoading && !user) {
      const returnUrl = encodeURIComponent(window.location.href)
      navigate(`/login?return_to=${returnUrl}`)
      return
    }

    // 验证授权请求参数
    if (!clientId || !redirectUri || !codeChallenge) {
      setError('授权请求参数不完整')
      setLoading(false)
      return
    }

    // 模拟获取客户端信息
    setConsentRequest({
      client_id: clientId,
      client_name: '示例应用',
      scopes: ['openid', 'profile', 'email'],
      redirect_uri: redirectUri,
    })
    setLoading(false)
  }, [user, sessionLoading, clientId, redirectUri, codeChallenge, navigate])

  async function handleApprove() {
    setProcessing(true)
    try {
      // TODO: 调用后端授权接口
      const params = new URLSearchParams()
      params.set('code', 'mock_auth_code')
      if (state) params.set('state', state)
      window.location.href = `${redirectUri}?${params.toString()}`
    } catch {
      setError('授权失败，请重试')
      setProcessing(false)
    }
  }

  function handleDeny() {
    const params = new URLSearchParams()
    params.set('error', 'access_denied')
    params.set('error_description', '用户拒绝了授权请求')
    if (state) params.set('state', state)
    window.location.href = `${redirectUri}?${params.toString()}`
  }

  if (loading || sessionLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-950">
        <div className="text-center">
          <div className="w-8 h-8 border-2 border-blue-600 border-t-transparent rounded-full animate-spin mx-auto mb-4" />
          <p className="text-gray-600 dark:text-gray-400">加载中...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-950 p-4">
        <div className="bg-white dark:bg-gray-900 rounded-2xl shadow-xl p-8 max-w-md w-full text-center">
          <div className="text-4xl mb-4">⚠️</div>
          <h1 className="text-xl font-bold text-gray-900 dark:text-white mb-2">授权请求错误</h1>
          <p className="text-gray-600 dark:text-gray-400 mb-6">{error}</p>
          <button
            onClick={() => navigate('/login')}
            className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            返回登录
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 p-4">
      <div className="max-w-lg mx-auto">
        {/* Header */}
        <div className="flex justify-end py-4">
          <ThemeToggle />
        </div>

        {/* Consent Card */}
        <div className="bg-white dark:bg-gray-900 rounded-2xl shadow-xl overflow-hidden">
          {/* App Info */}
          <div className="p-8 text-center border-b border-gray-100 dark:border-gray-800">
            <div className="w-20 h-20 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-3xl">
              📱
            </div>
            <h1 className="text-xl font-bold text-gray-900 dark:text-white mb-1">
              {consentRequest?.client_name}
            </h1>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              {consentRequest?.client_id}
            </p>
          </div>

          {/* User Info */}
          <div className="px-8 py-4 bg-gray-50 dark:bg-gray-800/50">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-gradient-to-br from-green-400 to-blue-500 flex items-center justify-center text-white font-medium">
                {user?.display_name?.charAt(0) || user?.username?.charAt(0) || '?'}
              </div>
              <div className="flex-1">
                <p className="text-sm font-medium text-gray-900 dark:text-white">
                  {user?.display_name || user?.username}
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  {user?.email}
                </p>
              </div>
              <button
                onClick={() => navigate('/login')}
                className="text-xs text-blue-600 hover:text-blue-700 dark:text-blue-400"
              >
                切换账户
              </button>
            </div>
          </div>

          {/* Scopes */}
          <div className="p-8">
            <h2 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-4">
              该应用请求访问以下信息：
            </h2>
            <div className="space-y-3">
              {consentRequest?.scopes.map((scope) => (
                <div
                  key={scope}
                  className="flex items-start gap-3 p-3 rounded-lg bg-gray-50 dark:bg-gray-800/50"
                >
                  <span className="text-xl">{scopeIcons[scope] || '🔹'}</span>
                  <div>
                    <p className="text-sm font-medium text-gray-900 dark:text-white">
                      {scope}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      {scopeLabels[scope] || '访问您的信息'}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Actions */}
          <div className="p-8 pt-0 space-y-3">
            <button
              onClick={handleApprove}
              disabled={processing}
              className="
                w-full py-3 px-4 rounded-lg
                bg-gradient-to-r from-blue-600 to-purple-600
                hover:from-blue-700 hover:to-purple-700
                text-white font-medium
                transition-all duration-200
                disabled:opacity-50 disabled:cursor-not-allowed
                focus:outline-none focus:ring-2 focus:ring-blue-500
              "
            >
              {processing ? '处理中...' : '同意授权'}
            </button>
            <button
              onClick={handleDeny}
              disabled={processing}
              className="
                w-full py-3 px-4 rounded-lg
                border border-gray-200 dark:border-gray-700
                text-gray-700 dark:text-gray-300
                hover:bg-gray-50 dark:hover:bg-gray-800
                font-medium
                transition-all duration-200
                disabled:opacity-50 disabled:cursor-not-allowed
                focus:outline-none focus:ring-2 focus:ring-gray-300
              "
            >
              拒绝
            </button>
          </div>

          {/* Footer */}
          <div className="px-8 pb-6 text-center">
            <p className="text-xs text-gray-500 dark:text-gray-400">
              授权后，您可以在"我的应用"中随时撤销此应用的访问权限
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
