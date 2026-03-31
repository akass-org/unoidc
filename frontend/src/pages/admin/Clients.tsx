import { useState } from 'react'

interface Client {
  id: string
  client_id: string
  name: string
  redirect_uris: string[]
  is_active: boolean
  created_at: string
  last_used?: string
}

const mockClients: Client[] = [
  {
    id: '1',
    client_id: 'demo-app-123456',
    name: '示例应用',
    redirect_uris: ['http://localhost:3000/callback'],
    is_active: true,
    created_at: '2025-03-01',
    last_used: '2025-03-30',
  },
  {
    id: '2',
    client_id: 'internal-dashboard',
    name: '内部仪表盘',
    redirect_uris: ['https://dashboard.example.com/auth/callback'],
    is_active: true,
    created_at: '2025-01-15',
  },
]

export function AdminClients() {
  const [clients, setClients] = useState<Client[]>(mockClients)
  const [search, setSearch] = useState('')
  const [showModal, setShowModal] = useState(false)
  const [showSecret, setShowSecret] = useState<string | null>(null)

  const filteredClients = clients.filter((c) =>
    c.name.toLowerCase().includes(search.toLowerCase()) ||
    c.client_id.toLowerCase().includes(search.toLowerCase())
  )

  function handleCreateClient() {
    const newClient: Client = {
      id: Date.now().toString(),
      client_id: `client-${Math.random().toString(36).substring(2, 10)}`,
      name: '新应用',
      redirect_uris: ['http://localhost:3000/callback'],
      is_active: true,
      created_at: new Date().toISOString().split('T')[0],
    }
    setClients([...clients, newClient])
    setShowSecret(newClient.id)
    setShowModal(false)
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">OIDC Client 管理</h1>
        <button
          onClick={() => setShowModal(true)}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors"
        >
          创建 Client
        </button>
      </div>

      {/* Search */}
      <div className="mb-6">
        <input
          type="text"
          placeholder="搜索 Client..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full max-w-md px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {/* Cards */}
      <div className="space-y-4">
        {filteredClients.map((client) => (
          <div
            key={client.id}
            className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-6"
          >
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500 to-cyan-600 flex items-center justify-center text-white text-xl">
                  🔐
                </div>
                <div>
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white">{client.name}</h3>
                  <p className="text-sm text-gray-500 dark:text-gray-400 font-mono">{client.client_id}</p>
                </div>
                {client.is_active ? (
                  <span className="px-2 py-1 text-xs font-medium rounded-full bg-green-100 dark:bg-green-900/20 text-green-700 dark:text-green-300">
                    活跃
                  </span>
                ) : (
                  <span className="px-2 py-1 text-xs font-medium rounded-full bg-red-100 dark:bg-red-900/20 text-red-700 dark:text-red-300">
                    禁用
                  </span>
                )}
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() => setShowSecret(client.id)}
                  className="px-3 py-1.5 text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400 border border-blue-200 dark:border-blue-800 rounded-lg hover:bg-blue-50 dark:hover:bg-blue-900/20"
                >
                  重置密钥
                </button>
                <button className="px-3 py-1.5 text-sm text-gray-600 hover:text-gray-700 dark:text-gray-400 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800">
                  编辑
                </button>
              </div>
            </div>

            {/* Secret Warning */}
            {showSecret === client.id && (
              <div className="mb-4 p-4 rounded-lg bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800">
                <p className="text-sm text-yellow-800 dark:text-yellow-300 font-medium mb-2">
                  ⚠️ 请保存以下 Client Secret，它将只显示一次
                </p>
                <code className="block p-2 bg-black/10 dark:bg-white/10 rounded text-sm font-mono break-all">
                  {Math.random().toString(36).substring(2, 34)}
                </code>
                <button
                  onClick={() => setShowSecret(null)}
                  className="mt-2 text-xs text-yellow-700 dark:text-yellow-400 hover:underline"
                >
                  我已保存
                </button>
              </div>
            )}

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div>
                <p className="text-gray-500 dark:text-gray-400 mb-1">重定向 URI</p>
                <ul className="space-y-1">
                  {client.redirect_uris.map((uri, i) => (
                    <li key={i} className="font-mono text-gray-700 dark:text-gray-300">{uri}</li>
                  ))}
                </ul>
              </div>
              <div className="flex gap-8">
                <div>
                  <p className="text-gray-500 dark:text-gray-400 mb-1">创建时间</p>
                  <p className="text-gray-700 dark:text-gray-300">{client.created_at}</p>
                </div>
                {client.last_used && (
                  <div>
                    <p className="text-gray-500 dark:text-gray-400 mb-1">最后使用</p>
                    <p className="text-gray-700 dark:text-gray-300">{client.last_used}</p>
                  </div>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Modal */}
      {showModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50">
          <div className="bg-white dark:bg-gray-900 rounded-xl shadow-xl max-w-md w-full p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">创建 OIDC Client</h3>
            <div className="space-y-4">
              <input
                type="text"
                placeholder="应用名称"
                className="w-full px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm"
              />
              <input
                type="url"
                placeholder="重定向 URI"
                className="w-full px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm"
              />
              <p className="text-xs text-gray-500">
                创建后将显示 Client Secret，请妥善保存。
              </p>
            </div>
            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setShowModal(false)}
                className="px-4 py-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
              >
                取消
              </button>
              <button
                onClick={handleCreateClient}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}