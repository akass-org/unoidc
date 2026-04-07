import { useState, useEffect } from 'react'
import { 
  Users, 
  Search, 
  Plus, 
  MoreHorizontal, 
  Key
} from 'lucide-react'
import { adminApi } from '#src/api/admin'
import { useApi } from '#src/hooks'
import { 
  Card, 
  Button, 
  Input,
  Modal,
  Badge,
  Avatar,
  Table,
  EmptyState,
  useToast
} from '#src/components/ui'
import { getErrorMessage } from '#src/api/client'

// Animation keyframes
const fadeIn = `@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }`
const slideUp = `@keyframes slideUp { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }`

interface User {
  id: string
  username: string
  email: string
  display_name: string
  is_admin: boolean
  is_active: boolean
  created_at: string
}

export function AdminUsers() {
  const [users, setUsers] = useState<User[]>([])
  const [filteredUsers, setFilteredUsers] = useState<User[]>([])
  const [search, setSearch] = useState('')
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [editingUser, setEditingUser] = useState<User | null>(null)
  const [resetPasswordUser, setResetPasswordUser] = useState<User | null>(null)
  const [resetPasswordSuccess, setResetPasswordSuccess] = useState(false)
  const { addToast } = useToast()

  // Form states
  const [formData, setFormData] = useState({
    username: '',
    email: '',
    display_name: '',
    password: '',
    is_admin: false,
  })

  // Load users
  useEffect(() => {
    loadUsers()
  }, [])

  // Filter users
  useEffect(() => {
    const filtered = users.filter(u =>
      u.username.toLowerCase().includes(search.toLowerCase()) ||
      u.email.toLowerCase().includes(search.toLowerCase()) ||
      u.display_name.toLowerCase().includes(search.toLowerCase())
    )
    setFilteredUsers(filtered)
  }, [users, search])

  const loadUsers = async () => {
    try {
      const data = await adminApi.getUsers() as User[]
      setUsers(data)
    } catch (err) {
      addToast({
        type: 'error',
        title: '加载失败',
        message: getErrorMessage(err),
      })
    }
  }

  const { loading: creating, execute: createUser } = useApi(
    adminApi.createUser,
    {
      successMessage: '用户创建成功',
      onSuccess: () => {
        setShowCreateModal(false)
        setFormData({ username: '', email: '', display_name: '', password: '', is_admin: false })
        loadUsers()
      }
    }
  )

  const { loading: updating, execute: updateUser } = useApi(
    (id: string, data: Record<string, unknown>) => adminApi.updateUser(id, data),
    {
      successMessage: '用户更新成功',
      onSuccess: () => {
        setEditingUser(null)
        loadUsers()
      }
    }
  )

  const { loading: resetting, execute: resetPassword } = useApi(
    (id: string) => adminApi.resetUserPassword(id),
    {
      onSuccess: () => {
        setResetPasswordSuccess(true)
        loadUsers()
      }
    }
  )

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    await createUser(formData)
  }

  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!editingUser) return
    await updateUser(editingUser.id, {
      display_name: editingUser.display_name,
      email: editingUser.email,
      is_admin: editingUser.is_admin,
      is_active: editingUser.is_active,
    })
  }

  const handleResetPassword = async () => {
    if (!resetPasswordUser) return
    await resetPassword(resetPasswordUser.id)
  }

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('zh-CN')
  }

  const columns = [
    {
      key: 'user',
      title: '用户',
      render: (user: User) => (
        <div className="flex items-center gap-3">
          <Avatar name={user.display_name} size="sm" />
          <div>
            <p className="text-sm text-gray-900 dark:text-white">{user.display_name}</p>
            <p className="text-xs text-gray-500 dark:text-gray-600">@{user.username}</p>
          </div>
        </div>
      ),
    },
    {
      key: 'email',
      title: '邮箱',
      render: (user: User) => (
        <span className="text-sm text-gray-500">{user.email}</span>
      ),
    },
    {
      key: 'role',
      title: '角色',
      render: (user: User) => (
        user.is_admin ? (
          <Badge variant="warning">管理员</Badge>
        ) : (
          <Badge>普通用户</Badge>
        )
      ),
    },
    {
      key: 'status',
      title: '状态',
      render: (user: User) => (
        user.is_active ? (
          <Badge variant="success">活跃</Badge>
        ) : (
          <Badge variant="error">禁用</Badge>
        )
      ),
    },
    {
      key: 'created',
      title: '创建时间',
      render: (user: User) => (
        <span className="text-sm text-gray-500 dark:text-gray-600">{formatDate(user.created_at)}</span>
      ),
    },
    {
      key: 'actions',
      title: '',
      width: '80px',
      render: (user: User) => (
        <div className="flex items-center gap-1">
          <button
            onClick={() => setResetPasswordUser(user)}
            className="p-1.5 text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-white/[0.04] rounded transition-colors"
            title="重置密码"
          >
            <Key className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => setEditingUser(user)}
            className="p-1.5 text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-white/[0.04] rounded transition-colors"
            title="编辑"
          >
            <MoreHorizontal className="w-3.5 h-3.5" />
          </button>
        </div>
      ),
    },
  ]

  return (
    <div className="space-y-5" style={{ animation: 'slideUp 0.3s ease-out' }}>
      <style>{fadeIn}{slideUp}</style>
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-xl font-bold text-gray-900 dark:text-white">用户管理</h1>
          <p className="text-sm text-gray-500 mt-0.5">管理系统用户和权限</p>
        </div>
        <Button 
          onClick={() => setShowCreateModal(true)}
          size="sm"
        >
          <Plus className="w-4 h-4" />
          添加用户
        </Button>
      </div>

      {/* Search */}
      <Card padding="sm">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-600" />
          <input
            type="text"
            placeholder="搜索用户..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full bg-transparent pl-9 pr-4 py-2 text-sm text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-gray-700 focus:outline-none"
          />
        </div>
      </Card>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-3">
        <Card className="text-center py-4">
          <p className="text-xl font-medium text-gray-900 dark:text-white">{users.length}</p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">总用户</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-xl font-medium text-emerald-600 dark:text-emerald-400">
            {users.filter(u => u.is_active).length}
          </p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">活跃用户</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-xl font-medium text-gray-600 dark:text-gray-300">
            {users.filter(u => u.is_admin).length}
          </p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">管理员</p>
        </Card>
      </div>

      {/* Table */}
      <Card padding="none">
        <Table
          data={filteredUsers}
          columns={columns}
          keyExtractor={(user) => user.id}
          emptyState={
            <EmptyState
              icon={<Users className="w-6 h-6" />}
              title="暂无用户"
              description="还没有任何用户，点击上方按钮添加"
            />
          }
        />
      </Card>

      {/* Create Modal */}
      <Modal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        title="添加用户"
        description="创建新用户账户"
        footer={
          <>
            <Button variant="ghost" onClick={() => setShowCreateModal(false)}>
              取消
            </Button>
            <Button 
              onClick={handleCreate} 
              loading={creating}
            >
              创建
            </Button>
          </>
        }
      >
        <form className="space-y-4">
          <Input
            label="用户名"
            value={formData.username}
            onChange={(e) => setFormData({ ...formData, username: e.target.value })}
            placeholder="username"
            required
          />
          <Input
            label="邮箱"
            type="email"
            value={formData.email}
            onChange={(e) => setFormData({ ...formData, email: e.target.value })}
            placeholder="user@example.com"
            required
          />
          <Input
            label="显示名称"
            value={formData.display_name}
            onChange={(e) => setFormData({ ...formData, display_name: e.target.value })}
            placeholder="用户显示名称"
            required
          />
          <Input
            label="密码"
            type="password"
            value={formData.password}
            onChange={(e) => setFormData({ ...formData, password: e.target.value })}
            placeholder="至少8位字符"
            required
          />
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={formData.is_admin}
              onChange={(e) => setFormData({ ...formData, is_admin: e.target.checked })}
              className="w-4 h-4 rounded border-gray-300 dark:border-white/[0.12] bg-white dark:bg-white/[0.04] text-black dark:text-white focus:ring-black/20 dark:focus:ring-white/20"
            />
            <span className="text-sm text-gray-600 dark:text-gray-400">设为管理员</span>
          </label>
        </form>
      </Modal>

      {/* Edit Modal */}
      {editingUser && (
        <Modal
          isOpen={!!editingUser}
          onClose={() => setEditingUser(null)}
          title="编辑用户"
          footer={
            <>
              <Button variant="ghost" onClick={() => setEditingUser(null)}>
                取消
              </Button>
              <Button 
                onClick={handleUpdate} 
                loading={updating}
              >
                保存
              </Button>
            </>
          }
        >
          <form className="space-y-4">
            <Input
              label="显示名称"
              value={editingUser.display_name}
              onChange={(e) => setEditingUser({ ...editingUser, display_name: e.target.value })}
            />
            <Input
              label="邮箱"
              type="email"
              value={editingUser.email}
              onChange={(e) => setEditingUser({ ...editingUser, email: e.target.value })}
            />
            <div className="space-y-3">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={editingUser.is_admin}
                  onChange={(e) => setEditingUser({ ...editingUser, is_admin: e.target.checked })}
                  className="w-4 h-4 rounded border-gray-300 dark:border-white/[0.12] bg-white dark:bg-white/[0.04] text-black dark:text-white focus:ring-black/20 dark:focus:ring-white/20"
                />
                <span className="text-sm text-gray-600 dark:text-gray-400">管理员权限</span>
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={editingUser.is_active}
                  onChange={(e) => setEditingUser({ ...editingUser, is_active: e.target.checked })}
                  className="w-4 h-4 rounded border-gray-300 dark:border-white/[0.12] bg-white dark:bg-white/[0.04] text-black dark:text-white focus:ring-black/20 dark:focus:ring-white/20"
                />
                <span className="text-sm text-gray-600 dark:text-gray-400">账户活跃</span>
              </label>
            </div>
          </form>
        </Modal>
      )}

      {/* Reset Password Modal */}
      {resetPasswordUser && (
        <Modal
          isOpen={!!resetPasswordUser}
          onClose={() => {
            setResetPasswordUser(null)
            setResetPasswordSuccess(false)
          }}
          title="重置密码"
          description={`为 ${resetPasswordUser.display_name} 重置密码`}
          footer={
            resetPasswordSuccess ? (
              <Button onClick={() => {
                setResetPasswordUser(null)
                setResetPasswordSuccess(false)
              }}>
                完成
              </Button>
            ) : (
              <>
                <Button variant="ghost" onClick={() => setResetPasswordUser(null)}>
                  取消
                </Button>
                <Button 
                  onClick={handleResetPassword}
                  loading={resetting}
                  variant="danger"
                >
                  重置
                </Button>
              </>
            )
          }
        >
          {resetPasswordSuccess ? (
            <div className="space-y-4">
              <div className="p-4 bg-emerald-50 dark:bg-emerald-500/[0.08] border border-emerald-200 dark:border-emerald-500/[0.16] rounded-lg">
                <p className="text-sm text-emerald-600 dark:text-emerald-400 font-medium">密码已成功重置</p>
              </div>
              <p className="text-sm text-gray-500 dark:text-gray-600">
                用户需要使用新密码登录。如用户忘记密码，可通过管理员再次重置。
              </p>
            </div>
          ) : (
            <p className="text-gray-600 dark:text-gray-400">
              确定要重置此用户的密码吗？系统将自动生成一个新密码。
            </p>
          )}
        </Modal>
      )}
    </div>
  )
}
