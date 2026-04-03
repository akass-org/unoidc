import { useEffect, useState } from 'react'
import { ClipboardList, CheckCircle, XCircle, RefreshCw, Key, LogOut, UserPlus, Lock, Search } from 'lucide-react'
import { meApi } from '#src/api/me'
import { getErrorMessage } from '#src/api/client'
import { Badge, Button, Card, EmptyState, Table, useToast } from '#src/components/ui'

interface AuditLog {
  id: string
  event_type: string
  user_id?: string
  username?: string
  client_id?: string
  client_name?: string
  ip_address: string
  user_agent: string
  outcome: 'success' | 'failure'
  reason?: string
  created_at: string
}

const eventTypeConfig: Record<string, { label: string; icon: typeof CheckCircle; color: string }> = {
  login_success: { label: '登录成功', icon: CheckCircle, color: 'text-emerald-400' },
  login_failure: { label: '登录失败', icon: XCircle, color: 'text-red-400' },
  logout: { label: '登出', icon: LogOut, color: 'text-gray-500' },
  token_issued: { label: '令牌发放', icon: Key, color: 'text-blue-400' },
  token_refresh: { label: '令牌刷新', icon: RefreshCw, color: 'text-cyan-400' },
  consent_granted: { label: '授权同意', icon: CheckCircle, color: 'text-emerald-400' },
  consent_revoked: { label: '授权撤销', icon: XCircle, color: 'text-amber-400' },
  user_created: { label: '用户创建', icon: UserPlus, color: 'text-blue-400' },
  password_reset: { label: '密码重置', icon: Lock, color: 'text-orange-400' },
}

const parseDate = (dateStr: string): Date => {
  if (!dateStr || typeof dateStr !== 'string') return new Date(NaN)

  let normalized = dateStr.trim()
  if (normalized.includes(' ') && !normalized.includes('T')) {
    normalized = normalized.replace(/^(\d{4}-\d{2}-\d{2}) /, '$1T')
  }
  normalized = normalized.replace(/(\d{2}:\d{2}:\d{2})\.(\d{6})\d+/, '$1.$2')

  return new Date(normalized)
}

const formatDate = (dateStr: string) => {
  const date = parseDate(dateStr)
  if (Number.isNaN(date.getTime())) return dateStr || '-'

  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: 'numeric',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
}

export function MyAuditLogsPage() {
  const [logs, setLogs] = useState<AuditLog[]>([])
  const [filteredLogs, setFilteredLogs] = useState<AuditLog[]>([])
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState('')
  const [filter, setFilter] = useState<'all' | 'success' | 'failure'>('all')
  const { addToast } = useToast()

  useEffect(() => {
    loadLogs()
  }, [])

  useEffect(() => {
    let filtered = logs

    if (search) {
      const keyword = search.toLowerCase()
      filtered = filtered.filter((log) => {
        return (
          log.event_type.toLowerCase().includes(keyword) ||
          log.client_name?.toLowerCase().includes(keyword) ||
          log.reason?.toLowerCase().includes(keyword) ||
          log.ip_address.includes(search)
        )
      })
    }

    if (filter !== 'all') {
      filtered = filtered.filter((log) => log.outcome === filter)
    }

    setFilteredLogs(filtered)
  }, [logs, search, filter])

  const loadLogs = async () => {
    try {
      setLoading(true)
      const data = await meApi.getAuditLogs() as AuditLog[]
      setLogs(data)
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

  const columns = [
    {
      key: 'event',
      title: '事件',
      render: (log: AuditLog) => {
        const config = eventTypeConfig[log.event_type] || {
          label: log.event_type,
          icon: ClipboardList,
          color: 'text-gray-500',
        }
        const Icon = config.icon
        return (
          <div className="flex items-center gap-2">
            <Icon className={`w-4 h-4 ${config.color}`} />
            <span className="text-sm text-gray-900 dark:text-white">{config.label}</span>
          </div>
        )
      },
    },
    {
      key: 'client',
      title: '应用',
      render: (log: AuditLog) => (
        <div className="space-y-0.5">
          <span className="text-sm text-gray-500">{log.client_name || '-'}</span>
          {log.reason && <p className="text-[11px] text-gray-400">{log.reason}</p>}
        </div>
      ),
    },
    {
      key: 'ip',
      title: 'IP 地址',
      render: (log: AuditLog) => (
        <code className="text-[11px] text-gray-600 font-mono">{log.ip_address}</code>
      ),
    },
    {
      key: 'outcome',
      title: '结果',
      render: (log: AuditLog) => (
        log.outcome === 'success' ? <Badge variant="success">成功</Badge> : <Badge variant="error">失败</Badge>
      ),
    },
    {
      key: 'time',
      title: '时间',
      render: (log: AuditLog) => (
        <span className="text-xs text-gray-600">{formatDate(log.created_at)}</span>
      ),
    },
  ]

  return (
    <div className="space-y-5">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-lg font-medium text-gray-900 dark:text-white">我的审计日志</h1>
          <p className="text-sm text-gray-500 mt-0.5">这里只显示与你账号相关的安全事件和操作记录</p>
        </div>
        <Button variant="secondary" size="sm" onClick={loadLogs} loading={loading}>
          刷新
        </Button>
      </div>

      <div className="flex flex-col sm:flex-row gap-2">
        <Card padding="none" className="flex-1 h-9 px-3">
          <div className="relative">
            <Search className="absolute left-0 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-600" />
            <input
              type="text"
              placeholder="搜索事件、应用或 IP..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="h-9 w-full bg-transparent pl-6 pr-2 text-sm text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-gray-700 focus:outline-none"
            />
          </div>
        </Card>

        <div className="flex h-9 items-center rounded-lg border border-gray-200 bg-gray-50 p-0.5 dark:border-white/[0.06] dark:bg-white/[0.02]">
          {(['all', 'success', 'failure'] as const).map((value) => (
            <button
              key={value}
              onClick={() => setFilter(value)}
              className={`h-full px-3 rounded-md text-xs font-medium transition-colors ${
                filter === value
                  ? 'bg-gray-900 text-white dark:bg-white dark:text-black'
                  : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
              }`}
            >
              {value === 'all' ? '全部' : value === 'success' ? '成功' : '失败'}
            </button>
          ))}
        </div>
      </div>

      <Card padding="none">
        <Table
          data={filteredLogs}
          columns={columns}
          keyExtractor={(log) => log.id}
          loading={loading}
          emptyState={
            <EmptyState
              icon={<ClipboardList className="w-6 h-6" />}
              title="暂无日志"
              description="没有找到匹配的个人审计日志记录"
            />
          }
        />
      </Card>
    </div>
  )
}