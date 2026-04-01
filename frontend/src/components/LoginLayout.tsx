import type { ReactNode } from 'react'
import { useState, useEffect } from 'react'
import { Shield, Fingerprint, Lock, KeyRound } from 'lucide-react'
import { authApi } from '#src/api/auth'
import type { LoginLayout } from '#src/stores/theme'

interface LoginLayoutSelectorProps {
  value: LoginLayout
  onChange: (layout: LoginLayout) => void
}

const layoutOptions: { value: LoginLayout; label: string; description: string }[] = [
  { value: 'split-left', label: '左侧品牌', description: '品牌展示在左' },
  { value: 'split-right', label: '右侧品牌', description: '品牌展示在右' },
  { value: 'centered', label: '居中', description: '简洁居中布局' },
  { value: 'fullscreen', label: '全屏', description: '沉浸式深色背景' },
]

export function LoginLayoutSelector({ value, onChange }: LoginLayoutSelectorProps) {
  return (
    <div className="grid grid-cols-2 gap-3">
      {layoutOptions.map((option) => (
        <button
          key={option.value}
          type="button"
          onClick={() => onChange(option.value)}
          className={`
            flex flex-col gap-1 p-3 rounded-lg border text-left transition-all duration-200
            ${value === option.value
              ? 'border-white/20 bg-white/[0.04]'
              : 'border-white/[0.06] hover:border-white/[0.12] hover:bg-white/[0.02]'
            }
          `}
        >
          <span className={`text-sm font-medium ${value === option.value ? 'text-white' : 'text-gray-300'}`}>
            {option.label}
          </span>
          <span className="text-xs text-gray-600">
            {option.description}
          </span>
        </button>
      ))}
    </div>
  )
}

// Abstract geometric pattern for split layouts
function GeometricPattern() {
  return (
    <div className="absolute inset-0 overflow-hidden opacity-[0.08]">
      <svg className="absolute w-full h-full" viewBox="0 0 400 400" fill="none">
        <defs>
          <linearGradient id="geo-grad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="white" stopOpacity="0.5" />
            <stop offset="100%" stopColor="white" stopOpacity="0.2" />
          </linearGradient>
        </defs>
        {/* Grid lines */}
        <line x1="0" y1="100" x2="400" y2="100" stroke="url(#geo-grad)" strokeWidth="0.5" />
        <line x1="0" y1="200" x2="400" y2="200" stroke="url(#geo-grad)" strokeWidth="0.5" />
        <line x1="0" y1="300" x2="400" y2="300" stroke="url(#geo-grad)" strokeWidth="0.5" />
        <line x1="100" y1="0" x2="100" y2="400" stroke="url(#geo-grad)" strokeWidth="0.5" />
        <line x1="200" y1="0" x2="200" y2="400" stroke="url(#geo-grad)" strokeWidth="0.5" />
        <line x1="300" y1="0" x2="300" y2="400" stroke="url(#geo-grad)" strokeWidth="0.5" />
        {/* Circles */}
        <circle cx="200" cy="200" r="100" stroke="url(#geo-grad)" strokeWidth="0.5" fill="none" />
        <circle cx="200" cy="200" r="150" stroke="url(#geo-grad)" strokeWidth="0.5" fill="none" />
      </svg>
    </div>
  )
}

