import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { Provider, getDefaultStore } from 'jotai'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createRouter, RouterProvider } from '@tanstack/react-router'
import 'flyonui/flyonui'
import './index.css'
import { routeTree } from './routeTree.gen'

const store = getDefaultStore()
// Expose for route loaders that run outside React.
;(window as unknown as { __JOTAI_STORE__: typeof store }).__JOTAI_STORE__ = store

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 10_000,
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
})

const router = createRouter({
  routeTree,
  basepath: '/admin',
  defaultPreload: 'intent',
})

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <Provider store={store}>
        <RouterProvider router={router} />
      </Provider>
    </QueryClientProvider>
  </StrictMode>,
)
