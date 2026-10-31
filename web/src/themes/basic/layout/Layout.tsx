import { Link, Outlet } from 'react-router'

/** g5 head.php / tail.php 대응 */
export function Layout() {
  return (
    <div className="flex min-h-dvh flex-col">
      <header className="border-b border-zinc-200 dark:border-zinc-800">
        <div className="mx-auto flex h-14 max-w-6xl items-center gap-6 px-4">
          <Link to="/" className="text-lg font-bold text-brand-700">
            beripickboard
          </Link>
          <nav className="text-sm text-zinc-600 dark:text-zinc-400">{/* g5_menu 연동 예정 */}</nav>
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
