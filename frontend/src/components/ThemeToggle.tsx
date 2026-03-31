import { Moon, Sun, Monitor } from 'lucide-react'
import { useThemeStore, type ThemeMode } from '#src/stores/theme'

const themeConfig: Record<ThemeMode, { icon: React.ReactNode; label: string }> = {
  light: { icon: <Sun className="w-4 h-4" />, label: '浅色' },
  dark: { icon: <Moon className="w-4 h-4" />, label: '深色' },
  auto: { icon: <Monitor className="w-4 h-4" />, label: '自动' },
}

export function ThemeToggle({ showLabel = false }: { showLabel?: boolean }) {
  const { mode, resolvedMode, setMode } = useThemeStore()

  return (
    <button
      type="button"
      onClick={() => setMode(mode === 'light' ? 'dark' : mode === 'dark' ? 'auto' : 'light')}
      className="
        group inline-flex items-center gap-2
        px-3 py-2 rounded-lg
        text-slate-600 dark:text-slate-400
        hover:text-slate-900 dark:hover:text-slate-200
        hover:bg-slate-100 dark:hover:bg-slate-800
        transition-all duration-150
        focus:outline-none focus:ring-2 focus:ring-slate-500/20
      "
      title={`当前: ${themeConfig[mode].label} (${resolvedMode === 'dark' ? '深色' : '浅色'}模式)`}
    >
      <span className="transition-transform duration-200 group-hover:scale-110">
        {resolvedMode === 'dark' ? <Moon className="w-4 h-4" /> : <Sun className="w-4 h-4" />}
      </span>
      {showLabel && (
        <span className="text-sm font-medium">{themeConfig[mode].label}</span>
      )}
    </button>
  )
}

export function ThemeSelector() {
  const { mode, setMode } = useThemeStore()

  const options: ThemeMode[] = ['light', 'dark', 'auto']

  return (
    <div className="inline-flex p-1 bg-slate-100 dark:bg-slate-800 rounded-lg">
      {options.map((option) => (
        <button
          key={option}
          type="button"
          onClick={() => setMode(option)}
          className={`
            flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium
            transition-all duration-150
            ${mode === option
              ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white shadow-sm'
              : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
            }
          `}
        >
          {themeConfig[option].icon}
          {themeConfig[option].label}
        </button>
      ))}
    </div>
  )
}
