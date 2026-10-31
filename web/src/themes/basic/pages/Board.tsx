import { useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, Navigate, useParams, useSearchParams } from 'react-router'
import { listPosts, unlock, viewPost } from '../../../api/board'
import { ApiError } from '../../../api/client'
import { List } from '../skin/board/basic/List'
import { PasswordForm, View } from '../skin/board/basic/View'

// bo_skin → 스킨 컴포넌트. 없는 스킨은 basic 으로 대체 (doc/05_platform_design.md)
const skins = { basic: { List, View } } as const
const pick = (name: string) => skins[name.replace(/^theme\//, '') as keyof typeof skins] ?? skins.basic

function ErrorBox({ error, boTable }: { error: Error; boTable?: string }) {
  const e = error instanceof ApiError ? error : null
  if (e?.code === 'AUTH_REQUIRED') {
    return <Navigate to={`/login?url=${encodeURIComponent(location.pathname + location.search)}`} replace />
  }
  return (
    <div className="py-20 text-center">
      <p className="text-zinc-700 dark:text-zinc-300">{error.message}</p>
      {boTable && (
        <Link to={`/b/${boTable}`} className="mt-4 inline-block text-brand-700 underline">
          목록으로
        </Link>
      )}
    </div>
  )
}

/** /b/:boTable — 목록 (bbs/board.php?bo_table=) */
export function BoardList() {
  const { boTable = '' } = useParams()
  const [params] = useSearchParams()
  const q = useQuery({ queryKey: ['board', boTable, params.toString()], queryFn: () => listPosts(boTable, params) })
  if (q.isPending) return <p className="py-20 text-center text-zinc-500">불러오는 중…</p>
  if (q.error) return <ErrorBox error={q.error} />
  const { List: Skin } = pick(q.data.board.skin)
  return <Skin data={q.data} />
}

/** /b/:boTable/:wrId — 글읽기 (bbs/board.php?bo_table=&wr_id=) */
export function BoardView() {
  const { boTable = '', wrId = '0' } = useParams()
  const [params] = useSearchParams()
  const qc = useQueryClient()
  const key = ['post', boTable, wrId, params.toString()]
  const q = useQuery({ queryKey: key, queryFn: () => viewPost(boTable, Number(wrId), params), staleTime: 0 })
  const reload = () => qc.invalidateQueries({ queryKey: key })

  if (q.isPending) return <p className="py-20 text-center text-zinc-500">불러오는 중…</p>
  if (q.error) {
    if (q.error instanceof ApiError && q.error.code === 'POST_SECRET') {
      return (
        <section className="mx-auto max-w-md py-16 text-center">
          <h1 className="mb-2 text-xl font-bold">🔒 비밀글</h1>
          <p className="mb-6 text-sm text-zinc-500">작성자와 관리자만 볼 수 있습니다. 비밀번호를 입력하세요.</p>
          <PasswordForm label="확인" onSubmit={(pw) => unlock(boTable, Number(wrId), pw).then(reload)} />
        </section>
      )
    }
    return <ErrorBox error={q.error} boTable={boTable} />
  }
  const { View: Skin } = pick(q.data.board.skin)
  return <Skin data={q.data} onUnlocked={() => void reload()} />
}

/** 구 주소 호환: /bbs/board.php?bo_table=free&wr_id=3 */
export function LegacyBoardRedirect() {
  const [params] = useSearchParams()
  const bo = params.get('bo_table')
  const wr = params.get('wr_id')
  if (!bo) return <Navigate to="/" replace />
  const rest = new URLSearchParams(params)
  rest.delete('bo_table')
  rest.delete('wr_id')
  const qs = rest.toString() ? `?${rest}` : ''
  return <Navigate to={wr ? `/b/${bo}/${wr}${qs}` : `/b/${bo}${qs}`} replace />
}
