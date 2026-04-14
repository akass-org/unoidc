import { useEffect, useState } from 'react'
import { AppWindow, Trash2, Clock, Shield } from 'lucide-react'
import { meApi } from '#src/api/me'
import { 
  Card, 
  Button, 
  EmptyState,
  ConfirmModal,
  useToast
} from '#src/components/ui'
import { getErrorMessage } from '#src/api/client'

interface App {
  client_id: string
  client_name: string
  description?: string
  granted_at?: string | null
  scopes: string[]
  access_source: 'consent' | 'group'
}

const scopeLabels: Record<string, string> = {
  openid: '唯一标识',
  profile: '基本资料',
  email: '邮箱地址',
  groups: '群组信息',
  offline_access: '长期访问',
}

const scopeDescriptions: Record<string, string> = {
  openid: '访问您的 OpenID 标识',
  profile: '访问您的姓名、头像等基本资料',
  email: '访问您的邮箱地址',
  groups: '访问您所属的用户组',
  offline_access: '在离线时继续访问您的账户',
}

export function MyAppsPage() {
  const [apps, setApps] = useState<App[]>([])
  const [loading, setLoading] = useState(true)
  const [revokingId, setRevokingId] = useState<string | null>(null)
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null)
  const { addToast } = useToast()

  // Fetch authorized apps
  useEffect(() => {
    loadApps()
  }, [])

  const loadApps = async () => {
    try {
      setLoading(true)
      const data = (await meApi.getApps()) as App[]
      setApps(data)
    } catch (err) {
      addToast({
        type: 'error',
        title: '加载失败',
        message: getErrorMessage(err),
      })
    } finally {
      setLoading(false)
    }
  }

  const handleRevoke = async (clientId: string) => {
    try {
      setRevokingId(clientId)
      await meApi.revokeConsent(clientId)
      await loadApps()
      addToast({
        type: 'success',
        title: '授权已撤销',
      })
    } catch (err) {
      addToast({
        type: 'error',
        title: '撤销失败',
        message: getErrorMessage(err),
      })
    } finally {
      setRevokingId(null)
      setConfirmDelete(null)
    }
  }

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr)
    if (Number.isNaN(date.getTime())) {
      return '未知时间'
    }
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    })
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-48">
        <div className="w-6 h-6 border border-gray-300 dark:border-white/20 border-t-gray-900 dark:border-t-white rounded-full animate-spin" />
      </div>
    )
  }

  return (
    <div className="space-y-5 page-content">
      <div>
        <h1 className="text-xl font-bold text-gray-900 dark:text-white">我的应用</h1>
        <p className="text-sm text-gray-500 mt-0.5">未主动同意过的显示为“可访问”，同意授权过的显示为“已授权”</p>
      </div>

      {apps.length === 0 ? (
        <Card padding="lg">
          <EmptyState
            icon={<AppWindow className="w-6 h-6" />}
            title="暂无可见的应用"
            description="当前没有任何可访问或已授权的应用"
          />
        </Card>
      ) : (
        <div className="space-y-6">
          {apps.map((app, index) => (
            <Card key={app.client_id} hover className="stagger-item" style={{ animationDelay: `${index * 0.05}s` }}>
              <div className="flex flex-col sm:flex-row sm:items-start justify-between gap-4">
                <div className="flex items-start gap-4">
                  <div className="w-11 h-11 rounded-lg bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.06] flex items-center justify-center flex-shrink-0">
                    <AppWindow className="w-5 h-5 text-gray-400" />
                  </div>
                  <div>
                    <h3 className="text-sm font-bold text-gray-900 dark:text-white">
                      {app.client_name}
                    </h3>
                    <p className="text-xs text-gray-500 dark:text-gray-600 font-mono mt-0.5">
                      {app.client_id}
                    </p>
                    {app.description && (
                      <p className="text-sm text-gray-500 mt-1">
                        {app.description}
                      </p>
                    )}
                    <div className="flex flex-wrap items-center gap-2 mt-2 text-xs text-gray-500 dark:text-gray-600">
                      {app.access_source === 'consent' ? (
                        <span className="inline-flex items-center gap-1.5 rounded-full border border-emerald-500/20 bg-emerald-500/10 px-2.5 py-1 text-emerald-500 dark:text-emerald-400">
                          <Clock className="w-3.5 h-3.5" />
                          已授权，上次登录于 {app.granted_at ? formatDate(app.granted_at) : '未知时间'}
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1.5 rounded-full border border-blue-500/20 bg-blue-500/10 px-2.5 py-1 text-blue-500 dark:text-blue-400">
                          <Shield className="w-3.5 h-3.5" />
                          可访问
                        </span>
                      )}
                    </div>
                  </div>
                </div>
                
                {app.access_source === 'consent' && (
                  <div className="flex items-center gap-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setConfirmDelete(app.client_id)}
                      loading={revokingId === app.client_id}
                      className="text-red-400 hover:text-red-300 hover:bg-red-500/[0.08]"
                    >
                      <Trash2 className="w-4 h-4" />
                      撤销
                    </Button>
                  </div>
                )}
              </div>

              {/* Scopes */}
              {app.access_source === 'consent' && app.scopes.length > 0 && (
                <div className="mt-5 pt-4 border-t border-gray-200 dark:border-white/[0.04]">
                  <p className="text-xs text-gray-500 dark:text-gray-600 uppercase tracking-wider mb-2">
                    已授权权限
                  </p>
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                    {app.scopes.map((scope) => (
                      <div 
                        key={scope}
                        className="flex items-start gap-2.5 p-2.5 rounded-md bg-gray-50 dark:bg-white/[0.02]"
                      >
                        <Shield className="w-4 h-4 text-gray-500 mt-0.5 flex-shrink-0" />
                        <div>
                          <p className="text-sm text-gray-700 dark:text-gray-300">
                            {scopeLabels[scope] || scope}
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-600 mt-0.5">
                            {scopeDescriptions[scope] || `访问 ${scope}`}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </Card>
          ))}
        </div>
      )}
      {/* Info Card - Only show if there are consented apps */}
      {apps.some((app) => app.access_source === 'consent') && (
        <Card className="bg-blue-50 dark:bg-blue-500/[0.02] border-blue-200 dark:border-blue-500/[0.08] card-hover">
          <div className="flex items-start gap-3">
            <Shield className="w-4 h-4 text-blue-500 dark:text-blue-400 mt-0.5 flex-shrink-0" />
            <div>
              <h4 className="text-sm font-bold text-blue-600 dark:text-blue-400">关于授权和用户组</h4>
              <ul className="text-sm text-blue-600/70 dark:text-blue-400/70 mt-2 space-y-1 list-disc list-inside">
                <li>“可访问”表示你有用户组准入资格，但还未主动同意授权</li>
                <li>“已授权”表示你曾在登录流程中点击过同意授权</li>
                <li>只有“已授权”的应用，才可以在这里撤销</li>
                <li>撤销后，该应用将无法再通过这次授权访问你的账户信息</li>
                <li>相关刷新令牌将被立即作废</li>
                <li>通过用户组获得访问资格的应用，由管理员配置，不能在这里单独撤销</li>
              </ul>
            </div>
          </div>
        </Card>
      )}

      {/* Confirm Delete Modal */}
      <ConfirmModal
        isOpen={!!confirmDelete}
        onClose={() => setConfirmDelete(null)}
        title="撤销应用授权"
        description="确定要撤销此应用的访问权限吗？撤销后该应用将无法再访问您的账户信息。"
        onConfirm={() => confirmDelete && handleRevoke(confirmDelete)}
        confirmText="确认撤销"
        variant="danger"
        loading={revokingId === confirmDelete}
      />
    </div>
  )
}
