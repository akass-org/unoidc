import { useState, useEffect } from 'react'
import { useSearchParams, useNavigate } from 'react-router-dom'
import { 
  Shield, 
  Check, 
  X,
  User,
  Mail,
  Users,
  Key
} from 'lucide-react'
import { useSessionStore } from '#src/stores/session'
import { ThemeToggle } from '#src/components/ThemeToggle'
import { Button, Card, Avatar, useToast, Badge } from '#src/components/ui'
import { api, getErrorMessage } from '#src/api/client'

interface ConsentRequest {
  client_id: string
  client_name: string
  client_description?: string
  client_logo?: string
  scopes: string[]
  redirect_uri: string
}

interface AuthorizePreviewResponse {
  client_id: string
  client_name: string
  redirect_uri: string
  scope: string
  state: string
  nonce?: string
  scopes: string[]
  requires_login: boolean
  requires_consent: boolean
}

interface ConsentResponse {
  code?: string
  redirect_uri?: string
}

function parseSafeRedirectUri(raw: string | null): URL | null {
  if (!raw) return null

  try {
    const url = new URL(raw)
    if (!['https:', 'http:'].includes(url.protocol)) {
      return null
    }
    // 拒绝带凭据的 URL，避免混淆与日志泄露风险
    if (url.username || url.password) {
      return null
    }
    return url
  } catch {
    return null
  }
}

const scopeConfig: Record<string, { label: string; description: string; icon: typeof User }> = {
  openid: { 
    label: 'OpenID', 
    description: '获取您的唯一标识符',
    icon: Key
  },
  profile: { 
    label: '基本资料', 
    description: '获取您的姓名、头像等基本资料',
    icon: User
  },
  email: { 
    label: '邮箱地址', 
    description: '获取您的邮箱地址',
    icon: Mail
  },
  groups: { 
    label: '群组信息', 
    description: '获取您所属的用户组',
    icon: Users
  },
  offline_access: { 
    label: '离线访问', 
    description: '在您不在线时继续访问您的账户',
    icon: Key
  },
}

