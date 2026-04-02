import { useState, useEffect } from 'react'
import { 
  Shield, 
  Search, 
  Plus, 
  Key,
  Trash2,
  Edit,
  Copy,
  Check,
  Minus,
  ChevronDown,
  ChevronUp,
  Link as LinkIcon
} from 'lucide-react'
import { adminApi } from '#src/api/admin'
import { useApi } from '#src/hooks'
import { 
  Card, 
  Button, 
  Input,
  Modal,
  Badge,
  EmptyState,
  useToast
} from '#src/components/ui'
import { getErrorMessage } from '#src/api/client'

// Animation keyframes
const fadeIn = `@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }`
const slideUp = `@keyframes slideUp { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }`

interface Client {
  id: string
  client_id: string
  name: string
  description?: string
  redirect_uris: string[]
  post_logout_redirect_uris?: string[]
  allowed_groups?: string[]
  is_active: boolean
  created_at: string
  last_used?: string
}

export function AdminClients() {
  const [clients, setClients] = useState<Client[]>([])
  const [filteredClients, setFilteredClients] = useState<Client[]>([])
  const [search, setSearch] = useState('')
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [editingClient, setEditingClient] = useState<Client | null>(null)
  const [deletingClient, setDeletingClient] = useState<Client | null>(null)
  const [resettingClient, setResettingClient] = useState<Client | null>(null)
  const [newClientSecret, setNewClientSecret] = useState<{ client: Client; secret: string } | null>(null)
  const [copiedField, setCopiedField] = useState<string | null>(null)
  const { addToast } = useToast()

  // Form states
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    redirect_uris: [''],
    post_logout_redirect_uris: [''],
  })

  // Load clients
  useEffect(() => {
    loadClients()
  }, [])

  // Filter clients
  useEffect(() => {
    const filtered = clients.filter(c =>
      c.name.toLowerCase().includes(search.toLowerCase()) ||
      c.client_id.toLowerCase().includes(search.toLowerCase())
    )
    setFilteredClients(filtered)
  }, [clients, search])

  const loadClients = async () => {
    try {
      const data = await adminApi.getClients() as Client[]
      setClients(data)
    } catch (err) {
      addToast({
        type: 'error',
        title: '加载失败',
        message: getErrorMessage(err),
      })
    }
  }

  const { loading: creating, execute: createClient } = useApi(
    adminApi.createClient,
    {
      successMessage: '应用创建成功',
      onSuccess: (data) => {
        const result = data as { client: Client; client_secret: string }
        setNewClientSecret({ client: result.client, secret: result.client_secret })
        setShowCreateModal(false)
        setFormData({ name: '', description: '', redirect_uris: [''], post_logout_redirect_uris: [''] })
        loadClients()
      }
    }
  )

  const { loading: updating, execute: updateClient } = useApi(
    (id: string, data: Record<string, unknown>) => adminApi.updateClient(id, data),
    {
      successMessage: '应用更新成功',
      onSuccess: () => {
        setEditingClient(null)
        loadClients()
      }
    }
  )

  const { loading: deleting, execute: deleteClient } = useApi(
    (id: string) => adminApi.deleteClient(id),
    {
      successMessage: '应用已删除',
      onSuccess: () => {
        setDeletingClient(null)
        loadClients()
      }
    }
  )

  const { loading: resetting, execute: resetSecret } = useApi(
    (id: string) => adminApi.resetClientSecret(id),
    {
      successMessage: '密钥已重置',
      onSuccess: (data) => {
        const result = data as { client: Client; client_secret: string }
        setNewClientSecret({ client: result.client, secret: result.client_secret })
        setResettingClient(null)
        loadClients()
      }
    }
  )

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    await createClient({
      name: formData.name,
      description: formData.description,
      redirect_uris: formData.redirect_uris.filter(Boolean),
      post_logout_redirect_uris: formData.post_logout_redirect_uris.filter(Boolean).length > 0 
        ? formData.post_logout_redirect_uris.filter(Boolean) 
        : undefined,
    })
  }

  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!editingClient) return
    await updateClient(editingClient.id, {
      name: editingClient.name,
      description: editingClient.description,
      redirect_uris: editingClient.redirect_uris,
      post_logout_redirect_uris: editingClient.post_logout_redirect_uris,
      is_active: editingClient.is_active,
    })
  }

  const handleDelete = async () => {
    if (!deletingClient) return
    await deleteClient(deletingClient.id)
  }

  const handleResetSecret = async () => {
    if (!resettingClient) return
    await resetSecret(resettingClient.id)
  }

  const handleCopy = async (text: string, field: string) => {
    await navigator.clipboard.writeText(text)
    setCopiedField(field)
    setTimeout(() => setCopiedField(null), 2000)
  }

  const getOidcEndpoints = () => {
    const baseUrl = window.location.origin
    return {
      issuer: baseUrl,
      well_known_endpoint: `${baseUrl}/.well-known/openid-configuration`,
      authorization_endpoint: `${baseUrl}/authorize`,
      token_endpoint: `${baseUrl}/token`,
      userinfo_endpoint: `${baseUrl}/userinfo`,
      jwks_uri: `${baseUrl}/jwks.json`,
      end_session_endpoint: `${baseUrl}/logout`,
    }
  }

  return (
    <div className="space-y-5" style={{ animation: 'slideUp 0.3s ease-out' }}>
      <style>{fadeIn}{slideUp}</style>
      
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-xl font-bold text-gray-900 dark:text-white">应用管理</h1>
          <p className="text-sm text-gray-500 mt-1">管理 OIDC 客户端和授权配置</p>
        </div>
        <Button onClick={() => setShowCreateModal(true)} size="sm">
          <Plus className="w-4 h-4 mr-1.5" />
          创建应用
        </Button>
      </div>

      {/* Search */}
      <Card padding="sm">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="搜索应用..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full bg-transparent pl-9 pr-4 py-2 text-sm text-gray-900 dark:text-white placeholder:text-gray-400 focus:outline-none"
          />
        </div>
      </Card>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-3">
        <Card className="text-center py-4">
          <p className="text-2xl font-bold text-gray-900 dark:text-white">{clients.length}</p>
          <p className="text-[11px] text-gray-500 uppercase tracking-wider mt-1">应用</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-2xl font-bold text-emerald-500">
            {clients.filter(c => c.is_active).length}
          </p>
          <p className="text-[11px] text-gray-500 uppercase tracking-wider mt-1">活跃</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-2xl font-bold text-gray-600 dark:text-gray-300">
            {clients.filter(c => c.last_used).length}
          </p>
          <p className="text-[11px] text-gray-500 uppercase tracking-wider mt-1">已使用</p>
        </Card>
      </div>

      {/* Clients List */}
      {filteredClients.length === 0 ? (
        <Card padding="lg">
          <EmptyState
            icon={<Shield className="w-8 h-8" />}
            title={search ? '无匹配结果' : '暂无应用'}
            description={search ? '尝试其他搜索词' : '点击上方按钮创建第一个应用'}
          />
        </Card>
      ) : (
        <div className="space-y-3">
          {filteredClients.map((client) => {
            const isExpanded = expandedId === client.id
            
            return (
              <Card key={client.id} className="overflow-hidden">
                {/* Header Row */}
                <div 
                  className="flex items-start gap-3 p-4 cursor-pointer hover:bg-gray-50/50 dark:hover:bg-white/[0.02] transition-colors"
                  onClick={() => setExpandedId(isExpanded ? null : client.id)}
                >
                  <div className="w-10 h-10 rounded-lg bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.06] flex items-center justify-center shrink-0">
                    <Shield className="w-5 h-5 text-gray-500" />
                  </div>

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                      <h3 className="font-medium text-gray-900 dark:text-white">{client.name}</h3>
                      <Badge variant={client.is_active ? 'success' : 'error'}>
                        {client.is_active ? '活跃' : '禁用'}
                      </Badge>
                    </div>
                    <code className="text-xs text-gray-500 font-mono block mt-0.5">{client.client_id}</code>
                    
                    <div className="flex flex-wrap items-center gap-3 mt-2 text-xs text-gray-500">
                      <span className="inline-flex items-center gap-1">
                        <LinkIcon className="w-3 h-3" />
                        {client.redirect_uris.length} 个回调
                      </span>
                      {client.allowed_groups && client.allowed_groups.length > 0 && (
                        <span>{client.allowed_groups.length} 个用户组</span>
                      )}
                    </div>
                  </div>

                  <div className="flex items-center gap-0.5 shrink-0">
                    <button
                      onClick={(e) => { e.stopPropagation(); setEditingClient(client) }}
                      className="p-1.5 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-white/[0.04] rounded transition-colors"
                      title="编辑"
                    >
                      <Edit className="w-4 h-4" />
                    </button>
                    <button
                      onClick={(e) => { e.stopPropagation(); setResettingClient(client) }}
                      className="p-1.5 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-white/[0.04] rounded transition-colors"
                      title="重置密钥"
                    >
                      <Key className="w-4 h-4" />
                    </button>
                    <button
                      onClick={(e) => { e.stopPropagation(); setDeletingClient(client) }}
                      className="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-500/[0.08] rounded transition-colors"
                      title="删除"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                    <div className="ml-1 text-gray-400">
                      {isExpanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                    </div>
                  </div>
                </div>

                {/* Expanded Details */}
                {isExpanded && (
                  <div className="border-t border-gray-200 dark:border-white/[0.04] px-4 py-4 bg-gray-50/30 dark:bg-white/[0.02]">
                    {/* OIDC Config */}
                    <div>
                      <h4 className="text-xs font-medium text-gray-500 mb-2 flex items-center gap-1.5">
                        <Shield className="w-3.5 h-3.5" />
                        OIDC 配置
                      </h4>
                      <div className="bg-white dark:bg-black/20 rounded-lg border border-gray-200 dark:border-white/[0.06] p-3 space-y-2">
                        {(() => {
                          const endpoints = getOidcEndpoints()
                          const items = [
                            { label: 'Issuer', value: endpoints.issuer },
                            { label: 'Well-Known', value: endpoints.well_known_endpoint },
                            { label: 'Client ID', value: client.client_id },
                            { label: '授权端点', value: endpoints.authorization_endpoint },
                            { label: '令牌端点', value: endpoints.token_endpoint },
                            { label: '用户信息', value: endpoints.userinfo_endpoint },
                            { label: 'JWKS', value: endpoints.jwks_uri },
                          ]
                          return items.map(({ label, value }) => (
                            <div key={label} className="flex items-center gap-2 group">
                              <span className="text-xs text-gray-500 w-20 shrink-0">{label}</span>
                              <code className="flex-1 text-xs font-mono text-gray-700 dark:text-gray-300 truncate">{value}</code>
                              <button
                                onClick={() => handleCopy(value, `${client.id}-${label}`)}
                                className="p-1 text-gray-400 hover:text-gray-700 opacity-0 group-hover:opacity-100 transition-opacity"
                              >
                                {copiedField === `${client.id}-${label}` ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3" />}
                              </button>
                            </div>
                          ))
                        })()}
                      </div>
                    </div>

                    {/* Callback URLs */}
                    <div className="mt-4">
                      <h4 className="text-xs font-medium text-gray-500 mb-2 flex items-center gap-1.5">
                        <LinkIcon className="w-3.5 h-3.5" />
                        回调地址
                      </h4>
                      <div className="space-y-2">
                        <div>
                          <span className="text-[10px] text-gray-400 uppercase tracking-wider">登录回调</span>
                          <div className="mt-1 space-y-1.5">
                            {client.redirect_uris.length > 0 ? (
                              client.redirect_uris.map((uri, i) => (
                                <div key={i} className="flex items-center gap-2 group">
                                  <code className="flex-1 text-xs font-mono text-gray-700 dark:text-gray-300 bg-white dark:bg-black/20 px-3 py-2 rounded border border-gray-200 dark:border-white/[0.06] truncate">{uri}</code>
                                  <button
                                    onClick={() => handleCopy(uri, `redirect-${i}`)}
                                    className="p-1.5 text-gray-400 hover:text-gray-700 opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                                  >
                                    {copiedField === `redirect-${i}` ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5" />}
                                  </button>
                                </div>
                              ))
                            ) : (
                              <div className="flex items-center gap-2">
                                <span className="flex-1 text-xs text-gray-400 italic bg-white dark:bg-black/20 px-3 py-2 rounded border border-dashed border-gray-200 dark:border-white/[0.06]">未配置</span>
                                <div className="w-8 shrink-0"></div>
                              </div>
                            )}
                          </div>
                        </div>
                        {client.post_logout_redirect_uris && client.post_logout_redirect_uris.length > 0 && (
                          <div>
                            <span className="text-[10px] text-gray-400 uppercase tracking-wider">登出回调</span>
                            <div className="mt-1 space-y-1.5">
                              {client.post_logout_redirect_uris.map((uri, i) => (
                                <div key={i} className="flex items-center gap-2 group">
                                  <code className="flex-1 text-xs font-mono text-gray-700 dark:text-gray-300 bg-white dark:bg-black/20 px-3 py-2 rounded border border-gray-200 dark:border-white/[0.06] truncate">{uri}</code>
                                  <button
                                    onClick={() => handleCopy(uri, `logout-${i}`)}
                                    className="p-1.5 text-gray-400 hover:text-gray-700 opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                                  >
                                    {copiedField === `logout-${i}` ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5" />}
                                  </button>
                                </div>
                              ))}
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                )}
              </Card>
            )
          })}
        </div>
      )}

      {/* Create Modal */}
      <Modal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        title="创建应用"
        description="注册新的 OIDC 客户端"
        footer={
          <>
            <Button variant="ghost" onClick={() => setShowCreateModal(false)}>取消</Button>
            <Button onClick={handleCreate} loading={creating}>创建</Button>
          </>
        }
      >
        <form className="space-y-4">
          <Input
            label="应用名称 *"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            placeholder="例如: 我的应用"
            required
          />
          <div>
            <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">描述</label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="应用描述（可选）"
              rows={2}
              className="w-full bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all resize-none"
            />
          </div>
          
          {/* Login Redirect URIs */}
          <div>
            <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">登录回调 URI</label>
            <div className="space-y-2">
              {formData.redirect_uris.map((uri, index) => (
                <div key={index} className="flex gap-2">
                  <input
                    type="url"
                    value={uri}
                    onChange={(e) => {
                      const newUris = [...formData.redirect_uris]
                      newUris[index] = e.target.value
                      setFormData({ ...formData, redirect_uris: newUris })
                    }}
                    placeholder="https://example.com/callback"
                    className="flex-1 bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all font-mono"
                  />
                  <button
                    type="button"
                    onClick={() => setFormData({ ...formData, redirect_uris: formData.redirect_uris.filter((_, i) => i !== index) })}
                    className="p-2.5 text-gray-500 hover:text-red-500 hover:bg-red-500/[0.08] rounded-lg border border-gray-200 dark:border-white/[0.08] transition-colors"
                  >
                    <Minus className="w-4 h-4" />
                  </button>
                </div>
              ))}
              <button
                type="button"
                onClick={() => setFormData({ ...formData, redirect_uris: [...formData.redirect_uris, ''] })}
                className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 border border-dashed border-gray-300 dark:border-white/[0.12] hover:border-gray-400 dark:hover:border-white/20 rounded-lg transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                添加回调地址
              </button>
            </div>
          </div>

          {/* Logout Redirect URIs */}
          <div>
            <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">登出回调 URI（可选）</label>
            <div className="space-y-2">
              {formData.post_logout_redirect_uris.map((uri, index) => (
                <div key={index} className="flex gap-2">
                  <input
                    type="url"
                    value={uri}
                    onChange={(e) => {
                      const newUris = [...formData.post_logout_redirect_uris]
                      newUris[index] = e.target.value
                      setFormData({ ...formData, post_logout_redirect_uris: newUris })
                    }}
                    placeholder="https://example.com/logout"
                    className="flex-1 bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all font-mono"
                  />
                  <button
                    type="button"
                    onClick={() => setFormData({ ...formData, post_logout_redirect_uris: formData.post_logout_redirect_uris.filter((_, i) => i !== index) })}
                    className="p-2.5 text-gray-500 hover:text-red-500 hover:bg-red-500/[0.08] rounded-lg border border-gray-200 dark:border-white/[0.08] transition-colors"
                  >
                    <Minus className="w-4 h-4" />
                  </button>
                </div>
              ))}
              <button
                type="button"
                onClick={() => setFormData({ ...formData, post_logout_redirect_uris: [...formData.post_logout_redirect_uris, ''] })}
                className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 border border-dashed border-gray-300 dark:border-white/[0.12] hover:border-gray-400 dark:hover:border-white/20 rounded-lg transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                添加登出回调
              </button>
            </div>
          </div>
        </form>
      </Modal>

      {/* Edit Modal */}
      {editingClient && (
        <Modal
          isOpen={!!editingClient}
          onClose={() => setEditingClient(null)}
          title="编辑应用"
          footer={
            <>
              <Button variant="ghost" onClick={() => setEditingClient(null)}>取消</Button>
              <Button onClick={handleUpdate} loading={updating}>保存</Button>
            </>
          }
        >
          <form className="space-y-4">
            <Input
              label="应用名称"
              value={editingClient.name}
              onChange={(e) => setEditingClient({ ...editingClient, name: e.target.value })}
            />
            <div>
              <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">描述</label>
              <textarea
                value={editingClient.description || ''}
                onChange={(e) => setEditingClient({ ...editingClient, description: e.target.value })}
                rows={2}
                className="w-full bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all resize-none"
              />
            </div>
            
            {/* Login Redirect URIs */}
            <div>
              <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">登录回调 URI</label>
              <div className="space-y-2">
                {editingClient.redirect_uris.map((uri, index) => (
                  <div key={index} className="flex gap-2">
                    <input
                      type="url"
                      value={uri}
                      onChange={(e) => {
                        const newUris = [...editingClient.redirect_uris]
                        newUris[index] = e.target.value
                        setEditingClient({ ...editingClient, redirect_uris: newUris })
                      }}
                      placeholder="https://example.com/callback"
                      className="flex-1 bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all font-mono"
                    />
                    <button
                      type="button"
                      onClick={() => setEditingClient({ ...editingClient, redirect_uris: editingClient.redirect_uris.filter((_, i) => i !== index) })}
                      className="p-2.5 text-gray-500 hover:text-red-500 hover:bg-red-500/[0.08] rounded-lg border border-gray-200 dark:border-white/[0.08] transition-colors"
                    >
                      <Minus className="w-4 h-4" />
                    </button>
                  </div>
                ))}
                <button
                  type="button"
                  onClick={() => setEditingClient({ ...editingClient, redirect_uris: [...editingClient.redirect_uris, ''] })}
                  className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 border border-dashed border-gray-300 dark:border-white/[0.12] hover:border-gray-400 dark:hover:border-white/20 rounded-lg transition-colors"
                >
                  <Plus className="w-3.5 h-3.5" />
                  添加回调地址
                </button>
              </div>
            </div>

            {/* Logout Redirect URIs */}
            <div>
              <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">登出回调 URI（可选）</label>
              <div className="space-y-2">
                {(editingClient.post_logout_redirect_uris || []).map((uri, index) => (
                  <div key={index} className="flex gap-2">
                    <input
                      type="url"
                      value={uri}
                      onChange={(e) => {
                        const newUris = [...(editingClient.post_logout_redirect_uris || [])]
                        newUris[index] = e.target.value
                        setEditingClient({ ...editingClient, post_logout_redirect_uris: newUris })
                      }}
                      placeholder="https://example.com/logout"
                      className="flex-1 bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all font-mono"
                    />
                    <button
                      type="button"
                      onClick={() => setEditingClient({ ...editingClient, post_logout_redirect_uris: (editingClient.post_logout_redirect_uris || []).filter((_, i) => i !== index) })}
                      className="p-2.5 text-gray-500 hover:text-red-500 hover:bg-red-500/[0.08] rounded-lg border border-gray-200 dark:border-white/[0.08] transition-colors"
                    >
                      <Minus className="w-4 h-4" />
                    </button>
                  </div>
                ))}
                <button
                  type="button"
                  onClick={() => setEditingClient({ ...editingClient, post_logout_redirect_uris: [...(editingClient.post_logout_redirect_uris || []), ''] })}
                  className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 border border-dashed border-gray-300 dark:border-white/[0.12] hover:border-gray-400 dark:hover:border-white/20 rounded-lg transition-colors"
                >
                  <Plus className="w-3.5 h-3.5" />
                  添加登出回调
                </button>
              </div>
            </div>

            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={editingClient.is_active}
                onChange={(e) => setEditingClient({ ...editingClient, is_active: e.target.checked })}
                className="w-4 h-4 rounded border-gray-200 dark:border-white/[0.12] bg-gray-50 dark:bg-white/[0.04] text-white focus:ring-white/20"
              />
              <span className="text-sm text-gray-600 dark:text-gray-400">启用应用</span>
            </label>
          </form>
        </Modal>
      )}

      {/* Delete Modal */}
      {deletingClient && (
        <Modal
          isOpen={!!deletingClient}
          onClose={() => setDeletingClient(null)}
          title="删除应用"
          description={`确定要删除应用 "${deletingClient.name}" 吗？此操作不可恢复。`}
          footer={
            <>
              <Button variant="ghost" onClick={() => setDeletingClient(null)}>取消</Button>
              <Button variant="danger" onClick={handleDelete} loading={deleting}>删除</Button>
            </>
          }
        >
          <div className="p-3 bg-red-500/[0.08] border border-red-500/[0.16] rounded-lg">
            <p className="text-sm text-red-400">警告：删除应用后，所有使用该应用的客户端将无法继续登录。</p>
          </div>
        </Modal>
      )}

      {/* Reset Secret Confirm Modal */}
      {resettingClient && (
        <Modal
          isOpen={!!resettingClient}
          onClose={() => setResettingClient(null)}
          title="重置密钥"
          description={`确定要重置 "${resettingClient.name}" 的密钥吗？旧密钥将立即失效。`}
          footer={
            <>
              <Button variant="ghost" onClick={() => setResettingClient(null)}>取消</Button>
              <Button variant="danger" onClick={handleResetSecret} loading={resetting}>确认重置</Button>
            </>
          }
        >
          <div className="p-3 bg-amber-500/[0.08] border border-amber-500/[0.16] rounded-lg">
            <p className="text-sm text-amber-400">需要使用新密钥更新所有客户端配置。</p>
          </div>
        </Modal>
      )}

      {/* Secret Display Modal */}
      {newClientSecret && (
        <Modal
          isOpen={!!newClientSecret}
          onClose={() => setNewClientSecret(null)}
          title="Client Secret"
          description={`${newClientSecret.client.name} 的密钥`}
          footer={<Button onClick={() => setNewClientSecret(null)}>我已保存</Button>}
        >
          <div className="space-y-4">
            <div className="p-3 bg-amber-500/[0.08] border border-amber-500/[0.16] rounded-lg">
              <p className="text-sm text-amber-400 font-medium mb-1">请立即复制并保存此密钥</p>
              <p className="text-xs text-amber-400/70">密钥只显示一次，关闭后将无法再次查看。</p>
            </div>
            <div>
              <label className="block text-xs font-medium text-gray-500 mb-1.5">Client ID</label>
              <code className="block w-full p-2.5 bg-gray-50 dark:bg-black/20 rounded text-xs font-mono text-gray-500 break-all border border-gray-200 dark:border-white/[0.06]">
                {newClientSecret.client.client_id}
              </code>
            </div>
            <div>
              <label className="block text-xs font-medium text-gray-500 mb-1.5">Client Secret</label>
              <div className="relative">
                <code className="block w-full p-2.5 pr-10 bg-gray-50 dark:bg-black/20 rounded text-xs font-mono text-gray-900 dark:text-white break-all border border-gray-200 dark:border-white/[0.06]">
                  {newClientSecret.secret}
                </code>
                <button
                  onClick={() => handleCopy(newClientSecret.secret, 'secret')}
                  className="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-gray-500 hover:text-gray-900 dark:hover:text-white transition-colors"
                >
                  {copiedField === 'secret' ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
                </button>
              </div>
            </div>
          </div>
        </Modal>
      )}
    </div>
  )
}
