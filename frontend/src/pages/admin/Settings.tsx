import { useState, useEffect } from 'react'
import { 
  Palette, 
  Type,
  RefreshCw,
  Shield,
  Save
} from 'lucide-react'
import { adminApi } from '#src/api/admin'
import { useUIConfigStore, type LoginLayout } from '#src/stores/theme'
import { useApi } from '#src/hooks'
import { 
  Card, 
  CardHeader,
  Button, 
  Input,
  useToast
} from '#src/components/ui'

// Animation keyframes
const fadeIn = `@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }`
const slideUp = `@keyframes slideUp { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }`

const layoutOptions: { value: LoginLayout; label: string; description: string }[] = [
  { value: 'split-left', label: '左侧品牌', description: '品牌展示在左' },
  { value: 'split-right', label: '右侧品牌', description: '品牌展示在右' },
  { value: 'centered', label: '居中', description: '简洁居中布局' },
  { value: 'fullscreen', label: '全屏', description: '沉浸式深色背景' },
]

export function AdminSettings() {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { brandName, setBrandName, loginLayout, setLoginLayout } = useUIConfigStore()
  const { addToast } = useToast()
  void addToast // suppress unused warning
  const [activeTab, setActiveTab] = useState<'branding' | 'appearance' | 'security'>('branding')
  
  // Settings state
  const [settings, setSettings] = useState({
    brand_name: brandName,
    logo_url: '',
    login_background_url: '',
    login_layout: loginLayout,
    session_timeout: 24,
    max_login_attempts: 5,
  })

  // Load settings
  useEffect(() => {
    loadSettings()
  }, [])

  const loadSettings = async () => {
    try {
      const data = await adminApi.getSettings() as typeof settings
      setSettings(prev => ({ ...prev, ...data }))
    } catch (err) {
      // Use defaults if backend not ready
    }
  }

  const { loading: saving, execute: saveSettings } = useApi(
    adminApi.updateSettings,
    {
      successMessage: '设置已保存',
      onSuccess: (_data) => {
        // Update local store
        setBrandName(settings.brand_name)
        setLoginLayout(settings.login_layout)
      }
    }
  )

  const { loading: rotating, execute: rotateKey } = useApi(
    adminApi.rotateKey,
    {
      successMessage: '密钥轮换成功',
    }
  )

  const handleSave = async () => {
    await saveSettings(settings)
  }

  const tabs = [
    { id: 'branding', label: '品牌', icon: Type },
    { id: 'appearance', label: '外观', icon: Palette },
    { id: 'security', label: '安全', icon: Shield },
  ]

  return (
    <div className="space-y-5" style={{ animation: 'slideUp 0.3s ease-out' }}>
      <style>{fadeIn}{slideUp}</style>
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-xl font-bold text-gray-900 dark:text-white">系统设置</h1>
          <p className="text-sm text-gray-500 mt-0.5">配置系统品牌和外观</p>
        </div>
        <Button 
          onClick={handleSave}
          loading={saving}
          size="sm"
        >
          <Save className="w-4 h-4" />
          保存
        </Button>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200 dark:border-white/[0.06]">
        <nav className="flex gap-1">
          {tabs.map((tab) => {
            const Icon = tab.icon
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as typeof activeTab)}
                className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
                  activeTab === tab.id
                    ? 'border-black text-black dark:border-white dark:text-white'
                    : 'border-transparent text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'
                }`}
              >
                <Icon className="w-4 h-4" />
                {tab.label}
              </button>
            )
          })}
        </nav>
      </div>

      {/* Branding Tab */}
      {activeTab === 'branding' && (
        <div className="space-y-4 max-w-lg">
          <Card>
            <CardHeader 
              title="品牌信息" 
              subtitle="配置系统的品牌标识"
            />
            <div className="space-y-4">
              <Input
                label="品牌名称"
                value={settings.brand_name}
                onChange={(e) => setSettings({ ...settings, brand_name: e.target.value })}
                placeholder="UNOIDC"
                helper="显示在登录页和导航栏的品牌名称"
              />
              <Input
                label="Logo URL"
                type="url"
                value={settings.logo_url}
                onChange={(e) => setSettings({ ...settings, logo_url: e.target.value })}
                placeholder="https://example.com/logo.png"
                helper="建议使用 200x200 像素的 PNG 图片"
              />
            </div>
          </Card>
        </div>
      )}

      {/* Appearance Tab */}
      {activeTab === 'appearance' && (
        <div className="space-y-4 max-w-lg">
          <Card>
            <CardHeader 
              title="登录页布局" 
              subtitle="选择登录页的显示样式"
            />
            <div className="grid grid-cols-2 gap-2">
              {layoutOptions.map((option) => (
                <button
                  key={option.value}
                  onClick={() => setSettings({ ...settings, login_layout: option.value })}
                  className={`flex flex-col gap-1 p-3 rounded-lg border text-left transition-all ${
                    settings.login_layout === option.value
                      ? 'border-black bg-gray-100 dark:border-white dark:bg-white/[0.04]'
                      : 'border-gray-200 dark:border-white/[0.06] hover:border-gray-300 dark:hover:border-white/[0.12]'
                  }`}
                >
                  <span className={`text-sm font-medium ${
                    settings.login_layout === option.value ? 'text-gray-900 dark:text-white' : 'text-gray-600 dark:text-gray-400'
                  }`}>
                    {option.label}
                  </span>
                  <span className="text-xs text-gray-500 dark:text-gray-600">{option.description}</span>
                </button>
              ))}
            </div>
          </Card>

          <Card>
            <CardHeader 
              title="登录页背景" 
              subtitle="配置登录页的背景图片"
            />
            <Input
              label="背景图片 URL"
              type="url"
              value={settings.login_background_url}
              onChange={(e) => setSettings({ ...settings, login_background_url: e.target.value })}
              placeholder="https://example.com/background.jpg"
              helper="建议使用 1920x1080 或更大尺寸的图片"
            />
          </Card>
        </div>
      )}

      {/* Security Tab */}
      {activeTab === 'security' && (
        <div className="space-y-4 max-w-lg">
          <Card>
            <CardHeader 
              title="会话设置" 
              subtitle="配置用户会话安全参数"
            />
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">
                  会话超时（小时）
                </label>
                <input
                  type="number"
                  min={1}
                  max={168}
                  value={settings.session_timeout}
                  onChange={(e) => setSettings({ ...settings, session_timeout: parseInt(e.target.value) })}
                  className="w-full max-w-xs bg-gray-50 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-black/10 dark:focus:ring-white/20"
                />
                <p className="text-xs text-gray-500 dark:text-gray-600 mt-1.5">
                  用户登录后多少小时自动过期
                </p>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">
                  最大登录尝试次数
                </label>
                <input
                  type="number"
                  min={1}
                  max={10}
                  value={settings.max_login_attempts}
                  onChange={(e) => setSettings({ ...settings, max_login_attempts: parseInt(e.target.value) })}
                  className="w-full max-w-xs bg-gray-50 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-black/10 dark:focus:ring-white/20"
                />
                <p className="text-xs text-gray-500 dark:text-gray-600 mt-1.5">
                  超过此次数后账户将被临时锁定
                </p>
              </div>
            </div>
          </Card>

          <Card>
            <CardHeader 
              title="密钥管理" 
              subtitle="管理签名密钥"
            />
            <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-white/[0.02] rounded-lg border border-gray-200 dark:border-white/[0.06]">
              <div>
                <p className="text-sm text-gray-900 dark:text-white">轮换签名密钥</p>
                <p className="text-xs text-gray-500 dark:text-gray-600 mt-0.5">
                  生成新的 JWT 签名密钥
                </p>
              </div>
              <Button
                variant="secondary"
                size="sm"
                onClick={rotateKey}
                loading={rotating}
              >
                <RefreshCw className="w-4 h-4" />
                轮换
              </Button>
            </div>
          </Card>
        </div>
      )}
    </div>
  )
}
