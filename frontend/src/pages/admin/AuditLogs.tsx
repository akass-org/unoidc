import { useState, useEffect } from 'react'
import { 
  ClipboardList, 
  Search, 
  Download,
  CheckCircle,
  XCircle,
  RefreshCw,
  Key,
  LogOut,
  UserPlus,
  Lock
} from 'lucide-react'
import { adminApi } from '#src/api/admin'
import { useToast, Button } from '#src/components/ui'
import { 
  Card, 
  Badge,
  EmptyState,
  Table
} from '#src/components/ui'
import { getErrorMessage } from '#src/api/client'

// Animation keyframes
const fadeIn = `@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }`
const slideUp = `@keyframes slideUp { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }`

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

export function AdminAuditLogs() {
  const [logs, setLogs] = useState<AuditLog[]>([])
  const [filteredLogs, setFilteredLogs] = useState<AuditLog[]>([])
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState('')
  const [filter, setFilter] = useState<'all' | 'success' | 'failure'>('all')
  const { addToast } = useToast()

  // Load logs
  useEffect(() => {
    loadLogs()
  }, [])

  // Filter logs
  useEffect(() => {
    let filtered = logs
    
    if (search) {
      filtered = filtered.filter(log =>
        log.username?.toLowerCase().includes(search.toLowerCase()) ||
        log.event_type.toLowerCase().includes(search.toLowerCase()) ||
        log.ip_address.includes(search)
      )
    }
    
    if (filter !== 'all') {
      filtered = filtered.filter(log => log.outcome === filter)
    }
    
    setFilteredLogs(filtered)
  }, [logs, search, filter])

  const loadLogs = async () => {
    try {
      setLoading(true)
      const data = await adminApi.getAuditLogs() as AuditLog[]
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

  const parseDate = (dateStr: string): Date => {
    if (!dateStr || typeof dateStr !== 'string') return new Date(NaN)
    
    // Rust OffsetDateTime::to_string() format: "2024-01-15 10:30:00.123456789 +00:00"
    // Replace first space (date/time sep) with T, remove second space (before timezone)
    let normalized = dateStr.trim().replace(/^([^ ]+) ([^ ]+) ([+-])/, '$1T$2$3')
    
    // Try to parse normalized string
    let date = new Date(normalized)
    if (!isNaN(date.getTime())) return date
    
    // Try removing microseconds if present (keep only milliseconds)
    const cleaned = normalized.replace(/\.(\d{3})\d+([+-]|Z)/, '.$1$2')
    date = new Date(cleaned)
    if (!isNaN(date.getTime())) return date
    
    // Fallback: try to extract date components
    const match = normalized.match(/^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})/)
    if (match) {
      const [, year, month, day, hour, minute, second] = match
      return new Date(Date.UTC(+year, +month - 1, +day, +hour, +minute, +second))
    }
    
    return new Date(NaN)
  }

  const formatDate = (dateStr: string) => {
    const date = parseDate(dateStr)
    if (isNaN(date.getTime())) return '-'
    return date.toLocaleString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
  }

  const columns = [
    {
      key: 'event',
      title: '事件',
      render: (log: AuditLog) => {
        const config = eventTypeConfig[log.event_type] || { 
          label: log.event_type, 
          icon: ClipboardList, 
          color: 'text-gray-500' 
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
      key: 'user',
      title: '用户',
      render: (log: AuditLog) => (
        <span className="text-sm text-gray-500">{log.username || '-'}</span>
      ),
    },
    {
      key: 'client',
      title: '应用',
      render: (log: AuditLog) => (
        <span className="text-xs text-gray-600">{log.client_name || '-'}</span>
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
        log.outcome === 'success' ? (
          <Badge variant="success">成功</Badge>
        ) : (
          <Badge variant="error">失败</Badge>
        )
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

  const todayLogs = logs.filter(l => {
    const logDate = parseDate(l.created_at)
    const today = new Date()
    if (isNaN(logDate.getTime())) return false
    return (
      logDate.getFullYear() === today.getFullYear() &&
      logDate.getMonth() === today.getMonth() &&
      logDate.getDate() === today.getDate()
    )
  })

  return (
    <div className="space-y-5" style={{ animation: 'slideUp 0.3s ease-out' }}>
      <style>{fadeIn}{slideUp}</style>
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-xl font-bold text-gray-900 dark:text-white">审计日志</h1>
          <p className="text-sm text-gray-500 mt-0.5">查看系统操作和安全事件记录</p>
        </div>
        <Button
          variant="secondary"
          size="sm"
          onClick={() => addToast({ type: 'info', title: '导出功能开发中' })}
        >
          <Download className="w-4 h-4" />
          导出
        </Button>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-4 gap-3">
        <Card className="text-center py-4">
          <p className="text-xl font-medium text-gray-900 dark:text-white">{todayLogs.length}</p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">今日事件</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-xl font-medium text-emerald-400">
            {todayLogs.filter(l => l.outcome === 'success').length}
          </p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">成功</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-xl font-medium text-red-400">
            {todayLogs.filter(l => l.outcome === 'failure').length}
          </p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">失败</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-xl font-medium text-gray-600 dark:text-gray-300">{logs.length}</p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">总计</p>
        </Card>
      </div>

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-2">
        <Card padding="none" className="flex-1 h-9 px-3">
          <div className="relative">
            <Search className="absolute left-0 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-600" />
            <input
              type="text"
              placeholder="搜索日志..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="h-9 w-full bg-transparent pl-6 pr-2 text-sm text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-gray-700 focus:outline-none"
            />
          </div>
        </Card>
        
        <div className="flex h-9 items-center rounded-lg border border-gray-200 bg-gray-50 p-0.5 dark:border-white/[0.06] dark:bg-white/[0.02]">
          {(['all', 'success', 'failure'] as const).map((f) => (
            <button
              key={f}
              onClick={() => setFilter(f)}
              className={`h-full px-3 rounded-md text-xs font-medium transition-colors ${
                filter === f
                  ? 'bg-gray-900 text-white dark:bg-white dark:text-black'
                  : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
              }`}
            >
              {f === 'all' ? '全部' : f === 'success' ? '成功' : '失败'}
            </button>
          ))}
        </div>
      </div>

      {/* Table */}
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
              description="没有找到匹配的审计日志记录"
            />
          }
        />
      </Card>

      {/* Pagination placeholder */}
      {filteredLogs.length > 0 && (
        <div className="flex items-center justify-between">
          <p className="text-xs text-gray-600">
            显示 {filteredLogs.length} 条记录
          </p>
          <div className="flex gap-2">
            <Button variant="ghost" size="sm" disabled>
              上一页
            </Button>
            <Button variant="ghost" size="sm" disabled>
              下一页
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}
