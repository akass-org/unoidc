import { useState } from 'react'

interface App {
  id: string
  name: string
  description: string
  grantedAt: string
  scopes: string[]
}

const mockApps: App[] = [
  {
    id: 'app-1',
    name: '示例应用',
    description: '这是一个示例应用',
    grantedAt: '2025-03-15',
    scopes: ['openid', 'profile', 'email'],
  },
]

const scopeLabels: Record<string, string> = {
  openid: '唯一标识',
  profile: '基本资料',
  email: '邮箱地址',
  groups: '群组信息',
  offline_access: '长期访问',
}

export function MyAppsPage() {
  const [apps, setApps] = useState<App[]>(mockApps)
  const [revoking, setRevoking] = useState<string | null>(null)

  async function handleRevoke(appId: string) {
    setRevoking(appId)
    try {
      // TODO: 调用撤销接口
      await new Promise((resolve) => setTimeout(resolve, 500))
      setApps(apps.filter((a) => a.id !== appId))
    } finally {
      setRevoking(null)
    }
  }

  return (
    <div>
      <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">我的应用</h1>

      {apps.length === 0 ? (
        <div className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-12 text-center">
          <div className="text-6xl mb-4">📱</div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
            暂无授权的应用
          </h3>
          <p className="text-gray-500 dark:text-gray-400 text-sm mb-6">
            您还没有授权任何第三方应用访问您的账户
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {apps.map((app) => (
            <div
              key={app.id}
              className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-6"
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-4">
                  <div className="w-14 h-14 rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-2xl">
                    📱
                  </div>
                  <div>
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                      {app.name}
                    </h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                      {app.description}
                    </p>
                    <p className="text-xs text-gray-400 dark:text-gray-500 mt-2">
                      授权时间: {app.grantedAt}
                    </p>
                  </div>
                </div>
                <button
                  onClick={() => handleRevoke(app.id)}
                  disabled={revoking === app.id}
                  className="px-4 py-2 text-sm font-medium text-red-600 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 border border-red-200 dark:border-red-800 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors disabled:opacity-50"
                >
                  {revoking === app.id ? '撤销中...' : '撤销访问'}
                </button>
              </div>

              {/* Scopes */}
              <div className="mt-4 pt-4 border-t border-gray-100 dark:border-gray-800">
                <p className="text-xs text-gray-500 dark:text-gray-400 mb-2">已授权权限:</p>
                <div className="flex flex-wrap gap-2">
                  {app.scopes.map((scope) => (
                    <span
                      key={scope}
                      className="px-2 py-1 text-xs rounded-full bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300"
                    >
                      {scopeLabels[scope] || scope}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Info Card */}
      <div className="mt-8 p-4 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-100 dark:border-blue-800">
        <h4 className="text-sm font-medium text-blue-900 dark:text-blue-300 mb-2">
          关于应用授权
        </h4>
        <ul className="text-xs text-blue-700 dark:text-blue-400 space-y-1 list-disc list-inside">
          <li>您可以随时撤销对任何应用的授权</li>
          <li>撤销授权后，该应用将无法再访问您的账户信息</li>
          <li>相关刷新令牌将被立即作废</li>
        </ul>
      </div>
    </div>
  )
}