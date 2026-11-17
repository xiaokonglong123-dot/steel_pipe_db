import { createContext, useContext, type ReactNode } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, csrf, type Schemas } from './api/client'

export type Identity = Schemas['IdentityDto']

type SessionValue = {
  user: Identity | null
  loading: boolean
  isAdmin: boolean
  canWrite: boolean
  login: (username: string, password: string) => Promise<void>
  logout: () => Promise<void>
}

const SessionContext = createContext<SessionValue | null>(null)

export function SessionProvider({ children }: { children: ReactNode }) {
  const queryClient = useQueryClient()

  const me = useQuery({
    queryKey: ['me'],
    queryFn: async () => {
      try {
        return await api<Identity>('/api/auth/me')
      } catch {
        return null
      }
    },
    staleTime: 30_000,
  })

  const loginMutation = useMutation({
    mutationFn: async ({ username, password }: { username: string; password: string }) => {
      await csrf.ensure()
      return api<Schemas['LoginResponse']>('/api/auth/login', {
        method: 'POST',
        body: { username, password },
        skipRefresh: true,
      })
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['me'], data.user)
    },
  })

  const logoutMutation = useMutation({
    mutationFn: async () => api<Schemas['LogoutResponse']>('/api/auth/logout', { method: 'POST', body: {} }),
    onSettled: () => {
      queryClient.setQueryData(['me'], null)
      queryClient.clear()
    },
  })

  const user = me.data ?? null

  return (
    <SessionContext.Provider
      value={{
        user,
        loading: me.isLoading,
        isAdmin: user?.role === 'admin',
        canWrite: user?.role === 'admin' || user?.role === 'operator',
        login: async (username, password) => {
          await loginMutation.mutateAsync({ username, password })
        },
        logout: async () => {
          await logoutMutation.mutateAsync()
        },
      }}
    >
      {children}
    </SessionContext.Provider>
  )
}

export function useSession(): SessionValue {
  const value = useContext(SessionContext)
  if (!value) throw new Error('useSession 必须在 SessionProvider 内使用')
  return value
}
