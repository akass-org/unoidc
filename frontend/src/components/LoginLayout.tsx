import type { ReactNode } from 'react'
import { Shield, Fingerprint, Lock, KeyRound } from 'lucide-react'
import { useUIConfigStore, type LoginLayout } from '#src/stores/theme'

interface LoginLayoutSelectorProps {
  value: LoginLayout
  onChange: (layout: LoginLayout) => void
}

const layoutOptions: { value: LoginLayout; label: string; description: string }[] = [
  { value: 'split-left', label: '左侧图片', description: '品牌展示在左' },
  { value: 'split-right', label: '右侧图片', description: '品牌展示在右' },
  { value: 'centered', label: '居中', description: '简洁居中布局' },
  { value: 'fullscreen', label: '全屏', description: '沉浸式背景' },
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
            flex flex-col gap-1 p-3 rounded-lg border text-left transition-all duration-150
            ${value === option.value
              ? 'border-primary-500 bg-primary-50/50 dark:bg-primary-900/10'
              : 'border-slate-200 dark:border-slate-700 hover:border-slate-300 dark:hover:border-slate-600'
            }
          `}
        >
          <span className={`text-sm font-medium ${value === option.value ? 'text-primary-700 dark:text-primary-400' : 'text-slate-700 dark:text-slate-300'}`}>
            {option.label}
          </span>
          <span className="text-xs text-slate-500 dark:text-slate-400">
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
    <div className="absolute inset-0 overflow-hidden opacity-30">
      <svg className="absolute w-full h-full" viewBox="0 0 400 400" fill="none">
        <defs>
          <linearGradient id="geo-grad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="currentColor" stopOpacity="0.3" />
            <stop offset="100%" stopColor="currentColor" stopOpacity="0.1" />
          </linearGradient>
        </defs>
        {/* Hexagons */}
        <path d="M50 100 L75 57 L125 57 L150 100 L125 143 L75 143 Z" stroke="url(#geo-grad)" strokeWidth="1" fill="none" />
        <path d="M150 100 L175 57 L225 57 L250 100 L225 143 L175 143 Z" stroke="url(#geo-grad)" strokeWidth="1" fill="none" />
        <path d="M100 186 L125 143 L175 143 L200 186 L175 229 L125 229 Z" stroke="url(#geo-grad)" strokeWidth="1" fill="none" />
        <path d="M200 186 L225 143 L275 143 L300 186 L275 229 L225 229 Z" stroke="url(#geo-grad)" strokeWidth="1" fill="none" />
        {/* Circles */}
        <circle cx="320" cy="80" r="40" stroke="url(#geo-grad)" strokeWidth="1" fill="none" />
        <circle cx="320" cy="80" r="60" stroke="url(#geo-grad)" strokeWidth="0.5" fill="none" />
        {/* Lines */}
        <line x1="0" y1="300" x2="400" y2="300" stroke="url(#geo-grad)" strokeWidth="0.5" />
        <line x1="0" y1="320" x2="400" y2="320" stroke="url(#geo-grad)" strokeWidth="0.5" />
        <line x1="0" y1="340" x2="400" y2="340" stroke="url(#geo-grad)" strokeWidth="0.5" />
      </svg>
    </div>
  )
}

// Brand content for sidebar
function BrandContent({ brandName }: { brandName: string }) {
  return (
    <div className="relative z-10 flex flex-col justify-center h-full p-12 text-white">
      <div className="mb-8">
        <div className="inline-flex items-center justify-center w-16 h-16 mb-6 rounded-2xl bg-white/10 backdrop-blur-sm ring-1 ring-white/20">
          <Shield className="w-8 h-8 text-white" />
        </div>
        <h1 className="text-3xl font-bold tracking-tight mb-3">{brandName}</h1>
        <p className="text-lg text-white/70 max-w-sm leading-relaxed">
          安全、可靠的统一身份认证解决方案
        </p>
      </div>

      <div className="space-y-4 mt-8">
        {[
          { icon: Fingerprint, text: '多因素身份验证' },
          { icon: Lock, text: '企业级安全防护' },
          { icon: KeyRound, text: 'OIDC 标准协议' },
        ].map(({ icon: Icon, text }, i) => (
          <div key={i} className="flex items-center gap-3 text-white/60">
            <Icon className="w-5 h-5" />
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

export function LoginPageWrapper({ children }: LoginPageWrapperProps) {
  const { loginLayout, loginBackgroundUrl, brandName } = useUIConfigStore()

  const formSection = (
    <div className="flex-1 flex items-center justify-center p-6 lg:p-12 bg-slate-50 dark:bg-slate-950">
      <div className="w-full max-w-sm">
        {children}
      </div>
    </div>
  )

  const imageSection = (
    <div className="relative hidden lg:flex lg:w-[55%] bg-gradient-to-br from-primary-600 via-primary-700 to-slate-900 overflow-hidden">
      <GeometricPattern />
      {loginBackgroundUrl ? (
        <>
          <img
            src={loginBackgroundUrl}
            alt=""
            className="absolute inset-0 w-full h-full object-cover opacity-40 mix-blend-overlay"
          />
          <div className="absolute inset-0 bg-gradient-to-br from-primary-600/80 to-primary-900/90" />
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
        <div className="min-h-screen flex items-center justify-center bg-slate-100 dark:bg-slate-950 p-4">
          <div className="w-full max-w-[420px]">
            <div className="card p-8">
              {children}
            </div>
          </div>
        </div>
      )

    case 'fullscreen':
      return (
        <div className="min-h-screen relative flex items-center justify-center p-4 overflow-hidden">
          {/* Animated gradient background */}
          <div className="absolute inset-0 bg-slate-950">
            <div className="absolute inset-0 bg-gradient-to-br from-primary-900/40 via-slate-950 to-slate-950" />
            <div className="absolute top-0 left-1/4 w-96 h-96 bg-primary-600/20 rounded-full blur-3xl" />
            <div className="absolute bottom-0 right-1/4 w-96 h-96 bg-violet-600/20 rounded-full blur-3xl" />
            <GeometricPattern />
          </div>

          {/* Glass card */}
          <div className="relative w-full max-w-[420px]">
            <div className="bg-white/95 dark:bg-slate-900/95 backdrop-blur-xl rounded-2xl shadow-elevated border border-white/20 dark:border-slate-800 p-8">
              {children}
            </div>
          </div>
        </div>
      )

    default:
      return formSection
  }
}
