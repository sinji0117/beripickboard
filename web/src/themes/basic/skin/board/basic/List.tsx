import { Link, useSearchParams } from 'react-router'
import type { ListRes } from '../../../../../api/board'
import { shortDate } from '../content'

/** g5 skin/board/basic/list.skin.php 대응 */
export function List({ data }: { data: ListRes }) {
  const { board, items, page, totalPage, totalCount, search } = data
  const [params, setParams] = useSearchParams()
  const sca = params.get('sca') ?? ''
  const go = (patch: Record<string, string | null>) => {
    const next = new URLSearchParams(params)
    for (const [k, v] of Object.entries(patch)) {
      if (v === null || v === '') next.delete(k)
      else next.set(k, v)
    }
    setParams(next)
  }
  const keep = params.toString() ? `?${params}` : ''

  return (
    <section>
      <header className="mb-4 flex items-end justify-between gap-4">
        <div>
          <p className="text-xs text-zinc-500">{board.grSubject}</p>
          <h1 className="text-2xl font-bold">{board.subject}</h1>
        </div>
        <span className="text-sm text-zinc-500">전체 {totalCount.toLocaleString()}건 · {page} 페이지</span>
      </header>

      {board.useCategory && board.categories.length > 0 && (
        <nav className="mb-4 flex flex-wrap gap-2 text-sm">
          {['', ...board.categories].map((c) => (
            <button
              key={c || '_all'}
              onClick={() => go({ sca: c, page: null, spt: null })}
              className={`rounded-full border px-3 py-1 ${sca === c ? 'border-brand-700 bg-brand-700 text-white' : 'border-zinc-300 dark:border-zinc-700'}`}
            >
              {c || '전체'}
            </button>
          ))}
        </nav>
      )}

      <div>
        <table className="w-full table-fixed border-t-2 border-zinc-800 text-sm dark:border-zinc-300">
          <thead>
            <tr className="border-b border-zinc-200 text-zinc-500 dark:border-zinc-800">
              <th className="hidden w-16 py-2 font-medium sm:table-cell">번호</th>
              <th className="py-2 font-medium">제목</th>
              <th className="hidden w-28 py-2 font-medium sm:table-cell">글쓴이</th>
              <th className="hidden w-16 py-2 font-medium sm:table-cell">조회</th>
              <th className="hidden w-20 py-2 font-medium sm:table-cell">날짜</th>
            </tr>
          </thead>
          <tbody>
            {items.length === 0 && (
              <tr>
                <td colSpan={5} className="py-16 text-center text-zinc-500">
                  게시물이 없습니다.
                </td>
              </tr>
            )}
            {items.map((it) => (
              <tr
                key={`${it.isNotice ? 'n' : 'r'}${it.wrId}`}
                className={`border-b border-zinc-100 dark:border-zinc-900 ${it.isNotice ? 'bg-brand-50/60 dark:bg-zinc-900' : ''}`}
              >
                <td className="hidden py-2.5 text-center text-zinc-500 sm:table-cell">
                  {it.isNotice ? <b className="text-brand-700">공지</b> : it.num}
                </td>
                <td className="py-2.5">
                  <div className="flex min-w-0 items-center gap-1.5" style={{ paddingLeft: it.depth * 16 }}>
                    {it.depth > 0 && <span className="text-zinc-400">ㄴ</span>}
                    {it.caName && <span className="text-xs text-brand-700">[{it.caName}]</span>}
                    {it.isSecret && <span title="비밀글">🔒</span>}
                    <Link to={`/b/${board.boTable}/${it.wrId}${keep}`} className={`min-w-0 truncate hover:underline ${it.isNotice ? 'font-semibold' : ''}`}>
                      {it.subject}
                    </Link>
                    {it.comment > 0 && <span className="text-xs font-semibold text-brand-700">[{it.comment}]</span>}
                    {it.file > 0 && <span title="첨부파일" className="text-xs">📎</span>}
                    {it.isNew && <span className="rounded bg-brand-500 px-1 text-[10px] font-bold text-white">N</span>}
                    {it.isHot && <span title="인기글" className="text-xs">🔥</span>}
                  </div>
                  {/* 모바일: 글쓴이·날짜·조회를 제목 아래에 */}
                  <div className="mt-0.5 text-xs text-zinc-500 sm:hidden" style={{ paddingLeft: it.depth * 16 }}>
                    {it.isNotice && <b className="mr-1 text-brand-700">공지</b>}
                    {it.name} · {shortDate(it.datetime)} · 조회 {it.hit.toLocaleString()}
                  </div>
                </td>
                <td className="hidden truncate py-2.5 text-center sm:table-cell">{it.name}</td>
                <td className="hidden py-2.5 text-center text-zinc-500 sm:table-cell">{it.hit.toLocaleString()}</td>
                <td className="hidden py-2.5 text-center text-zinc-500 sm:table-cell">{shortDate(it.datetime)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
        <Pager page={page} totalPage={totalPage} onGo={(p) => go({ page: String(p) })} />
        {board.canWrite && (
          <span className="rounded-md bg-zinc-200 px-3 py-1.5 text-sm text-zinc-500 dark:bg-zinc-800" title="v0.0.5 에서 지원">
            글쓰기
          </span>
        )}
      </div>

      {search && (search.prevSpt !== null || search.nextSpt !== null) && (
        <div className="mt-3 flex gap-2 text-sm">
          {search.prevSpt !== null && (
            <button onClick={() => go({ spt: String(search.prevSpt), page: '1' })} className="underline">
              이전검색
            </button>
          )}
          {search.nextSpt !== null && (
            <button onClick={() => go({ spt: String(search.nextSpt), page: '1' })} className="underline">
              다음검색
            </button>
          )}
        </div>
      )}

      <SearchForm
        initial={{ sfl: params.get('sfl') ?? 'wr_subject', stx: params.get('stx') ?? '' }}
        onSearch={(sfl, stx) => go({ sfl, stx, page: null, spt: null })}
      />
    </section>
  )
}

function Pager({ page, totalPage, onGo }: { page: number; totalPage: number; onGo: (p: number) => void }) {
  if (totalPage <= 1) return <span />
  // g5 cf_write_pages 기본 10
  const start = Math.floor((page - 1) / 10) * 10 + 1
  const end = Math.min(start + 9, totalPage)
  const btn = 'min-w-8 rounded px-2 py-1 text-sm'
  return (
    <nav className="flex flex-wrap gap-1" aria-label="페이지">
      {start > 1 && (
        <button className={btn} onClick={() => onGo(start - 1)}>
          이전
        </button>
      )}
      {Array.from({ length: end - start + 1 }, (_, i) => start + i).map((p) => (
        <button
          key={p}
          onClick={() => onGo(p)}
          aria-current={p === page ? 'page' : undefined}
          className={`${btn} ${p === page ? 'bg-brand-700 text-white' : 'hover:bg-zinc-100 dark:hover:bg-zinc-800'}`}
        >
          {p}
        </button>
      ))}
      {end < totalPage && (
        <button className={btn} onClick={() => onGo(end + 1)}>
          다음
        </button>
      )}
    </nav>
  )
}

function SearchForm({ initial, onSearch }: { initial: { sfl: string; stx: string }; onSearch: (sfl: string, stx: string) => void }) {
  return (
    <form
      className="mt-6 flex justify-center gap-2"
      onSubmit={(e) => {
        e.preventDefault()
        const f = new FormData(e.currentTarget)
        onSearch(String(f.get('sfl')), String(f.get('stx')).trim())
      }}
    >
      <select name="sfl" defaultValue={initial.sfl} className="rounded-md border border-zinc-300 px-2 py-1.5 text-sm dark:border-zinc-700 dark:bg-zinc-900">
        <option value="wr_subject">제목</option>
        <option value="wr_content">내용</option>
        <option value="wr_subject||wr_content">제목+내용</option>
        <option value="mb_id,1">회원아이디</option>
        <option value="mb_id,0">회원아이디(코)</option>
        <option value="wr_name,1">글쓴이</option>
        <option value="wr_name,0">글쓴이(코)</option>
      </select>
      <input
        name="stx"
        defaultValue={initial.stx}
        placeholder="검색어"
        className="w-48 rounded-md border border-zinc-300 px-3 py-1.5 text-sm dark:border-zinc-700 dark:bg-zinc-900"
      />
      <button className="rounded-md bg-zinc-800 px-3 py-1.5 text-sm text-white dark:bg-zinc-200 dark:text-zinc-900">검색</button>
    </form>
  )
}
