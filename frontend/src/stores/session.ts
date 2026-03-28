import { create } from 'zustand'

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
  logout: () => void
}

export const useSessionStore = create<SessionState>((set) => ({
  user: null,
  loading: true,
  setUser: (user) => set({ user, loading: false }),
  setLoading: (loading) => set({ loading }),
  logout: () => set({ user: null, loading: false }),
}))
