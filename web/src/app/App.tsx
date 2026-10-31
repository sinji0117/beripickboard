import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter, Route, Routes } from 'react-router'
import { Layout } from '../themes/basic/layout/Layout'
import { Index } from '../themes/basic/pages/Index'
import { Login } from '../themes/basic/pages/Login'
import { NotFound } from '../themes/basic/pages/NotFound'
import { SessionProvider } from './session'

const queryClient = new QueryClient({
  defaultOptions: { queries: { staleTime: 30_000, retry: 1 } },
})

// 테마 선택(g5_config.cf_theme)은 v0.0.7 에서 동적 로딩으로 교체한다 (doc/05_platform_design.md).
export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <SessionProvider>
        <BrowserRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route index element={<Index />} />
            <Route path="login" element={<Login />} />
            <Route path="*" element={<NotFound />} />
          </Route>
        </Routes>
        </BrowserRouter>
      </SessionProvider>
    </QueryClientProvider>
  )
}
