import { createFileRoute } from '@tanstack/react-router'
import {
  IconCheck,
  IconCopy,
  IconKey,
  IconServer,
  IconDatabase,
} from '@tabler/icons-react'
import { useState } from 'react'
import { useAtomValue } from 'jotai'
import { sessionAtom } from '../atoms/auth'
import { BUCKET, REGION } from '../lib/s3'

export const Route = createFileRoute('/_app/settings')({
  component: SettingsPage,
})

function SettingsPage() {
  const session = useAtomValue(sessionAtom)
  const endpoint =
    typeof window !== 'undefined' ? window.location.origin : ''

  return (
    <div className="flex flex-col gap-4">
      <div className="card bg-base-100 border-base-content/10 border">
        <div className="card-body gap-4 p-5">
          <h2 className="card-title text-base">Endpoint</h2>
          <p className="text-base-content/60 text-sm">
            Use these values with <code className="font-mono">mc</code>,{' '}
            <code className="font-mono">aws-cli</code>, or any S3 SDK.
          </p>
          <Field
            icon={<IconServer size={16} />}
            label="Endpoint URL"
            value={endpoint}
          />
          <Field
            icon={<IconDatabase size={16} />}
            label="Bucket"
            value={BUCKET}
          />
          <Field
            icon={<IconServer size={16} />}
            label="Region"
            value={REGION}
          />
          <Field
            icon={<IconKey size={16} />}
            label="Access key"
            value={session?.accessKey ?? '—'}
          />
        </div>
      </div>

      <div className="card bg-base-100 border-base-content/10 border">
        <div className="card-body gap-3 p-5">
          <h2 className="card-title text-base">Quick start (mc)</h2>
          <CodeBlock
            code={`mc alias set smol ${endpoint} ${
              session?.accessKey ?? '<access-key>'
            } <secret-key> --api S3v4
mc ls smol/${BUCKET}/`}
          />
        </div>
      </div>

      <div className="card bg-base-100 border-base-content/10 border">
        <div className="card-body gap-3 p-5">
          <h2 className="card-title text-base">Quick start (aws-cli)</h2>
          <CodeBlock
            code={`aws --endpoint-url ${endpoint} \\
    s3 cp song.flac s3://${BUCKET}/Artist/Album/song.flac`}
          />
        </div>
      </div>
    </div>
  )
}

function Field({
  label,
  value,
  icon,
}: {
  label: string
  value: string
  icon: React.ReactNode
}) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-base-content/60 flex items-center gap-1.5 text-xs font-medium">
        {icon}
        {label}
      </span>
      <div className="bg-base-200 rounded-box flex items-center justify-between gap-2 p-3">
        <code className="truncate font-mono text-sm">{value}</code>
        <CopyButton value={value} />
      </div>
    </div>
  )
}

function CopyButton({ value }: { value: string }) {
  const [copied, setCopied] = useState(false)
  return (
    <button
      type="button"
      className="btn btn-ghost btn-xs btn-square"
      aria-label="Copy"
      onClick={async () => {
        await navigator.clipboard.writeText(value)
        setCopied(true)
        setTimeout(() => setCopied(false), 1500)
      }}
    >
      {copied ? (
        <IconCheck size={14} className="text-success" />
      ) : (
        <IconCopy size={14} />
      )}
    </button>
  )
}

function CodeBlock({ code }: { code: string }) {
  return (
    <pre className="bg-base-300 rounded-box overflow-x-auto p-3 font-mono text-xs">
      <code>{code}</code>
    </pre>
  )
}
