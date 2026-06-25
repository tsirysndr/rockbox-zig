import { createFileRoute, Link } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import {
  IconArrowUpRight,
  IconDatabase,
  IconFolder,
  IconLoader2,
  IconMusic,
  IconServer,
  IconUsers,
} from '@tabler/icons-react'
import { BUCKET, REGION, listAllObjects } from '../lib/s3'
import { formatBytes, formatDate } from '../lib/format'

export const Route = createFileRoute('/_app/')({
  component: OverviewPage,
})

function OverviewPage() {
  const all = useQuery({
    queryKey: ['all-objects'],
    queryFn: listAllObjects,
  })

  const totalSize = (all.data ?? []).reduce((acc, o) => acc + o.size, 0)
  const objectCount = (all.data ?? []).length
  const topLevel = new Set<string>()
  for (const o of all.data ?? []) {
    const slash = o.key.indexOf('/')
    topLevel.add(slash === -1 ? o.key : o.key.slice(0, slash))
  }

  const recentObjects = (all.data ?? [])
    .slice()
    .sort((a, b) => b.last_modified.localeCompare(a.last_modified))
    .slice(0, 8)

  return (
    <div className="flex flex-col gap-6">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <StatCard
          label="Objects"
          icon={<IconMusic size={20} />}
          value={all.isLoading ? '—' : objectCount.toLocaleString()}
          hint="Files indexed in the bucket"
          tone="primary"
        />
        <StatCard
          label="Total size"
          icon={<IconDatabase size={20} />}
          value={all.isLoading ? '—' : formatBytes(totalSize)}
          hint="On-disk usage"
          tone="secondary"
        />
        <StatCard
          label="Top-level folders"
          icon={<IconUsers size={20} />}
          value={all.isLoading ? '—' : topLevel.size.toLocaleString()}
          hint="Distinct first-segment prefixes"
          tone="accent"
        />
        <StatCard
          label="Region"
          icon={<IconServer size={20} />}
          value={REGION}
          hint={`Bucket "${BUCKET}"`}
          tone="info"
        />
      </div>

      <div className="grid grid-cols-1 gap-4 xl:grid-cols-3">
        <div className="card bg-base-100 border-base-content/10 border xl:col-span-2">
          <div className="card-body gap-4">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="card-title text-base">Recent uploads</h2>
                <p className="text-base-content/60 text-sm">
                  The most recently modified objects in the bucket.
                </p>
              </div>
              <Link
                to="/browser"
                className="btn btn-ghost btn-sm gap-1"
              >
                Browse all
                <IconArrowUpRight size={16} />
              </Link>
            </div>

            {all.isLoading ? (
              <div className="text-base-content/60 flex items-center gap-2 py-6 text-sm">
                <IconLoader2 size={16} className="animate-spin" />
                Loading…
              </div>
            ) : recentObjects.length === 0 ? (
              <EmptyState />
            ) : (
              <div className="overflow-x-auto">
                <table className="table table-sm">
                  <thead>
                    <tr>
                      <th>Key</th>
                      <th className="text-right">Size</th>
                      <th className="hidden lg:table-cell">Modified</th>
                    </tr>
                  </thead>
                  <tbody>
                    {recentObjects.map((o) => (
                      <tr key={o.key}>
                        <td className="font-mono text-xs">{o.key}</td>
                        <td className="text-right tabular-nums">
                          {formatBytes(o.size)}
                        </td>
                        <td className="text-base-content/60 hidden text-xs lg:table-cell">
                          {formatDate(o.last_modified)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </div>

        <div className="card bg-base-100 border-base-content/10 border">
          <div className="card-body gap-4">
            <h2 className="card-title text-base">Endpoint</h2>
            <p className="text-base-content/60 text-sm">
              The SPA talks to this server directly via the S3 API.
            </p>
            <div className="bg-base-200 rounded-box flex items-start gap-3 p-3">
              <IconFolder size={20} className="text-primary mt-0.5" />
              <code className="font-mono text-xs break-all">
                {typeof window !== 'undefined'
                  ? window.location.origin
                  : ''}
              </code>
            </div>
            <div className="divider my-0" />
            <div className="grid grid-cols-2 gap-2 text-sm">
              <span className="text-base-content/60">Bucket</span>
              <span className="font-mono text-right">{BUCKET}</span>
              <span className="text-base-content/60">Region</span>
              <span className="font-mono text-right">{REGION}</span>
              <span className="text-base-content/60">Auth</span>
              <span className="text-right">SigV4</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function StatCard({
  label,
  value,
  hint,
  icon,
  tone,
}: {
  label: string
  value: string | number
  hint: string
  icon: React.ReactNode
  tone: 'primary' | 'secondary' | 'accent' | 'info'
}) {
  const tones: Record<string, string> = {
    primary: 'bg-primary/10 text-primary',
    secondary: 'bg-secondary/10 text-secondary',
    accent: 'bg-accent/10 text-accent',
    info: 'bg-info/10 text-info',
  }
  return (
    <div className="card bg-base-100 border-base-content/10 border">
      <div className="card-body gap-3 p-5">
        <div className="flex items-center justify-between">
          <span className="text-base-content/60 text-sm font-medium">
            {label}
          </span>
          <div
            className={`flex h-9 w-9 items-center justify-center rounded-lg ${tones[tone]}`}
          >
            {icon}
          </div>
        </div>
        <div className="text-2xl font-semibold tracking-tight tabular-nums">
          {value}
        </div>
        <div className="text-base-content/50 text-xs">{hint}</div>
      </div>
    </div>
  )
}

function EmptyState() {
  return (
    <div className="text-base-content/60 flex flex-col items-center gap-2 py-10 text-center text-sm">
      <IconMusic size={32} stroke={1.4} className="opacity-60" />
      <p>No objects yet. Upload your first file to get started.</p>
      <Link to="/upload" className="btn btn-primary btn-sm mt-2">
        Upload files
      </Link>
    </div>
  )
}
