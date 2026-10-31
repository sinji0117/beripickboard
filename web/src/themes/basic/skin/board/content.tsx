import DOMPurify from 'dompurify'
import type { ContentMode } from '../../../../api/board'

const URL_RE = /(https?:\/\/[^\s<]+)/g

/** g5 conv_content(): text 는 이스케이프 + 줄바꿈 + 자동링크, html 은 정제 후 출력 */
export function PostContent({ content, mode, className }: { content: string; mode: ContentMode; className?: string }) {
  if (mode === 'text') {
    const parts = content.split(URL_RE)
    return (
      <div className={`whitespace-pre-wrap break-words ${className ?? ''}`}>
        {parts.map((p, i) =>
          i % 2 === 1 ? (
            <a key={i} href={p} target="_blank" rel="noopener noreferrer nofollow" className="text-brand-700 underline">
              {p}
            </a>
          ) : (
            p
          ),
        )}
      </div>
    )
  }
  const raw = mode === 'html_br' ? content.replace(/\n/g, '<br/>') : content
  const clean = DOMPurify.sanitize(raw, { ADD_ATTR: ['target'], FORBID_TAGS: ['style', 'form', 'input'] })
  return <div className={`prose-g5 break-words ${className ?? ''}`} dangerouslySetInnerHTML={{ __html: clean }} />
}

export function formatSize(n: number): string {
  if (n >= 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)}M`
  if (n >= 1024) return `${(n / 1024).toFixed(1)}K`
  return `${n}B`
}

/** g5 목록 날짜: 오늘이면 시:분, 아니면 월-일 */
export function shortDate(dt: string): string {
  const today = new Date().toLocaleDateString('sv-SE', { timeZone: 'Asia/Seoul' })
  return dt.startsWith(today) ? dt.slice(11, 16) : dt.slice(5, 10)
}
