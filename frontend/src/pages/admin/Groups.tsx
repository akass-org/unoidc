import { useState } from 'react'

interface Group {
  id: string
  name: string
  description: string
  member_count: number
  created_at: string
}

const mockGroups: Group[] = [
  { id: '1', name: 'admin', description: '系统管理员组', member_count: 2, created_at: '2025-01-01' },
  { id: '2', name: 'developers', description: '开发团队', member_count: 5, created_at: '2025-02-15' },
  { id: '3', name: 'users', description: '普通用户组', member_count: 100, created_at: '2025-01-01' },
]

export function AdminGroups() {
  const [groups] = useState<Group[]>(mockGroups)
  const [search, setSearch] = useState('')
  const [showModal, setShowModal] = useState(false)

  const filteredGroups = groups.filter((g) =>
    g.name.toLowerCase().includes(search.toLowerCase()) ||
    g.description.toLowerCase().includes(search.toLowerCase())
  )

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">用户组管理</h1>
        <button
          onClick={() => setShowModal(true)}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors"
        >
          添加用户组
        </button>
      </div>

      {/* Search */}
      <div className="mb-6">
        <input
          type="text"
          placeholder="搜索用户组..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full max-w-md px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {/* Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {filteredGroups.map((group) => (
          <div
            key={group.id}
            className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-6"
          >
            <div className="flex items-start justify-between mb-4">
              <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-purple-500 to-pink-600 flex items-center justify-center text-white text-xl">
                🏷️
              </div>
              <button className="text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400">
                编辑
              </button>
            </div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-1">{group.name}</h3>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">{group.description}</p>
            <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
              <span>{group.member_count} 成员</span>
              <span>创建于 {group.created_at}</span>
            </div>
          </div>
        ))}
      </div>

      {/* Modal */}
      {showModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50">
          <div className="bg-white dark:bg-gray-900 rounded-xl shadow-xl max-w-md w-full p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">添加用户组</h3>
            <div className="space-y-4">
              <input
                type="text"
                placeholder="组名称"
                className="w-full px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm"
              />
              <textarea
                placeholder="描述"
                rows={3}
                className="w-full px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm"
              />
            </div>
            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setShowModal(false)}
                className="px-4 py-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
              >
                取消
              </button>
              <button
                onClick={() => setShowModal(false)}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg"
              >
                添加
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}