// Brand content for sidebar
function BrandContent({ brandName }: { brandName: string }) {
  return (
    <div className="relative z-10 flex flex-col justify-center h-full p-12 text-white">
      <div className="mb-8">
        <div className="inline-flex items-center justify-center w-14 h-14 mb-6 rounded-xl bg-white/[0.06] backdrop-blur-sm ring-1 ring-white/[0.08]">
          <Shield className="w-7 h-7 text-white" />
        </div>
        <h1 className="text-3xl font-semibold tracking-tight mb-3">{brandName}</h1>
        <p className="text-base text-white/50 max-w-sm leading-relaxed">
          安全、可靠的统一身份认证解决方案
        </p>
      </div>

      <div className="space-y-3 mt-8">
        {[
          { icon: Fingerprint, text: '多因素身份验证' },
          { icon: Lock, text: '企业级安全防护' },
          { icon: KeyRound, text: 'OIDC 标准协议' },
        ].map(({ icon: Icon, text }, i) => (
          <div key={i} className="flex items-center gap-3 text-white/40">
            <Icon className="w-4 h-4" />
            <span className="text-sm">{text}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

interface LoginPageWrapperProps {
  children: ReactNode
}

// 默认配置
const defaultConfig = {
  brandName: 'UNOIDC',
  loginLayout: 'split-left' as LoginLayout,
  loginBackgroundUrl: '',
}

export function LoginPageWrapper({ children }: LoginPageWrapperProps) {
  // 从后端获取公共配置
  const [config, setConfig] = useState(defaultConfig)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    authApi.getPublicConfig()
      .then((data) => {
        setConfig({
          brandName: data.brand_name,
          loginLayout: data.login_layout,
          loginBackgroundUrl: data.login_background_url,
        })
      })
      .catch(() => {
        // 使用默认配置
      })
      .finally(() => {
        setLoading(false)
      })
  }, [])

  const { brandName, loginLayout, loginBackgroundUrl } = config

  if (loading) {
    // 加载时显示简洁的加载状态
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-black">
        <div className="animate-pulse">
          <div className="w-8 h-8 rounded-md bg-gray-300 dark:bg-gray-700" />
        </div>
      </div>
    )
  }

  const formSection = (
    <div className="flex-1 flex items-center justify-center p-6 lg:p-12 bg-gray-50 dark:bg-black">
      <div className="w-full max-w-sm">
        {children}
      </div>
    </div>
  )

  const imageSection = (
    <div className="relative hidden lg:flex lg:w-[55%] bg-gray-900 dark:bg-neutral-950 overflow-hidden">
      <GeometricPattern />
      {loginBackgroundUrl ? (
        <>
          <img
            src={loginBackgroundUrl}
            alt=""
            className="absolute inset-0 w-full h-full object-cover opacity-30"
          />
          <div className="absolute inset-0 bg-gradient-to-br from-black/60 to-neutral-950/90" />
        </>
      ) : null}
      <BrandContent brandName={brandName} />
    </div>
  )

  switch (loginLayout) {
    case 'split-left':
      return (
        <div className="min-h-screen flex">
          {imageSection}
          {formSection}
        </div>
      )

    case 'split-right':
      return (
        <div className="min-h-screen flex">
          {formSection}
          {imageSection}
        </div>
      )

    case 'centered':
      return (
        <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-black p-4">
          <div className="w-full max-w-[380px]">
            <div className="bg-white dark:bg-white/[0.02] border border-gray-200 dark:border-white/[0.06] rounded-xl p-8 shadow-sm dark:shadow-none">
              {children}
            </div>
          </div>
        </div>
      )

    case 'fullscreen':
      return (
        <div className="min-h-screen relative flex items-center justify-center p-4 overflow-hidden bg-gray-50 dark:bg-black">
          {/* Subtle grid background */}
          <div className="absolute inset-0 opacity-[0.03] dark:opacity-[0.03] opacity-0">
            <div className="absolute inset-0" style={{
              backgroundImage: `linear-gradient(to right, white 1px, transparent 1px), linear-gradient(to bottom, white 1px, transparent 1px)`,
              backgroundSize: '60px 60px'
            }} />
          </div>

          {/* Card */}
          <div className="relative w-full max-w-[380px]">
            <div className="bg-white dark:bg-white/[0.02] backdrop-blur-xl rounded-xl border border-gray-200 dark:border-white/[0.06] p-8 shadow-lg dark:shadow-none">
              {children}
            </div>
          </div>
        </div>
      )

    default:
      return formSection
  }
}
