import { useState } from 'react'
import { useUIConfigStore } from '#src/stores/theme'
import { LoginLayoutSelector } from '#src/components/LoginLayout'
import { ThemeSelector } from '#src/components/ThemeToggle'

export function AdminSettings() {
  const { loginLayout, setLoginLayout, brandName, setBrandName } = useUIConfigStore()
  const [activeTab, setActiveTab] = useState<'branding' | 'appearance'>('branding')

  return (
    <div>
      <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">系统设置</h1>

      {/* Tabs */}
      <div className="border-b border-gray-200 dark:border-gray-800 mb-6">
        <nav className="flex gap-6">
          <button
            onClick={() => setActiveTab('branding')}
            className={`pb-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'branding'
                ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
            }`}
          >
            品牌设置
          </button>
          <button
            onClick={() => setActiveTab('appearance')}
            className={`pb-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'appearance'
                ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
            }`}
          >
            外观设置
          </button>
        </nav>
      </div>

      {activeTab === 'branding' && (
        <div className="space-y-6 max-w-2xl">
          <div className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-6">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">品牌信息</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  品牌名称
                </label>
                <input
                  type="text"
                  value={brandName}
                  onChange={(e) => setBrandName(e.target.value)}
                  className="w-full px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Logo URL
                </label>
                <input
                  type="url"
                  placeholder="https://example.com/logo.png"
                  className="w-full px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500"
                />
                <p className="mt-1 text-xs text-gray-500">留空使用默认图标</p>
              </div>
            </div>
          </div>
        </div>
      )}

      {activeTab === 'appearance' && (
        <div className="space-y-6 max-w-2xl">
          {/* Theme */}
          <div className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-6">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">主题模式</h3>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">
              选择系统的主题模式。自动模式会根据系统设置自动切换深浅色。
            </p>
            <ThemeSelector />
          </div>

          {/* Login Layout */}
          <div className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-6">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">登录页布局</h3>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">
              选择登录页的布局样式。更改会立即生效。
            </p>
            <LoginLayoutSelector value={loginLayout} onChange={setLoginLayout} />
          </div>

          {/* Background Image */}
          <div className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-6">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">登录页背景</h3>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                背景图片 URL
              </label>
              <input
                type="url"
                placeholder="https://example.com/background.jpg"
                className="w-full px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500"
              />
              <p className="mt-1 text-xs text-gray-500">建议使用 1920x1080 或更大尺寸的图片</p>
            </div>
          </div>

          {/* Preview */}
          <div className="bg-white dark:bg-gray-900 rounded-xl shadow-sm border border-gray-200 dark:border-gray-800 p-6">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">预览</h3>
            <div className="aspect-video rounded-lg bg-gray-100 dark:bg-gray-800 overflow-hidden border border-gray-200 dark:border-gray-700">
              {loginLayout === 'split-left' && (
                <div className="flex h-full">
                  <div className="w-1/2 bg-gradient-to-br from-blue-600 to-purple-700" />
                  <div className="w-1/2 bg-white dark:bg-gray-900 p-8">
                    <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 mb-4" />
                    <div className="h-4 w-24 bg-gray-200 dark:bg-gray-700 rounded mb-2" />
                    <div className="h-3 w-32 bg-gray-100 dark:bg-gray-800 rounded" />
                  </div>
                </div>
              )}
              {loginLayout === 'split-right' && (
                <div className="flex h-full">
                  <div className="w-1/2 bg-white dark:bg-gray-900 p-8">
                    <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 mb-4" />
                    <div className="h-4 w-24 bg-gray-200 dark:bg-gray-700 rounded mb-2" />
                    <div className="h-3 w-32 bg-gray-100 dark:bg-gray-800 rounded" />
                  </div>
                  <div className="w-1/2 bg-gradient-to-br from-blue-600 to-purple-700" />
                </div>
              )}
              {loginLayout === 'centered' && (
                <div className="flex h-full items-center justify-center bg-gray-50 dark:bg-gray-950 p-8">
                  <div className="w-48 bg-white dark:bg-gray-900 rounded-xl shadow-lg p-6">
                    <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 mx-auto mb-3" />
                    <div className="h-3 w-20 bg-gray-200 dark:bg-gray-700 rounded mx-auto mb-2" />
                    <div className="h-2 w-28 bg-gray-100 dark:bg-gray-800 rounded mx-auto" />
                  </div>
                </div>
              )}
              {loginLayout === 'fullscreen' && (
                <div className="flex h-full items-center justify-center bg-gradient-to-br from-blue-900 via-purple-900 to-gray-900 p-8">
                  <div className="w-48 bg-white/95 dark:bg-gray-900/95 rounded-xl p-6 backdrop-blur">
                    <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 mx-auto mb-3" />
                    <div className="h-3 w-20 bg-gray-200 dark:bg-gray-700 rounded mx-auto mb-2" />
                    <div className="h-2 w-28 bg-gray-100 dark:bg-gray-800 rounded mx-auto" />
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
