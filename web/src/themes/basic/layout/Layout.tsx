import { useQuery } from '@tanstack/react-query'
import { useEffect } from 'react'
import { Link, NavLink, Outlet } from 'react-router'
import { listBoards } from '../../../api/board'
import { getSite } from '../../../api/client'
import { useSession } from '../../../app/session'

/** g5 head.php / tail.php 대응 */
export function Layout() {
  const { data: site } = useQuery({ queryKey: ['site'], queryFn: getSite, staleTime: 300_000 })
  const { member, ready, logout } = useSession()
  // 메뉴(g5_menu) 연동 전까지는 볼 수 있는 게시판을 메뉴로 사용
  const { data: boards } = useQuery({ queryKey: ['boards', member?.mbId], queryFn: listBoards, staleTime: 300_000 })
  const title = site?.title || 'beripickboard'

  useEffect(() => {
    document.title = title
  }, [title])

  return (
    <div className="flex min-h-dvh flex-col">
      <header className="border-b border-zinc-200 dark:border-zinc-800">
        <div className="mx-auto flex h-14 max-w-6xl items-center gap-6 px-4">
          <Link to="/" className="text-lg font-bold text-brand-700">
            {title}
          </Link>
          <nav className="flex flex-1 gap-4 overflow-x-auto text-sm text-zinc-600 dark:text-zinc-400">
            {boards?.map((b) => (
              <NavLink
                key={b.boTable}
                to={`/b/${b.boTable}`}
                className={({ isActive }) => `whitespace-nowrap hover:text-brand-700 ${isActive ? 'font-semibold text-brand-700' : ''}`}
              >
                {b.subject}
              </NavLink>
            ))}
          </nav>
          {/* g5 outlogin 대응 */}
          <div className="flex items-center gap-3 text-sm">
            {!ready ? null : member ? (
              <>
                <span>
                  <b>{member.mbNick}</b>님 · {member.mbPoint.toLocaleString()}P
                  {member.isAdmin && <span className="ml-1 rounded bg-brand-50 px-1.5 text-xs text-brand-700">관리자</span>}
                </span>
                <button onClick={() => void logout()} className="text-zinc-500 hover:underline">
                  로그아웃
                </button>
              </>
            ) : (
              <Link to="/login" className="rounded-md bg-brand-700 px-3 py-1.5 text-white hover:bg-brand-500">
                로그인
              </Link>
            )}
          </div>
        </div>
      </header>
      <main className="mx-auto w-full max-w-6xl flex-1 px-4 py-8">
        <Outlet />
      </main>
      <footer className="border-t border-zinc-200 py-6 text-center text-xs text-zinc-500 dark:border-zinc-800">
        Powered by beripickboard
      </footer>
    </div>
  )
}
