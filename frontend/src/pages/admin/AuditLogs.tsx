import { useState } from 'react'

interface AuditLog {
  id: string
  event_type: string
  user_id?: string
  username?: string
  client_id?: string
  ip_address: string
  user_agent: string
  outcome: 'success' | 'failure'
  reason?: string
  created_at: string
}

const mockLogs: AuditLog[] = [
  {
    id: '1',
    event_type: 'login_success',
    user_id: '1',
    username: 'admin',
    ip_address: '192.168.1.1',
    user_agent: 'Mozilla/5.0...',
    outcome: 'success',
    created_at: '2025-03-30 10:30:00',
  },
  {
    id: '2',
    event_type: 'login_failure',
    username: 'user1',
    ip_address: '192.168.1.2',
    user_agent: 'Mozilla/5.0...',
    outcome: 'failure',
    reason: '密码错误',
    created_at: '2025-03-30 09:15:00',
  },
  {
    id: '3',
    event_type: 'token_issued',
    user_id: '2',
    username: 'user1',
    client_id: 'demo-app',
    ip_address: '192.168.1.1',
    user_agent: 'Mozilla/5.0...',
    outcome: 'success',
    created_at: '2025-03-30 10:35:00',
  },
]

const eventTypeLabels: Record<string, string> = {
  login_success: '登录成功',
  login_failure: '登录失败',
  logout: '登出',
  token_issued: '令牌发放',
  token_refresh: '令牌刷新',
  consent_granted: '授权同意',
  consent_revoked: '授权撤销',
  account_locked: '账户锁定',
}

const eventTypeIcons: Record<string, string> = {
  login_success: '✅',
  login_failure: '❌',
  logout: '🚪',
  token_issued: '🔑',
  token_refresh: '🔄',
  consent_granted: '✓',
  consent_revoked: '✗',
  account_locked: '🔒',
}

export function AdminAuditLogs() {
  const [logs] = useState<AuditLog[]>(mockLogs)
  const [search, setSearch] = useState('')
  const [filter, setFilter] = useState<'all' | 'success' | 'failure'>('all')

  const filteredLogs = logs.filter((log) => {
    const matchesSearch =
      log.username?.toLowerCase().includes(search.toLowerCase()) ||
      log.event_type.toLowerCase().includes(search.toLowerCase()) ||
      log.ip_address.includes(search)
    const matchesFilter = filter === 'all' || log.outcome === filter
    return matchesSearch && matchesFilter
  })

  return (
    <div>
      <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">审计日志</h1>

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-4 mb-6">
        <input
          type="text"
          placeholder="搜索日志..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="flex-1 max-w-md px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500"
        />
        <div className="flex gap-2">
          {(['all', 'success', 'failure'] as const).map((f) => (
            <button
              key={f}
              onClick={() => setFilter(f)}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                filter === f
                  ? 'bg-blue-600 text-white'
                  : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700'
              }`}
            >
              {f === 'all' ? '全部' : f === 'success' ? '成功' : '失败'}
            </button>
          ))}
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-4 mb-6">
        <div className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-4">
          <p className="text-sm text-gray-500 dark:text-gray-400">今日事件</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">128</p>
        </div>
        <div className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-4">
          <p className="text-sm text-gray-500 dark:text-gray-400">成功</p>
          <p className="text-2xl font-bold text-green-600 dark:text-green-400">120</p>
        </div>
        <div className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-4">
          <p className="text-sm text-gray-500 dark:text-gray-400">失败</p>
          <p className="text-2xl font-bold text-red-600 dark:text-red-400">8</p>
        </div>
      </div>

      {/* Table */}
      <div className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 overflow-hidden">
        <table className="w-full">
          <thead className="bg-gray-50 dark:bg-gray-800">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">事件</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">用户</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">IP 地址</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">结果</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">时间</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200 dark:divide-gray-800">
            {filteredLogs.map((log) => (
              <tr key={log.id} className="hover:bg-gray-50 dark:hover:bg-gray-800/50">
                <td className="px-6 py-4">
                  <div className="flex items-center gap-2">
                    <span>{eventTypeIcons[log.event_type] || '📝'}</span>
                    <span className="text-sm text-gray-900 dark:text-white">
                      {eventTypeLabels[log.event_type] || log.event_type}
                    </span>
                  </div>
                </td>
                <td className="px-6 py-4 text-sm text-gray-600 dark:text-gray-300">
                  {log.username || '-'}
                </td>
                <td className="px-6 py-4 text-sm text-gray-600 dark:text-gray-300 font-mono">
                  {log.ip_address}
                </td>
                <td className="px-6 py-4">
                  {log.outcome === 'success' ? (
                    <span className="px-2 py-1 text-xs font-medium rounded-full bg-green-100 dark:bg-green-900/20 text-green-700 dark:text-green-300">
                      成功
                    </span>
                  ) : (
                    <span className="px-2 py-1 text-xs font-medium rounded-full bg-red-100 dark:bg-red-900/20 text-red-700 dark:text-red-300">
                      失败
                    </span>
                  )}
                </td>
                <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">
                  {log.created_at}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      <div className="flex items-center justify-between mt-6">
        <p className="text-sm text-gray-500 dark:text-gray-400">
          显示 {filteredLogs.length} 条记录
        </p>
        <div className="flex gap-2">
          <button className="px-4 py-2 text-sm text-gray-600 dark:text-gray-400 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 disabled:opacity-50" disabled>
            上一页
          </button>
          <button className="px-4 py-2 text-sm text-gray-600 dark:text-gray-400 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 disabled:opacity-50" disabled>
            下一页
          </button>
        </div>
      </div>
    </div>
  )
}