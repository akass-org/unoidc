import { useState, useEffect } from 'react'
import { Camera } from 'lucide-react'
import { useSessionStore } from '#src/stores/session'
import { meApi } from '#src/api/me'
import { useApi } from '#src/hooks'
import { 
  Card, 
  CardHeader, 
  Input, 
  Button, 
  Avatar,
  useToast 
} from '#src/components/ui'
import { getErrorMessage } from '#src/api/client'

export function ProfilePage() {
  const { user, setUser } = useSessionStore()
  const { addToast } = useToast()
  
  // Profile form state
  const [displayName, setDisplayName] = useState('')
  const [email, setEmail] = useState('')
  
  // Password form state
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  
  // Update profile API
  const { loading: updatingProfile, execute: updateProfile } = useApi(
    meApi.updateProfile,
    {
      successMessage: '个人资料已更新',
      onSuccess: (data) => {
        setUser(data as { id: string; username: string; email: string; display_name: string; picture?: string; is_admin: boolean })
      }
    }
  )
  
  // Change password API
  const { loading: changingPassword, execute: changePassword } = useApi(
    meApi.changePassword,
    {
      successMessage: '密码已修改',
      onSuccess: () => {
        setCurrentPassword('')
        setNewPassword('')
        setConfirmPassword('')
      }
    }
  )

  // Load user data
  useEffect(() => {
    if (user) {
      setDisplayName(user.display_name || '')
      setEmail(user.email || '')
    }
  }, [user])

  const handleProfileSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    await updateProfile({ display_name: displayName, email })
  }

  const handlePasswordSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (newPassword !== confirmPassword) {
      addToast({
        type: 'error',
        title: '两次输入的密码不一致',
      })
      return
    }
    
    if (newPassword.length < 8) {
      addToast({
        type: 'error',
        title: '新密码至少需要8个字符',
      })
      return
    }
    
    await changePassword({ current_password: currentPassword, new_password: newPassword })
  }

  const handleAvatarUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    // 前端预检查：限制 1MB，避免大图上传超时
    if (file.size > 1024 * 1024) {
      addToast({
        type: 'error',
        title: '图片过大',
        message: `文件大小 ${(file.size / 1024).toFixed(0)}KB，请选择小于 1MB 的图片`,
      })
      // 重置 input，允许重复选择同一文件
      e.target.value = ''
      return
    }

    try {
      const result = await meApi.uploadAvatar(file)
      setUser(result as { id: string; username: string; email: string; display_name: string; picture?: string; is_admin: boolean })
      addToast({
        type: 'success',
        title: '头像已更新',
      })
    } catch (err) {
      addToast({
        type: 'error',
        title: '头像上传失败',
        message: getErrorMessage(err),
      })
    }
  }

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-lg font-medium text-gray-900 dark:text-white">个人资料</h1>
        <p className="text-sm text-gray-500 mt-0.5">管理您的账户信息和安全设置</p>
      </div>

      {/* Avatar Section */}
      <Card>
        <div className="flex flex-col sm:flex-row items-center gap-5">
          <div className="relative">
            <Avatar 
              name={user?.display_name || user?.username || '?'} 
              src={user?.picture}
              size="xl" 
            />
            <label className="absolute -bottom-1 -right-1 w-7 h-7 bg-white hover:bg-gray-200 rounded-full flex items-center justify-center cursor-pointer transition-colors shadow-lg">
              <Camera className="w-3.5 h-3.5 text-black" />
              <input
                type="file"
                accept="image/*"
                onChange={handleAvatarUpload}
                className="hidden"
              />
            </label>
          </div>
          <div className="text-center sm:text-left">
            <h2 className="text-base font-medium text-gray-900 dark:text-white">
              {user?.display_name || user?.username}
            </h2>
            <p className="text-sm text-gray-500">@{user?.username}</p>
            <p className="text-xs text-gray-500 dark:text-gray-600 mt-1">
              {user?.is_admin ? '管理员' : '普通用户'}
            </p>
          </div>
        </div>
      </Card>

      {/* Profile Form */}
      <Card>
        <CardHeader 
          title="基本信息" 
          subtitle="更新您的显示名称和邮箱地址"
        />
        <form onSubmit={handleProfileSubmit} className="space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <Input
              label="用户名"
              value={user?.username || ''}
              disabled
              helper="用户名不可修改"
            />
            <Input
              label="显示名称"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder="您的显示名称"
            />
          </div>
          <Input
            label="邮箱地址"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="your@email.com"
          />
          <div className="flex justify-end pt-1">
            <Button 
              type="submit" 
              loading={updatingProfile}
              size="sm"
            >
              保存更改
            </Button>
          </div>
        </form>
      </Card>

      {/* Password Form */}
      <Card>
        <CardHeader 
          title="修改密码" 
          subtitle="定期更改密码可以提高账户安全性"
        />
        <form onSubmit={handlePasswordSubmit} className="space-y-4 max-w-md">
          <Input
            label="当前密码"
            type="password"
            value={currentPassword}
            onChange={(e) => setCurrentPassword(e.target.value)}
            placeholder="输入当前密码"
            required
          />
          <Input
            label="新密码"
            type="password"
            value={newPassword}
            onChange={(e) => setNewPassword(e.target.value)}
            placeholder="至少8位字符"
            required
          />
          <Input
            label="确认新密码"
            type="password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            placeholder="再次输入新密码"
            required
          />
          <div className="flex justify-end pt-1">
            <Button 
              type="submit" 
              loading={changingPassword}
              variant="secondary"
              size="sm"
            >
              更新密码
            </Button>
          </div>
        </form>
      </Card>
    </div>
  )
}
