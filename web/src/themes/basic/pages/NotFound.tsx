import { Link } from 'react-router'

export function NotFound() {
  return (
    <section className="py-20 text-center">
      <h1 className="text-xl font-bold">페이지를 찾을 수 없습니다</h1>
      <Link to="/" className="mt-4 inline-block text-brand-700 underline">
        처음으로
      </Link>
    </section>
  )
}
