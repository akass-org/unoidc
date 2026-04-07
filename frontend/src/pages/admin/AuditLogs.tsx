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
  attempted_username?: string
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
  const [debouncedSearch, setDebouncedSearch] = useState('')
  const [filter, setFilter] = useState<'all' | 'success' | 'failure'>('all')
  const [currentPage, setCurrentPage] = useState(1)
  const [pageSize, setPageSize] = useState(20)
  const { addToast } = useToast()

  // Load logs
  useEffect(() => {
    loadLogs()
  }, [])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedSearch(search.trim().toLowerCase())
    }, 200)
    return () => window.clearTimeout(timer)
  }, [search])

  // Filter logs
  useEffect(() => {
    let filtered = logs
    
    if (debouncedSearch) {
      filtered = filtered.filter(log =>
        log.username?.toLowerCase().includes(debouncedSearch) ||
        log.attempted_username?.toLowerCase().includes(debouncedSearch) ||
        log.client_name?.toLowerCase().includes(debouncedSearch) ||
        log.event_type.toLowerCase().includes(debouncedSearch) ||
        log.ip_address.includes(debouncedSearch)
      )
    }
    
    if (filter !== 'all') {
      filtered = filtered.filter(log => log.outcome === filter)
    }
    
    setFilteredLogs(filtered)
    setCurrentPage(1)
  }, [logs, debouncedSearch, filter])

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
    
    try {
      // Support RFC 3339 format: "2024-01-15T10:30:00+00:00"
      // and Rust to_string format: "2024-01-15 10:30:00.123456789 +00:00"
      let normalized = dateStr.trim()
      
      // If it has space instead of T, replace it
      if (normalized.includes(' ') && !normalized.includes('T')) {
        normalized = normalized.replace(/^(\d{4}-\d{2}-\d{2}) /, '$1T')
      }
      
      // Remove fractional seconds if too long
      normalized = normalized.replace(/(\d{2}:\d{2}:\d{2})\.(\d{6})\d+/, '$1.$2')
      
      // Parse as ISO date
      const date = new Date(normalized)
      return date
    } catch {
      return new Date(NaN)
    }
  }

  const formatDate = (dateStr: string) => {
    const date = parseDate(dateStr)
    if (isNaN(date.getTime())) return dateStr || '-'
    
    // Format as: "2024/4/2 10:30"
    return date.toLocaleString('zh-CN', { 
      year: 'numeric',
      month: 'numeric', 
      day: 'numeric', 
      hour: '2-digit', 
      minute: '2-digit',
      hour12: false
    })
  }

  const columns = [
    {
      key: 'event',
      title: '事件',
      width: '180px',
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
            <span className="text-sm text-gray-900 dark:text-white whitespace-nowrap">{config.label}</span>
          </div>
        )
      },
    },
    {
      key: 'user',
      title: '用户',
      width: '300px',
      render: (log: AuditLog) => (
        <span
          className="block max-w-[300px] truncate whitespace-nowrap text-sm text-gray-500"
          title={log.username || log.attempted_username || '-'}
        >
          {log.username || log.attempted_username || '-'}
        </span>
      ),
    },
    {
      key: 'client',
      title: '应用',
      width: '200px',
      render: (log: AuditLog) => (
        <span className="block max-w-[200px] truncate whitespace-nowrap text-xs text-gray-600" title={log.client_name || '-'}>
          {log.client_name || '-'}
        </span>
      ),
    },
    {
      key: 'ip',
      title: 'IP 地址',
      width: '170px',
      render: (log: AuditLog) => (
        <code className="block max-w-[170px] truncate whitespace-nowrap text-[11px] text-gray-600 font-mono" title={log.ip_address}>{log.ip_address}</code>
      ),
    },
    {
      key: 'outcome',
      title: '结果',
      width: '112px',
      render: (log: AuditLog) => (
        log.outcome === 'success' ? (
          <Badge variant="success" size="md">成功</Badge>
        ) : (
          <Badge variant="error" size="md">失败</Badge>
        )
      ),
    },
    {
      key: 'time',
      title: '时间',
      width: '168px',
      render: (log: AuditLog) => (
        <span className="text-xs text-gray-600 whitespace-nowrap">{formatDate(log.created_at)}</span>
      ),
    },
  ]

  const totalPages = Math.max(1, Math.ceil(filteredLogs.length / pageSize))
  const pageStart = (currentPage - 1) * pageSize
  const pagedLogs = filteredLogs.slice(pageStart, pageStart + pageSize)

  useEffect(() => {
    if (currentPage > totalPages) {
      setCurrentPage(totalPages)
    }
  }, [currentPage, totalPages])

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
          <p className="text-2xl font-bold text-gray-900 dark:text-white">{todayLogs.length}</p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">今日事件</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-2xl font-bold text-emerald-400">
            {logs.filter(l => l.outcome === 'success').length}
          </p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">成功</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-2xl font-bold text-red-400">
            {logs.filter(l => l.outcome === 'failure').length}
          </p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">失败</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-2xl font-bold text-gray-600 dark:text-gray-300">{logs.length}</p>
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

        <Card padding="none" className="h-9 px-2">
          <select
            value={pageSize}
            onChange={(e) => setPageSize(Number(e.target.value))}
            className="h-9 bg-transparent px-2 text-xs text-gray-700 dark:text-gray-200 focus:outline-none"
          >
            <option value={20}>20/页</option>
            <option value={50}>50/页</option>
            <option value={100}>100/页</option>
          </select>
        </Card>
      </div>

      {/* Table */}
      <Card padding="none">
        <Table
          data={pagedLogs}
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
        <div className="flex items-center justify-between gap-2">
          <p className="text-xs text-gray-600">
            显示 {pageStart + 1}-{Math.min(pageStart + pageSize, filteredLogs.length)} 条，共 {filteredLogs.length} 条
          </p>
          <div className="flex gap-2">
            <Button
              variant="ghost"
              size="sm"
              disabled={currentPage <= 1}
              onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
            >
              上一页
            </Button>
            <span className="px-2 py-1 text-xs text-gray-500 whitespace-nowrap">
              第 {currentPage}/{totalPages} 页
            </span>
            <Button
              variant="ghost"
              size="sm"
              disabled={currentPage >= totalPages}
              onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
            >
              下一页
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}
