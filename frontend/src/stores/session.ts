import { create } from 'zustand'
import { authApi } from '#src/api/auth'

interface User {
  id: string
  username: string
  email: string
  display_name: string
  picture?: string
  is_admin: boolean
}

interface SessionState {
  user: User | null
  loading: boolean
  setUser: (user: User | null) => void
  setLoading: (loading: boolean) => void
  logout: () => Promise<void>
  checkSession: () => Promise<void>
}

export const useSessionStore = create<SessionState>((set) => ({
  user: null,
  loading: true,
  
  setUser: (user) => set({ user, loading: false }),
  
  setLoading: (loading) => set({ loading }),
  
  logout: async () => {
    try {
      await authApi.logout()
    } catch (err) {
      console.error('Logout error:', err)
    } finally {
      set({ user: null, loading: false })
    }
  },
  
  checkSession: async () => {
    try {
      const result = await authApi.getSession() as { user: User }
      set({ user: result.user, loading: false })
    } catch {
      set({ user: null, loading: false })
    }
  },
}))