export function AuthorizePage() {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const { user, loading: sessionLoading } = useSessionStore()
  const { addToast } = useToast()

  const [consentRequest, setConsentRequest] = useState<ConsentRequest | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [processing, setProcessing] = useState(false)
  const [selectedScopes, setSelectedScopes] = useState<string[]>([])

  const clientId = searchParams.get('client_id')
  const redirectUri = searchParams.get('redirect_uri')
  const authState = searchParams.get('state')
  const codeChallenge = searchParams.get('code_challenge')
  const codeChallengeMethod = searchParams.get('code_challenge_method') || 'S256'
  const nonce = searchParams.get('nonce')
  const scopeParam = searchParams.get('scope')

  const submitApprovedAuthorization = async (
    clientId: string,
    redirectUri: string,
    scopes: string[],
  ) => {
    const data = await api.post('authorize/consent', {
      json: {
        client_id: clientId,
        redirect_uri: redirectUri,
        state: authState,
        code_challenge: codeChallenge,
        code_challenge_method: codeChallengeMethod,
        nonce,
        scopes,
        approved: true,
      },
    }).json<ConsentResponse>()

    const approvedRedirect = parseSafeRedirectUri(data.redirect_uri || redirectUri)
    if (!approvedRedirect) {
      throw new Error('无效的回调地址')
    }

    if (!data.code) {
      throw new Error('授权响应缺少 code')
    }

    const params = new URLSearchParams()
    params.set('code', data.code)
    if (authState) params.set('state', authState)
    approvedRedirect.search = params.toString()
    window.location.href = approvedRedirect.toString()
  }

  // Check login status and load client info
  useEffect(() => {
    if (sessionLoading) return

    if (!user) {
      const returnUrl = encodeURIComponent(window.location.href)
      navigate(`/login?return_to=${returnUrl}`)
      return
    }

    // Validate required parameters
    if (!clientId || !redirectUri || !codeChallenge || !authState) {
      setError('授权请求参数不完整')
      setLoading(false)
      return
    }

    // Load client information from backend
    void loadClientInfo()
  }, [user, sessionLoading, clientId, redirectUri, codeChallenge, authState, navigate])

  const loadClientInfo = async () => {
    try {
      const requestedScope = scopeParam || 'openid profile'
      const preview = await api.get('authorize', {
        searchParams: {
          response_type: 'code',
          client_id: clientId!,
          redirect_uri: redirectUri!,
          scope: requestedScope,
          state: authState!,
          code_challenge: codeChallenge!,
          code_challenge_method: codeChallengeMethod,
          ...(nonce ? { nonce } : {}),
        },
      }).json<AuthorizePreviewResponse>()

      const scopes = preview.scopes.length > 0
        ? preview.scopes
        : preview.scope.split(' ').filter(Boolean)

      setConsentRequest({
        client_id: preview.client_id,
        client_name: preview.client_name,
        client_description: '正在请求访问您的账户信息',
        scopes,
        redirect_uri: preview.redirect_uri,
      })
      setSelectedScopes(scopes)
    } catch (err) {
      setError('无法获取应用信息')
    } finally {
      setLoading(false)
    }
  }

  const handleApprove = async () => {
    if (!consentRequest || !codeChallenge || !authState) {
      setError('授权请求参数不完整')
      return
    }

    setProcessing(true)
    try {
      await submitApprovedAuthorization(consentRequest.client_id, consentRequest.redirect_uri, selectedScopes)
    } catch (err) {
      addToast({
        type: 'error',
        title: '授权失败',
        message: getErrorMessage(err),
      })
      setProcessing(false)
    }
  }

  const handleDeny = () => {
    const deniedRedirect = parseSafeRedirectUri(redirectUri)
    if (!deniedRedirect) {
      setError('无效的回调地址')
      return
    }

    const params = new URLSearchParams()
    params.set('error', 'access_denied')
    params.set('error_description', '用户拒绝了授权请求')
    if (authState) params.set('state', authState)
    deniedRedirect.search = params.toString()
    window.location.href = deniedRedirect.toString()
  }

  const toggleScope = (scope: string) => {
    if (scope === 'openid') return
    
    setSelectedScopes(prev => 
      prev.includes(scope) 
        ? prev.filter(s => s !== scope)
        : [...prev, scope]
    )
  }

  if (loading || sessionLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-black page-content">
        <div className="text-center">
          <div className="w-6 h-6 border border-gray-300 dark:border-white/20 border-t-gray-900 dark:border-t-white rounded-full spinner-smooth mx-auto mb-3" />
          <p className="text-sm text-gray-500 dark:text-gray-600">加载中...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-black p-4 page-content">
        <Card className="max-w-sm w-full text-center">
          <div className="w-12 h-12 mx-auto mb-4 rounded-full bg-red-500/[0.08] flex items-center justify-center border border-red-500/[0.16]">
            <X className="w-6 h-6 text-red-400" />
          </div>
          <h1 className="text-lg font-bold text-gray-900 dark:text-white mb-1">授权请求错误</h1>
          <p className="text-sm text-gray-500 mb-5">{error}</p>
          <Button onClick={() => navigate('/login')} size="sm">
            返回登录
          </Button>
        </Card>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-black p-4 page-content">
      <div className="max-w-md mx-auto">
        {/* Header */}
        <div className="flex justify-end py-3">
          <ThemeToggle />
        </div>

        {/* Consent Card */}
        <Card className="overflow-hidden">
          {/* App Info */}
          <div className="p-6 text-center border-b border-gray-200 dark:border-white/[0.06]">
            <div className="w-16 h-16 mx-auto mb-4 rounded-xl bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] flex items-center justify-center">
              {consentRequest?.client_logo ? (
                <img 
                  src={consentRequest.client_logo} 
                  alt="" 
                  className="w-9 h-9 object-contain"
                />
              ) : (
                <Shield className="w-8 h-8 text-gray-400" />
              )}
            </div>
            <h1 className="text-lg font-bold text-gray-900 dark:text-white mb-0.5">
              {consentRequest?.client_name}
            </h1>
            <p className="text-sm text-gray-500">
              {consentRequest?.client_description}
            </p>
            <code className="block mt-2 text-[11px] text-gray-400 dark:text-gray-700 font-mono">
              {consentRequest?.client_id}
            </code>
          </div>

          {/* User Info */}
          <div className="px-5 py-3 bg-gray-50 dark:bg-white/[0.02] border-b border-gray-200 dark:border-white/[0.06]">
            <div className="flex items-center gap-3">
              <Avatar 
                name={user?.display_name || user?.username || '?'} 
                src={user?.picture}
                size="md" 
              />
              <div className="flex-1 min-w-0">
                <p className="text-sm text-gray-900 dark:text-white truncate">
                  {user?.display_name || user?.username}
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-600 truncate">
                  {user?.email}
                </p>
              </div>
              <button
                onClick={() => navigate('/login')}
                className="text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white transition-colors"
              >
                切换
              </button>
            </div>
          </div>

          {/* Scopes */}
          <div className="p-5">
            <h2 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-3">
              该应用请求访问以下信息：
            </h2>
            <div className="space-y-2">
              {consentRequest?.scopes.map((scope) => {
                const config = scopeConfig[scope] || { 
                  label: scope, 
                  description: `访问 ${scope}`,
                  icon: Shield
                }
                const Icon = config.icon
                const isRequired = scope === 'openid'
                const isSelected = selectedScopes.includes(scope)

                return (
                  <div
                    key={scope}
                    onClick={() => !isRequired && toggleScope(scope)}
                    className={`
                      flex items-start gap-3 p-3 rounded-lg border transition-all
                      ${isRequired 
                        ? 'bg-gray-50 dark:bg-white/[0.02] border-gray-200 dark:border-white/[0.06]' 
                        : isSelected
                          ? 'bg-gray-100 dark:bg-white/[0.04] border-gray-300 dark:border-white/[0.12] cursor-pointer'
                          : 'bg-transparent border-gray-200 dark:border-white/[0.04] cursor-pointer hover:border-gray-300 dark:hover:border-white/[0.08]'
                      }
                    `}
                  >
                    <div className={`
                      w-4 h-4 rounded flex items-center justify-center flex-shrink-0 mt-0.5 border
                      ${isRequired 
                        ? 'bg-gray-200 dark:bg-white/[0.06] border-gray-300 dark:border-white/[0.12] text-gray-500' 
                        : isSelected
                          ? 'bg-black text-white dark:bg-white dark:text-black border-black dark:border-white'
                          : 'border-gray-300 dark:border-white/[0.12]'
                      }
                    `}>
                      {(isRequired || isSelected) && <Check className="w-3 h-3" />}
                    </div>
                    <Icon className={`w-4 h-4 flex-shrink-0 mt-0.5 ${isSelected ? 'text-gray-600 dark:text-gray-400' : 'text-gray-400 dark:text-gray-600'}`} />
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <p className={`text-sm ${isSelected ? 'text-gray-900 dark:text-white' : 'text-gray-600 dark:text-gray-400'}`}>
                          {config.label}
                        </p>
                        {isRequired && (
                          <Badge size="sm">必需</Badge>
                        )}
                      </div>
                      <p className="text-xs text-gray-500 dark:text-gray-600 mt-0.5">
                        {config.description}
                      </p>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>

          {/* Actions */}
          <div className="px-5 pb-4 space-y-2">
            <Button
              onClick={handleApprove}
              loading={processing}
              className="w-full"
            >
              同意授权
            </Button>
            <Button
              variant="ghost"
              onClick={handleDeny}
              disabled={processing}
              className="w-full"
            >
              拒绝
            </Button>
          </div>

          {/* Footer */}
          <div className="px-5 pb-5 text-center">
            <p className="text-[11px] text-gray-400 dark:text-gray-700">
              授权后，您可以在"我的应用"中随时撤销此应用的访问权限
            </p>
          </div>
        </Card>

        {/* Security Note */}
        <div className="mt-5 text-center">
          <div className="inline-flex items-center gap-1.5 text-xs text-gray-500 dark:text-gray-700">
            <Shield className="w-3.5 h-3.5" />
            <span>此应用已通过安全验证</span>
          </div>
        </div>
      </div>
    </div>
  )
}
