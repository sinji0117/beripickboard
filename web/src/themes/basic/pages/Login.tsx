import { useActionState } from 'react'
import { Navigate, useSearchParams } from 'react-router'
import { ApiError, login } from '../../../api/client'
import { useSession } from '../../../app/session'

/** g5 skin/member/basic/login.skin.php 대응 */
export function Login() {
  const { member, setMember } = useSession()
  const [params] = useSearchParams()
  const next = params.get('url') ?? '/'

  // React 19 폼 액션은 제출 후 폼을 초기화하므로, 실패 시 아이디를 상태로 돌려받아 defaultValue 로 유지한다.
  const [state, submit, pending] = useActionState(
    async (_: { error: string | null; mbId: string }, form: FormData) => {
      const mbId = String(form.get('mb_id') ?? '')
      try {
        setMember(await login(mbId, String(form.get('mb_password') ?? ''), form.get('auto_login') === 'on'))
        return { error: null, mbId }
      } catch (e) {
        return { error: e instanceof ApiError ? e.message : '로그인 중 오류가 발생했습니다.', mbId }
      }
    },
    { error: null, mbId: '' },
  )
  const error = state.error

  if (member) return <Navigate to={next.startsWith('/') ? next : '/'} replace />

  return (
    <section className="mx-auto max-w-sm py-10">
      <h1 className="mb-6 text-center text-2xl font-bold">회원 로그인</h1>
      <form action={submit} className="space-y-3">
        <input
          name="mb_id"
          defaultValue={state.mbId}
          required
          autoComplete="username"
          placeholder="아이디"
          className="w-full rounded-md border border-zinc-300 px-3 py-2 dark:border-zinc-700 dark:bg-zinc-900"
        />
        <input
          name="mb_password"
          type="password"
          required
          autoComplete="current-password"
          placeholder="비밀번호"
          className="w-full rounded-md border border-zinc-300 px-3 py-2 dark:border-zinc-700 dark:bg-zinc-900"
        />
        <label className="flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-400">
          <input type="checkbox" name="auto_login" /> 자동로그인
        </label>
        {error && (
          <p role="alert" className="rounded-md bg-red-50 px-3 py-2 text-sm text-red-700 dark:bg-red-950 dark:text-red-300">
            {error}
          </p>
        )}
        <button
          disabled={pending}
          className="w-full rounded-md bg-brand-700 py-2 font-medium text-white hover:bg-brand-500 disabled:opacity-60"
        >
          {pending ? '확인 중…' : '로그인'}
        </button>
      </form>
    </section>
  )
}
