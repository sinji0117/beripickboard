/** API 공통 클라이언트 (doc/09_api.md). */
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

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    credentials: 'include',
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  })
  const body: unknown = await res.json().catch(() => null)
  if (!res.ok) {
    const err = (body as ApiErrorBody | null)?.error
    throw new ApiError(err ?? { code: 'NETWORK', message: res.statusText, status: res.status })
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

export const getHealth = () => api<Health>('/health')
