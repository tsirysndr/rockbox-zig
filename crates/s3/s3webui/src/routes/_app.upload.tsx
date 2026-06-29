import { createFileRoute, useNavigate } from '@tanstack/react-router'
import {
  IconAlertCircle,
  IconCheck,
  IconCloudUpload,
  IconFile,
  IconFolder,
  IconLoader2,
  IconTrash,
  IconX,
} from '@tabler/icons-react'
import { useQueryClient } from '@tanstack/react-query'
import { useCallback, useRef, useState, type DragEvent } from 'react'
import { v4 as uuidv4 } from 'uuid'
import { uploadObject } from '../lib/s3'
import { formatBytes } from '../lib/format'

type UploadSearch = { prefix?: string }

type UploadStatus = 'queued' | 'uploading' | 'done' | 'error'

type UploadItem = {
  id: string
  file: File
  relPath: string
  key: string
  status: UploadStatus
  progress: number
  error?: string
}

export const Route = createFileRoute('/_app/upload')({
  validateSearch: (search: Record<string, unknown>): UploadSearch => ({
    prefix: typeof search.prefix === 'string' ? search.prefix : undefined,
  }),
  component: UploadPage,
})

function UploadPage() {
  const { prefix: initialPrefix = '' } = Route.useSearch()
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [prefix, setPrefix] = useState(initialPrefix)
  const [dragging, setDragging] = useState(false)
  const [items, setItems] = useState<UploadItem[]>([])
  const fileInput = useRef<HTMLInputElement>(null)
  const folderInput = useRef<HTMLInputElement>(null)

  const enqueue = useCallback(
    (files: { file: File; relPath: string }[]) => {
      const normalized = prefix
        ? prefix.endsWith('/')
          ? prefix
          : prefix + '/'
        : ''
      setItems((prev) => [
        ...prev,
        ...files.map((f) => ({
          id: uuidv4(),
          file: f.file,
          relPath: f.relPath,
          key: normalized + f.relPath,
          status: 'queued' as UploadStatus,
          progress: 0,
        })),
      ])
    },
    [prefix],
  )

  function onPickFiles(list: FileList | null) {
    if (!list || list.length === 0) return
    const files = Array.from(list).map((file) => ({
      file,
      relPath: (file as File & { webkitRelativePath?: string })
        .webkitRelativePath || file.name,
    }))
    enqueue(files)
  }

  function onDrop(e: DragEvent) {
    e.preventDefault()
    setDragging(false)
    if (!e.dataTransfer.items?.length && !e.dataTransfer.files?.length) return
    const items = Array.from(e.dataTransfer.items)
    if (items.some((i) => i.webkitGetAsEntry?.())) {
      collectFromEntries(items).then(enqueue)
    } else {
      const fileList = Array.from(e.dataTransfer.files).map((file) => ({
        file,
        relPath: file.name,
      }))
      enqueue(fileList)
    }
  }

  async function startAll() {
    const queued = items.filter((i) => i.status === 'queued')
    for (const item of queued) {
      setItems((prev) =>
        prev.map((p) =>
          p.id === item.id ? { ...p, status: 'uploading', progress: 0 } : p,
        ),
      )
      try {
        await uploadObject(item.key, item.file, (loaded, total) => {
          const pct = total > 0 ? Math.round((loaded / total) * 100) : 0
          setItems((prev) =>
            prev.map((p) => (p.id === item.id ? { ...p, progress: pct } : p)),
          )
        })
        setItems((prev) =>
          prev.map((p) =>
            p.id === item.id ? { ...p, status: 'done', progress: 100 } : p,
          ),
        )
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'upload failed'
        setItems((prev) =>
          prev.map((p) =>
            p.id === item.id
              ? { ...p, status: 'error', error: msg }
              : p,
          ),
        )
      }
    }
    qc.invalidateQueries({ queryKey: ['list'] })
    qc.invalidateQueries({ queryKey: ['stats'] })
    qc.invalidateQueries({ queryKey: ['recent'] })
  }

  const queuedCount = items.filter((i) => i.status === 'queued').length
  const uploadingCount = items.filter((i) => i.status === 'uploading').length
  const doneCount = items.filter((i) => i.status === 'done').length

  return (
    <div className="flex flex-col gap-4">
      <div className="card bg-base-100 border-base-content/10 border">
        <div className="card-body gap-4 p-5">
          <div>
            <label
              className="label-text mb-1 block text-sm font-medium"
              htmlFor="prefix"
            >
              Destination prefix
            </label>
            <label className="input">
              <IconFolder size={16} className="text-base-content/50" />
              <input
                id="prefix"
                type="text"
                value={prefix}
                placeholder="Artist/Album/"
                spellCheck={false}
                onChange={(e) => setPrefix(e.target.value)}
              />
            </label>
            <p className="text-base-content/50 mt-1 text-xs">
              Files will be uploaded under{' '}
              <code className="font-mono">
                music/{prefix.replace(/\/?$/, '/')}
                {'<file>'}
              </code>
            </p>
          </div>

          <div
            className={`border-base-content/20 hover:border-primary/60 rounded-box flex flex-col items-center justify-center gap-3 border-2 border-dashed px-6 py-12 transition ${
              dragging
                ? 'border-primary bg-primary/5'
                : 'bg-base-200/40'
            }`}
            onDragOver={(e) => {
              e.preventDefault()
              setDragging(true)
            }}
            onDragLeave={() => setDragging(false)}
            onDrop={onDrop}
          >
            <div className="bg-primary/10 text-primary flex h-12 w-12 items-center justify-center rounded-2xl">
              <IconCloudUpload size={24} />
            </div>
            <div className="text-center">
              <p className="text-base font-medium">
                Drag files or folders here to upload
              </p>
              <p className="text-base-content/60 text-sm">
                or pick from your computer
              </p>
            </div>
            <div className="flex gap-2">
              <button
                type="button"
                className="btn btn-primary btn-sm"
                onClick={() => fileInput.current?.click()}
              >
                Choose files
              </button>
              <button
                type="button"
                className="btn btn-outline btn-sm"
                onClick={() => folderInput.current?.click()}
              >
                Choose folder
              </button>
            </div>
            <input
              ref={fileInput}
              type="file"
              multiple
              className="hidden"
              onChange={(e) => {
                onPickFiles(e.target.files)
                e.currentTarget.value = ''
              }}
            />
            <input
              ref={folderInput}
              type="file"
              multiple
              // @ts-expect-error non-standard but widely supported
              webkitdirectory=""
              className="hidden"
              onChange={(e) => {
                onPickFiles(e.target.files)
                e.currentTarget.value = ''
              }}
            />
          </div>
        </div>
      </div>

      {items.length > 0 && (
        <div className="card bg-base-100 border-base-content/10 border">
          <div className="card-body gap-3 p-5">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div>
                <h2 className="card-title text-base">Queue</h2>
                <p className="text-base-content/60 text-sm">
                  {queuedCount} queued · {uploadingCount} uploading ·{' '}
                  {doneCount} done
                </p>
              </div>
              <div className="flex gap-2">
                <button
                  type="button"
                  className="btn btn-ghost btn-sm"
                  onClick={() => setItems([])}
                  disabled={uploadingCount > 0}
                >
                  Clear
                </button>
                <button
                  type="button"
                  className="btn btn-primary btn-sm gap-1"
                  onClick={startAll}
                  disabled={queuedCount === 0 || uploadingCount > 0}
                >
                  {uploadingCount > 0 ? (
                    <>
                      <IconLoader2 size={16} className="animate-spin" />
                      Uploading…
                    </>
                  ) : (
                    <>Start upload</>
                  )}
                </button>
                {doneCount > 0 && (
                  <button
                    type="button"
                    className="btn btn-outline btn-sm"
                    onClick={() =>
                      navigate({
                        to: '/browser',
                        search: prefix ? { prefix } : {},
                      })
                    }
                  >
                    View in browser
                  </button>
                )}
              </div>
            </div>

            <ul className="flex flex-col gap-2">
              {items.map((it) => (
                <li
                  key={it.id}
                  className="bg-base-200/60 flex items-center gap-3 rounded-lg p-3"
                >
                  <div className="bg-base-100 text-base-content/70 flex h-9 w-9 shrink-0 items-center justify-center rounded-md">
                    <IconFile size={16} />
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="truncate text-sm font-medium">
                        {it.relPath}
                      </span>
                      <span className="text-base-content/50 text-xs">
                        {formatBytes(it.file.size)}
                      </span>
                    </div>
                    <div className="text-base-content/50 truncate font-mono text-xs">
                      → {it.key}
                    </div>
                    {(it.status === 'uploading' || it.status === 'done') && (
                      <progress
                        className="progress progress-primary mt-1 h-1"
                        value={it.progress}
                        max={100}
                      />
                    )}
                    {it.status === 'error' && (
                      <div className="text-error mt-1 flex items-center gap-1 text-xs">
                        <IconAlertCircle size={12} />
                        {it.error}
                      </div>
                    )}
                  </div>
                  <StatusBadge status={it.status} />
                  <button
                    type="button"
                    className="btn btn-ghost btn-xs btn-square"
                    aria-label="Remove"
                    disabled={it.status === 'uploading'}
                    onClick={() =>
                      setItems((prev) => prev.filter((p) => p.id !== it.id))
                    }
                  >
                    <IconTrash size={14} />
                  </button>
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  )
}

function StatusBadge({ status }: { status: UploadStatus }) {
  if (status === 'queued')
    return <span className="badge badge-soft badge-neutral">Queued</span>
  if (status === 'uploading')
    return (
      <span className="badge badge-soft badge-primary gap-1">
        <IconLoader2 size={12} className="animate-spin" />
        Uploading
      </span>
    )
  if (status === 'done')
    return (
      <span className="badge badge-soft badge-success gap-1">
        <IconCheck size={12} />
        Done
      </span>
    )
  return (
    <span className="badge badge-soft badge-error gap-1">
      <IconX size={12} />
      Failed
    </span>
  )
}

async function collectFromEntries(
  items: DataTransferItem[],
): Promise<{ file: File; relPath: string }[]> {
  const out: { file: File; relPath: string }[] = []
  const entries = items
    .map((i) => i.webkitGetAsEntry?.())
    .filter((e): e is FileSystemEntry => !!e)
  for (const entry of entries) {
    await walk(entry, '', out)
  }
  return out
}

async function walk(
  entry: FileSystemEntry,
  prefix: string,
  out: { file: File; relPath: string }[],
): Promise<void> {
  if (entry.isFile) {
    const file = await new Promise<File>((resolve, reject) =>
      (entry as FileSystemFileEntry).file(resolve, reject),
    )
    out.push({ file, relPath: prefix + entry.name })
    return
  }
  if (entry.isDirectory) {
    const reader = (entry as FileSystemDirectoryEntry).createReader()
    const children = await new Promise<FileSystemEntry[]>((resolve, reject) =>
      reader.readEntries(resolve, reject),
    )
    for (const child of children) {
      await walk(child, prefix + entry.name + '/', out)
    }
  }
}
