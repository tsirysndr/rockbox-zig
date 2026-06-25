import { createFileRoute, Outlet, redirect } from '@tanstack/react-router'
import { getDefaultStore } from 'jotai'
import { isAuthedAtom } from '../atoms/auth'
import { Sidebar } from '../components/Sidebar'
import { Topbar } from '../components/Topbar'

export const Route = createFileRoute('/_app')({
  beforeLoad: () => {
    if (!getDefaultStore().get(isAuthedAtom)) {
      throw redirect({ to: '/login' })
    }
  },
  component: AppLayout,
})

function AppLayout() {
  return (
    <div className="bg-base-200 relative flex h-full min-h-screen">
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <Topbar />
        <main className="min-h-0 flex-1 overflow-y-auto p-3 sm:p-4 lg:p-6">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
