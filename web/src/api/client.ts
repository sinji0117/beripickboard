/** API 공통 클라이언트 (doc/09_api.md). Access 토큰은 메모리에만 둔다. */
export const API_BASE = '/api/v1'

export interface ApiErrorBody {
  error: { code: string; message: string; status: number }
}

export class ApiError extends Error {
  readonly code: string
  readonly status: number

  constructor(body: ApiErrorBody['error']) {
    super(body.message)
    this.code = body.code
    this.status = body.status
  }
}

let accessToken: string | null = null
export const setAccessToken = (t: string | null) => {
  accessToken = t
}

let refreshing: Promise<boolean> | null = null
/** 리프레시 쿠키로 Access 토큰 재발급 (동시 호출은 하나로 합침) */
export function tryRefresh(): Promise<boolean> {
  refreshing ??= fetch(`${API_BASE}/auth/refresh`, { method: 'POST', credentials: 'include' })
    .then(async (res) => {
      if (!res.ok) {
        setAccessToken(null)
        return false
      }
      const body = (await res.json()) as { data: TokenRes }
      setAccessToken(body.data.accessToken)
      onSession?.(body.data.member)
      return true
    })
    .catch(() => false)
    .finally(() => {
      refreshing = null
    })
  return refreshing
}

let onSession: ((m: Member | null) => void) | null = null
export const subscribeSession = (fn: (m: Member | null) => void) => {
  onSession = fn
}

export async function api<T>(path: string, init?: RequestInit, retry = true): Promise<T> {
  const headers = new Headers(init?.headers)
  if (init?.body && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json')
  if (accessToken) headers.set('Authorization', `Bearer ${accessToken}`)
  const res = await fetch(`${API_BASE}${path}`, { credentials: 'include', ...init, headers })
  if (res.status === 204) return undefined as T
  const body: unknown = await res.json().catch(() => null)
  if (!res.ok) {
    const err = (body as ApiErrorBody | null)?.error ?? { code: 'NETWORK', message: res.statusText, status: res.status }
    if (retry && err.code === 'AUTH_EXPIRED' && (await tryRefresh())) return api<T>(path, init, false)
    throw new ApiError(err)
  }
  return body as T
}

export interface Health {
  status: 'ok' | 'degraded'
  version: string
  env: string
  db: 'ok' | 'error' | 'disabled'
  redis: 'ok' | 'error' | 'disabled'
  tablePrefix: string
}

export interface Site {
  title: string
  theme: string
}

/** g5_member 공개 필드 */
export interface Member {
  mbId: string
  mbNick: string
  mbName: string
  mbEmail: string
  mbLevel: number
  mbPoint: number
  isAdmin: 'super' | null
}

export interface TokenRes {
  accessToken: string
  expiresIn: number
  member: Member
}

export const getHealth = () => api<Health>('/health')
export const getSite = () => api<{ data: Site }>('/site').then((r) => r.data)

export async function login(mbId: string, password: string, autoLogin: boolean): Promise<Member> {
  const r = await api<{ data: TokenRes }>('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ mbId, password, autoLogin }),
  })
  setAccessToken(r.data.accessToken)
  return r.data.member
}

export async function logout(): Promise<void> {
  await api<void>('/auth/logout', { method: 'POST' }).catch(() => undefined)
  setAccessToken(null)
}
