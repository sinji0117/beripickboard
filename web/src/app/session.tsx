import { createContext, use, useEffect, useState, type ReactNode } from 'react'
import { logout as apiLogout, subscribeSession, tryRefresh, type Member } from '../api/client'

interface Session {
  member: Member | null
  ready: boolean
  setMember: (m: Member | null) => void
  logout: () => Promise<void>
}

const SessionContext = createContext<Session | null>(null)

/** 새로고침 시 리프레시 쿠키로 로그인 상태 복원 */
export function SessionProvider({ children }: { children: ReactNode }) {
  const [member, setMember] = useState<Member | null>(null)
  const [ready, setReady] = useState(false)

  useEffect(() => {
    subscribeSession(setMember)
    void tryRefresh().finally(() => setReady(true))
  }, [])

  const logout = async () => {
    await apiLogout()
    setMember(null)
  }

  return <SessionContext value={{ member, ready, setMember, logout }}>{children}</SessionContext>
}

export function useSession(): Session {
  const s = use(SessionContext)
  if (!s) throw new Error('SessionProvider 밖에서 useSession 사용')
  return s
}
