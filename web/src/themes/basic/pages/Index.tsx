import { useQuery } from '@tanstack/react-query'
import { getHealth } from '../../../api/client'

const badge: Record<string, string> = {
  ok: 'bg-emerald-100 text-emerald-800',
  degraded: 'bg-amber-100 text-amber-800',
  error: 'bg-red-100 text-red-800',
  disabled: 'bg-zinc-100 text-zinc-600',
}

export function Index() {
  const { data, error, isPending } = useQuery({ queryKey: ['health'], queryFn: getHealth })

  return (
    <section className="space-y-4">
      <h1 className="text-2xl font-bold">베리픽보드 v{data?.version ?? ''}</h1>
      <p className="text-zinc-600 dark:text-zinc-400">
        그누보드5 · 영카트5 · 이윰보드4 호환 CMS — 개발 뼈대가 동작 중입니다.
      </p>
      <div className="rounded-lg border border-zinc-200 p-4 dark:border-zinc-800">
        <h2 className="mb-3 font-semibold">서버 상태</h2>
        {isPending && <p className="text-sm text-zinc-500">확인 중…</p>}
        {error && <p className="text-sm text-red-600">API 서버에 연결할 수 없습니다: {error.message}</p>}
        {data && (
          <dl className="grid grid-cols-[8rem_1fr] gap-y-2 text-sm">
            {(
              [
                ['상태', data.status],
                ['DB', data.db],
                ['Redis', data.redis],
              ] as const
            ).map(([label, v]) => (
              <div key={label} className="contents">
                <dt className="text-zinc-500">{label}</dt>
                <dd>
                  <span className={`rounded px-2 py-0.5 text-xs font-medium ${badge[v] ?? ''}`}>{v}</span>
                </dd>
              </div>
            ))}
            <dt className="text-zinc-500">버전</dt>
            <dd>{data.version}</dd>
            <dt className="text-zinc-500">테이블 접두사</dt>
            <dd>
              <code>{data.tablePrefix}</code>
            </dd>
          </dl>
        )}
      </div>
    </section>
  )
}
