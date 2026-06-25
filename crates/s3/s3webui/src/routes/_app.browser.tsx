import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  IconChevronRight,
  IconCloudUpload,
  IconDownload,
  IconFile,
  IconFolder,
  IconHome,
  IconLoader2,
  IconRefresh,
  IconSearch,
  IconTrash,
} from '@tabler/icons-react'
import { useMemo, useState } from 'react'
import { deleteObject, getDownloadUrl, listObjects } from '../lib/s3'
import { formatBytes, formatDate } from '../lib/format'

type BrowserSearch = { prefix?: string }

export const Route = createFileRoute('/_app/browser')({
  validateSearch: (search: Record<string, unknown>): BrowserSearch => ({
    prefix: typeof search.prefix === 'string' ? search.prefix : undefined,
  }),
  component: BrowserPage,
})

function BrowserPage() {
  const { prefix = '' } = Route.useSearch()
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [query, setQuery] = useState('')

  const list = useQuery({
    queryKey: ['list', prefix],
    queryFn: () => listObjects(prefix),
  })

  const remove = useMutation({
    mutationFn: (key: string) => deleteObject(key),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['list'] }),
  })

  const items = useMemo(() => {
    const folders = (list.data?.prefixes ?? []).map((p) => ({
      type: 'folder' as const,
      key: p.prefix,
      name: p.name,
    }))
    const files = (list.data?.objects ?? []).map((o) => ({
      type: 'file' as const,
      key: o.key,
      name: o.name,
      size: o.size,
      last_modified: o.last_modified,
      etag: o.etag,
    }))
    const all = [...folders, ...files]
    if (!query) return all
    const q = query.toLowerCase()
    return all.filter((i) => i.name.toLowerCase().includes(q))
  }, [list.data, query])

  function goTo(p: string) {
    navigate({ to: '/browser', search: p ? { prefix: p } : {} })
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="card bg-base-100 border-base-content/10 border">
        <div className="card-body gap-4 p-4 lg:p-5">
          <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
            <div className="min-w-0 max-w-full overflow-x-auto">
              <Breadcrumbs prefix={prefix} onNavigate={goTo} />
            </div>
            <div className="ms-auto flex items-center gap-2 sm:w-auto">
              <label className="input input-sm flex-1 sm:w-64 sm:flex-none">
                <IconSearch size={16} className="text-base-content/50" />
                <input
                  type="search"
                  placeholder="Filter…"
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                />
              </label>
              <button
                type="button"
                className="btn btn-ghost btn-sm btn-square"
                aria-label="Refresh"
                onClick={() => list.refetch()}
              >
                <IconRefresh size={16} />
              </button>
              <button
                type="button"
                className="btn btn-primary btn-sm gap-1"
                onClick={() =>
                  navigate({
                    to: '/upload',
                    search: prefix ? { prefix } : {},
                  })
                }
              >
                <IconCloudUpload size={16} />
                <span className="hidden sm:inline">Upload</span>
              </button>
            </div>
          </div>

          {list.isLoading ? (
            <Loading />
          ) : list.isError ? (
            <ErrorState message={(list.error as Error).message} />
          ) : items.length === 0 ? (
            <EmptyState />
          ) : (
            <div className="overflow-x-auto">
              <table className="table table-zebra table-sm">
                <thead>
                  <tr>
                    <th className="w-10"></th>
                    <th>Name</th>
                    <th className="hidden text-right sm:table-cell">Size</th>
                    <th className="hidden lg:table-cell">Modified</th>
                    <th className="w-24 text-right sm:w-32">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {items.map((item) =>
                    item.type === 'folder' ? (
                      <tr
                        key={`p:${item.key}`}
                        className="hover:bg-base-200/60 cursor-pointer"
                        onClick={() => goTo(item.key)}
                      >
                        <td>
                          <div className="bg-primary/10 text-primary inline-flex h-7 w-7 items-center justify-center rounded-md">
                            <IconFolder size={16} />
                          </div>
                        </td>
                        <td className="font-medium break-all">{item.name}</td>
                        <td className="text-base-content/50 hidden text-right sm:table-cell">
                          —
                        </td>
                        <td className="text-base-content/50 hidden lg:table-cell">
                          —
                        </td>
                        <td></td>
                      </tr>
                    ) : (
                      <tr key={`o:${item.key}`}>
                        <td>
                          <div className="bg-base-200 text-base-content/70 inline-flex h-7 w-7 items-center justify-center rounded-md">
                            <IconFile size={16} />
                          </div>
                        </td>
                        <td className="font-mono text-xs break-all">{item.name}</td>
                        <td className="hidden text-right tabular-nums sm:table-cell">
                          {formatBytes(item.size)}
                        </td>
                        <td className="text-base-content/60 hidden text-xs lg:table-cell">
                          {formatDate(item.last_modified)}
                        </td>
                        <td className="text-right">
                          <div className="inline-flex items-center gap-1">
                            <DownloadButton objectKey={item.key} />
                            <button
                              type="button"
                              className="btn btn-ghost btn-xs btn-square text-error"
                              aria-label="Delete"
                              disabled={remove.isPending}
                              onClick={() => {
                                if (
                                  confirm(`Delete "${item.key}"? This cannot be undone.`)
                                ) {
                                  remove.mutate(item.key)
                                }
                              }}
                            >
                              <IconTrash size={14} />
                            </button>
                          </div>
                        </td>
                      </tr>
                    ),
                  )}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>

      {list.data && (
        <p className="text-base-content/50 text-xs">
          {(list.data.prefixes?.length ?? 0) + (list.data.objects?.length ?? 0)}{' '}
          item(s) in this folder
          {list.data.is_truncated ? ' — listing truncated, refine prefix' : ''}
        </p>
      )}
    </div>
  )
}

function Breadcrumbs({
  prefix,
  onNavigate,
}: {
  prefix: string
  onNavigate: (p: string) => void
}) {
  const segments = prefix
    .replace(/\/$/, '')
    .split('/')
    .filter(Boolean)
  return (
    <nav className="breadcrumbs text-sm">
      <ul>
        <li>
          <button
            type="button"
            onClick={() => onNavigate('')}
            className="flex items-center gap-1"
          >
            <IconHome size={14} />
            <span>bucket</span>
          </button>
        </li>
        {segments.map((seg, i) => {
          const full = segments.slice(0, i + 1).join('/') + '/'
          const isLast = i === segments.length - 1
          return (
            <li key={full} className="flex items-center gap-1">
              <IconChevronRight size={12} className="text-base-content/40" />
              {isLast ? (
                <span className="font-medium">{seg}</span>
              ) : (
                <button type="button" onClick={() => onNavigate(full)}>
                  {seg}
                </button>
              )}
            </li>
          )
        })}
      </ul>
    </nav>
  )
}

function DownloadButton({ objectKey }: { objectKey: string }) {
  const [busy, setBusy] = useState(false)
  return (
    <button
      type="button"
      className="btn btn-ghost btn-xs btn-square"
      aria-label="Download"
      disabled={busy}
      onClick={async () => {
        setBusy(true)
        try {
          const url = await getDownloadUrl(objectKey)
          window.open(url, '_blank', 'noopener')
        } finally {
          setBusy(false)
        }
      }}
    >
      {busy ? (
        <IconLoader2 size={14} className="animate-spin" />
      ) : (
        <IconDownload size={14} />
      )}
    </button>
  )
}

function Loading() {
  return (
    <div className="text-base-content/60 flex items-center justify-center gap-2 py-12 text-sm">
      <IconLoader2 size={16} className="animate-spin" />
      Loading objects…
    </div>
  )
}

function ErrorState({ message }: { message: string }) {
  return (
    <div className="alert alert-soft alert-error text-sm">
      <span>{message}</span>
    </div>
  )
}

function EmptyState() {
  return (
    <div className="text-base-content/60 flex flex-col items-center gap-2 py-16 text-center text-sm">
      <IconFolder size={36} stroke={1.4} className="opacity-60" />
      <p>This folder is empty.</p>
    </div>
  )
}
