import { useActionState } from 'react'
import { Link, useSearchParams } from 'react-router'
import { ApiError } from '../../../../../api/client'
import { unlock, type CommentView, type ViewRes } from '../../../../../api/board'
import { PostContent, formatSize } from '../content'

/** g5 skin/board/basic/view.skin.php + view_comment.skin.php 대응 */
export function View({ data, onUnlocked }: { data: ViewRes; onUnlocked: () => void }) {
  const { board, post, files, comments, prev, next, downloadPoint } = data
  const [params] = useSearchParams()
  const keep = params.toString() ? `?${params}` : ''
  const images = files.filter((f) => f.isImage && f.imageUrl)

  const confirmDownload = (e: React.MouseEvent, source: string) => {
    if (downloadPoint < 0 && !confirm(`${source} 파일을 다운로드 하시면 포인트가 차감(${downloadPoint.toLocaleString()}점)됩니다.\n포인트는 게시물당 한번만 차감되며 다음에 다시 다운로드 하셔도 중복하여 차감하지 않습니다.\n그래도 다운로드 하시겠습니까?`)) {
      e.preventDefault()
    }
  }

  return (
    <article>
      <p className="text-xs text-zinc-500">
        <Link to={`/b/${board.boTable}${keep}`} className="hover:underline">
          {board.subject}
        </Link>
      </p>
      <header className="border-b border-zinc-200 pb-4 dark:border-zinc-800">
        <h1 className="mt-1 text-2xl font-bold break-words">
          {post.caName && <span className="mr-2 text-base text-brand-700">[{post.caName}]</span>}
          {post.isSecret && <span className="mr-1">🔒</span>}
          {post.subject}
        </h1>
        <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-sm text-zinc-500">
          <span>
            <b className="text-zinc-800 dark:text-zinc-200">{post.name}</b>
            {post.ip && <span className="ml-1">({post.ip})</span>}
          </span>
          <span>{post.datetime.slice(0, 16)}</span>
          <span>조회 {post.hit.toLocaleString()}</span>
          <span>댓글 {post.commentCount}</span>
        </div>
      </header>

      {files.length > 0 && (
        <ul className="mt-4 space-y-1 rounded-md border border-zinc-200 p-3 text-sm dark:border-zinc-800">
          {files.map((f) => (
            <li key={f.no} className="flex flex-wrap items-center gap-2">
              <span>📎</span>
              <a href={f.downloadUrl} onClick={(e) => confirmDownload(e, f.source)} className="font-medium hover:underline">
                {f.source}
              </a>
              <span className="text-zinc-500">
                ({formatSize(f.size)}) · {f.download}회 다운로드
              </span>
            </li>
          ))}
        </ul>
      )}

      {post.links.length > 0 && (
        <ul className="mt-3 space-y-1 text-sm">
          {post.links.map((l) => (
            <li key={l}>
              🔗{' '}
              <a href={/^https?:\/\//.test(l) ? l : `http://${l}`} target="_blank" rel="noopener noreferrer nofollow" className="text-brand-700 hover:underline">
                {l}
              </a>
            </li>
          ))}
        </ul>
      )}

      <div className="py-8">
        {images.map((f) => (
          <img key={f.no} src={f.imageUrl ?? undefined} alt={f.content || f.source} width={f.width || undefined} className="mb-4 h-auto max-w-full" loading="lazy" />
        ))}
        <PostContent content={post.content} mode={post.contentMode} className="leading-7" />
      </div>

      <Comments boTable={board.boTable} wrId={post.wrId} comments={comments} onUnlocked={onUnlocked} />

      <nav className="mt-8 divide-y divide-zinc-200 border-y border-zinc-200 text-sm dark:divide-zinc-800 dark:border-zinc-800">
        {prev && (
          <Link to={`/b/${board.boTable}/${prev.wrId}${keep}`} className="flex gap-3 px-2 py-2.5 hover:bg-zinc-50 dark:hover:bg-zinc-900">
            <span className="w-12 text-zinc-500">이전글</span>
            <span className="truncate">{prev.wrSubject}</span>
          </Link>
        )}
        {next && (
          <Link to={`/b/${board.boTable}/${next.wrId}${keep}`} className="flex gap-3 px-2 py-2.5 hover:bg-zinc-50 dark:hover:bg-zinc-900">
            <span className="w-12 text-zinc-500">다음글</span>
            <span className="truncate">{next.wrSubject}</span>
          </Link>
        )}
      </nav>
      <div className="mt-4">
        <Link to={`/b/${board.boTable}${keep}`} className="rounded-md border border-zinc-300 px-3 py-1.5 text-sm dark:border-zinc-700">
          목록
        </Link>
      </div>
    </article>
  )
}

function Comments({ boTable, wrId, comments, onUnlocked }: { boTable: string; wrId: number; comments: CommentView[]; onUnlocked: () => void }) {
  if (comments.length === 0) return null
  return (
    <section className="border-t border-zinc-200 pt-6 dark:border-zinc-800">
      <h2 className="mb-3 font-semibold">댓글 {comments.length}</h2>
      <ul className="space-y-3">
        {comments.map((c) => (
          <li key={c.wrId} id={`c_${c.wrId}`} className="rounded-md bg-zinc-50 p-3 dark:bg-zinc-900" style={{ marginLeft: c.depth * 24 }}>
            <div className="mb-1 flex flex-wrap gap-2 text-xs text-zinc-500">
              {c.depth > 0 && <span>ㄴ</span>}
              <b className="text-zinc-800 dark:text-zinc-200">{c.name}</b>
              {c.ip && <span>({c.ip})</span>}
              <span>{c.datetime.slice(2, 16)}</span>
              {c.isSecret && <span>🔒 비밀댓글</span>}
            </div>
            {c.content !== null ? (
              <PostContent content={c.content} mode="text" className="text-sm" />
            ) : (
              <PasswordForm label="댓글내용 확인" onSubmit={(pw) => unlock(boTable, wrId, pw, c.wrId).then(onUnlocked)} compact />
            )}
          </li>
        ))}
      </ul>
    </section>
  )
}

/** g5 password.php 대응 */
export function PasswordForm({ label, onSubmit, compact }: { label: string; onSubmit: (pw: string) => Promise<void>; compact?: boolean }) {
  const [error, action, pending] = useActionState(async (_: string | null, f: FormData) => {
    try {
      await onSubmit(String(f.get('wr_password') ?? ''))
      return null
    } catch (e) {
      return e instanceof ApiError ? e.message : '확인 중 오류가 발생했습니다.'
    }
  }, null)
  return (
    <form action={action} className={`flex flex-wrap items-center gap-2 ${compact ? '' : 'justify-center'}`}>
      <input
        type="password"
        name="wr_password"
        required
        placeholder="비밀번호"
        aria-label={`${label} 비밀번호`}
        className="rounded-md border border-zinc-300 px-3 py-1.5 text-sm dark:border-zinc-700 dark:bg-zinc-900"
      />
      <button disabled={pending} className="rounded-md bg-zinc-800 px-3 py-1.5 text-sm text-white disabled:opacity-60 dark:bg-zinc-200 dark:text-zinc-900">
        {label}
      </button>
      {error && (
        <span role="alert" className="w-full text-sm text-red-600">
          {error}
        </span>
      )}
    </form>
  )
}
