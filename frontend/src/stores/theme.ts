import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type ThemeMode = 'light' | 'dark' | 'auto'
export type LoginLayout = 'split-left' | 'split-right' | 'centered' | 'fullscreen'

interface ThemeState {
  // Theme
  mode: ThemeMode
  resolvedMode: 'light' | 'dark' // Computed from mode + system preference
  setMode: (mode: ThemeMode) => void
  toggleTheme: () => void
}

interface UIConfigState {
  // Login page layout
  loginLayout: LoginLayout
  setLoginLayout: (layout: LoginLayout) => void

  // Branding
  brandName: string
  setBrandName: (name: string) => void
  logoUrl?: string
  setLogoUrl: (url: string | undefined) => void
  loginBackgroundUrl?: string
  setLoginBackgroundUrl: (url: string | undefined) => void
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set, get) => ({
      mode: 'auto',
      resolvedMode: 'light',

      setMode: (mode) => {
        const resolved = resolveThemeMode(mode)
        set({ mode, resolvedMode: resolved })
        applyTheme(resolved)
      },

      toggleTheme: () => {
        const current = get().mode
        let next: ThemeMode
        if (current === 'light') next = 'dark'
        else if (current === 'dark') next = 'auto'
        else next = 'light'
        get().setMode(next)
      },
    }),
    {
      name: 'oidc-theme',
      partialize: (state) => ({ mode: state.mode }),
    }
  )
)

export const useUIConfigStore = create<UIConfigState>()(
  persist(
    (set) => ({
      loginLayout: 'split-left',
      setLoginLayout: (layout) => set({ loginLayout: layout }),

      brandName: 'UNOIDC',
      setBrandName: (name) => set({ brandName: name }),

      logoUrl: undefined,
      setLogoUrl: (url) => set({ logoUrl: url }),

      loginBackgroundUrl: undefined,
      setLoginBackgroundUrl: (url) => set({ loginBackgroundUrl: url }),
    }),
    {
      name: 'oidc-ui-config',
    }
  )
)

// Helper functions
function resolveThemeMode(mode: ThemeMode): 'light' | 'dark' {
  if (mode === 'auto') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches
      ? 'dark'
      : 'light'
  }
  return mode
}

function applyTheme(mode: 'light' | 'dark') {
  const root = document.documentElement
  if (mode === 'dark') {
    root.classList.add('dark')
  } else {
    root.classList.remove('dark')
  }
}

// Initialize theme from store
export function initTheme() {
  // Get current state from store (persist middleware will have restored it)
  const state = useThemeStore.getState()
  const resolved = resolveThemeMode(state.mode)

  // Apply resolved theme
  applyTheme(resolved)

  // Update resolved mode in store if needed
  if (state.resolvedMode !== resolved) {
    useThemeStore.setState({ resolvedMode: resolved })
  }

  // Listen for system theme changes when in auto mode
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  mediaQuery.addEventListener('change', (e) => {
    const currentState = useThemeStore.getState()
    if (currentState.mode === 'auto') {
      const newResolved = e.matches ? 'dark' : 'light'
      useThemeStore.setState({ resolvedMode: newResolved })
      applyTheme(newResolved)
    }
  })
}
