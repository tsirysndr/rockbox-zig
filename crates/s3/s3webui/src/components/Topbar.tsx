import { useNavigate, useRouterState } from '@tanstack/react-router'
import { useSetAtom } from 'jotai'
import { IconLogout, IconMenu2 } from '@tabler/icons-react'
import { sessionAtom } from '../atoms/auth'
import { sidebarOpenAtom } from '../atoms/ui'

const TITLES: Record<string, string> = {
  '/': 'Overview',
  '/browser': 'Objects',
  '/upload': 'Upload',
  '/settings': 'Settings',
}

export function Topbar() {
  const path = useRouterState({ select: (s) => s.location.pathname })
  const setSession = useSetAtom(sessionAtom)
  const setSidebarOpen = useSetAtom(sidebarOpenAtom)
  const navigate = useNavigate()

  function signOut() {
    setSession(null)
    navigate({ to: '/login', replace: true })
  }

  const title =
    TITLES[path] ?? (path.startsWith('/browser') ? 'Objects' : 'rockboxd')

  return (
    <header className="bg-base-100/80 border-base-content/10 sticky top-0 z-20 flex items-center justify-between gap-3 border-b px-3 py-3 backdrop-blur sm:px-4 lg:px-6">
      <div className="flex min-w-0 items-center gap-2">
        <button
          type="button"
          className="btn btn-ghost btn-sm btn-square lg:hidden"
          aria-label="Open menu"
          onClick={() => setSidebarOpen(true)}
        >
          <IconMenu2 size={18} />
        </button>
        <h1 className="truncate text-base font-semibold tracking-tight lg:text-lg">
          {title}
        </h1>
      </div>

      <div className="flex items-center gap-1">
        <button
          type="button"
          className="btn btn-ghost btn-sm btn-square lg:hidden"
          aria-label="Sign out"
          onClick={signOut}
        >
          <IconLogout size={16} />
        </button>
      </div>
    </header>
  )
}
