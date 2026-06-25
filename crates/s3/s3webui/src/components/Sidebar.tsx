import { Link, useNavigate, useRouterState } from '@tanstack/react-router'
import {
  IconCloudUpload,
  IconFolder,
  IconLayoutDashboard,
  IconLogout,
  IconSettings,
  IconServer,
  IconX,
} from '@tabler/icons-react'
import { useAtomValue, useSetAtom } from 'jotai'
import { useEffect } from 'react'
import { sessionAtom } from '../atoms/auth'
import { sidebarOpenAtom } from '../atoms/ui'

const NAV = [
  { to: '/', label: 'Overview', icon: IconLayoutDashboard, exact: true },
  { to: '/browser', label: 'Objects', icon: IconFolder, exact: false },
  { to: '/upload', label: 'Upload', icon: IconCloudUpload, exact: false },
  { to: '/settings', label: 'Settings', icon: IconSettings, exact: false },
] as const

export function Sidebar() {
  const path = useRouterState({ select: (s) => s.location.pathname })
  const setSession = useSetAtom(sessionAtom)
  const session = useAtomValue(sessionAtom)
  const navigate = useNavigate()
  const open = useAtomValue(sidebarOpenAtom)
  const setOpen = useSetAtom(sidebarOpenAtom)

  function signOut() {
    setOpen(false)
    setSession(null)
    navigate({ to: '/login', replace: true })
  }

  function isActive(to: string, exact: boolean): boolean {
    if (exact) return path === to
    return path === to || path.startsWith(`${to}/`)
  }

  // Close drawer on Escape; lock body scroll while open on mobile.
  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false)
    }
    document.addEventListener('keydown', onKey)
    document.body.style.overflow = 'hidden'
    return () => {
      document.removeEventListener('keydown', onKey)
      document.body.style.overflow = ''
    }
  }, [open, setOpen])

  return (
    <>
      {/* Mobile backdrop */}
      <div
        aria-hidden={!open}
        onClick={() => setOpen(false)}
        className={`bg-base-300/70 fixed inset-0 z-30 backdrop-blur-sm transition-opacity lg:hidden ${
          open ? 'opacity-100' : 'pointer-events-none opacity-0'
        }`}
      />

      <aside
        className={`bg-base-100 border-base-content/10 fixed inset-y-0 left-0 z-40 flex w-72 max-w-[85vw] shrink-0 flex-col border-r transition-transform lg:static lg:w-64 lg:translate-x-0 ${
          open ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'
        }`}
      >
        <div className="border-base-content/10 flex items-center gap-3 border-b px-5 py-5">
          <div className="bg-primary/10 text-primary flex h-9 w-9 shrink-0 items-center justify-center rounded-lg">
            <IconServer size={20} stroke={1.6} />
          </div>
          <div className="flex min-w-0 flex-col leading-tight">
            <span className="truncate text-sm font-semibold">rockboxd</span>
            <span className="text-base-content/50 text-xs">S3 admin</span>
          </div>
          <button
            type="button"
            className="btn btn-ghost btn-sm btn-square ms-auto lg:hidden"
            aria-label="Close menu"
            onClick={() => setOpen(false)}
          >
            <IconX size={16} />
          </button>
        </div>

        <nav className="flex-1 overflow-y-auto p-3">
          <ul className="menu menu-vertical w-full gap-1 p-0">
            {NAV.map((item) => {
              const Icon = item.icon
              const active = isActive(item.to, item.exact)
              return (
                <li key={item.to}>
                  <Link
                    to={item.to}
                    className={active ? 'menu-active' : ''}
                    onClick={() => setOpen(false)}
                  >
                    <Icon size={18} stroke={1.6} />
                    <span>{item.label}</span>
                  </Link>
                </li>
              )
            })}
          </ul>
        </nav>

        <div className="border-base-content/10 flex items-center justify-between gap-2 border-t p-3">
          <div className="flex min-w-0 items-center gap-2">
            <div className="avatar avatar-placeholder">
              <div className="bg-neutral text-neutral-content w-8 rounded-full">
                <span className="text-xs">
                  {(session?.accessKey ?? '?').slice(0, 2).toUpperCase()}
                </span>
              </div>
            </div>
            <div className="min-w-0 leading-tight">
              <div className="truncate text-sm font-medium">
                {session?.accessKey ?? '—'}
              </div>
              <div className="text-base-content/50 text-xs">signed in</div>
            </div>
          </div>
          <button
            type="button"
            className="btn btn-ghost btn-sm btn-square"
            aria-label="Sign out"
            onClick={signOut}
          >
            <IconLogout size={16} />
          </button>
        </div>
      </aside>
    </>
  )
}
