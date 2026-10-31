import { api } from './client'

export interface BoardInfo {
  boTable: string
  grId: string
  grSubject: string
  subject: string
  skin: string
  useCategory: boolean
  categories: string[]
  useGood: boolean
  useNogood: boolean
  useSecret: number
  galleryCols: number
  pageRows: number
  subjectLen: number
  canWrite: boolean
  admin: 'super' | 'group' | 'board' | null
}

export interface ListItem {
  wrId: number
  num: number
  isNotice: boolean
  isSecret: boolean
  depth: number
  caName: string
  subject: string
  content: string | null
  contentMode: ContentMode
  name: string
  mbId: string
  datetime: string
  last: string
  hit: number
  good: number
  nogood: number
  comment: number
  file: number
  hasLink: boolean
  isNew: boolean
  isHot: boolean
}

export type ContentMode = 'text' | 'html' | 'html_br'

export interface ListRes {
  board: BoardInfo
  items: ListItem[]
  page: number
  totalCount: number
  totalPage: number
  pageRows: number
  search: { spt: number; prevSpt: number | null; nextSpt: number | null } | null
}

export interface FileView {
  no: number
  source: string
  size: number
  download: number
  content: string
  isImage: boolean
  width: number
  height: number
  datetime: string
  downloadUrl: string
  imageUrl: string | null
}

export interface CommentView {
  wrId: number
  depth: number
  name: string
  mbId: string
  isSecret: boolean
  content: string | null
  datetime: string
  ip: string | null
  canEdit: boolean
  canReply: boolean
}

export interface Neighbor {
  wrId: number
  wrSubject: string
  wrDatetime: string
}

export interface ViewRes {
  board: BoardInfo
  post: {
    wrId: number
    wrNum: number
    depth: number
    caName: string
    subject: string
    content: string
    contentMode: ContentMode
    isSecret: boolean
    name: string
    mbId: string
    datetime: string
    hit: number
    good: number
    nogood: number
    commentCount: number
    links: string[]
    ip: string | null
  }
  files: FileView[]
  comments: CommentView[]
  prev: Neighbor | null
  next: Neighbor | null
  downloadPoint: number
  perm: Record<'canReply' | 'canEdit' | 'canDelete' | 'guestPassword' | 'canComment' | 'canGood' | 'canNogood' | 'canMove', boolean>
}

/** 비밀글·비밀댓글 열람 허가 (PHP 세션 대신 이 탭의 sessionStorage 에 보관) */
const GRANT_KEY = 'bp_secret_grants'
function loadGrants(): string[] {
  try {
    return JSON.parse(sessionStorage.getItem(GRANT_KEY) ?? '[]') as string[]
  } catch {
    return []
  }
}
function saveGrant(g: string) {
  try {
    sessionStorage.setItem(GRANT_KEY, JSON.stringify([...loadGrants(), g].slice(-30)))
  } catch {
    /* 저장 불가 환경 — 이번 요청에만 사용 */
  }
}

export interface BoardLink {
  boTable: string
  grId: string
  subject: string
  countWrite: number
}

export const listBoards = () => api<{ data: BoardLink[] }>('/boards').then((r) => r.data)

export const listPosts = (boTable: string, params: URLSearchParams) =>
  api<{ data: ListRes }>(`/boards/${encodeURIComponent(boTable)}/posts?${params}`).then((r) => r.data)

export function viewPost(boTable: string, wrId: number, params: URLSearchParams) {
  const grants = loadGrants()
  return api<{ data: ViewRes }>(`/boards/${encodeURIComponent(boTable)}/posts/${wrId}?${params}`, {
    headers: grants.length ? { 'x-secret-grant': grants.join(',') } : undefined,
  }).then((r) => r.data)
}

export async function unlock(boTable: string, wrId: number, password: string, commentId?: number) {
  const r = await api<{ data: { grant: string } }>(`/boards/${encodeURIComponent(boTable)}/posts/${wrId}/unlock`, {
    method: 'POST',
    body: JSON.stringify({ password, commentId }),
  })
  saveGrant(r.data.grant)
}
