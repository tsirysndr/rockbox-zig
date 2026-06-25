import { createFileRoute, redirect, useNavigate } from '@tanstack/react-router'
import { useAtomValue, useSetAtom } from 'jotai'
import { useState, type FormEvent } from 'react'
import {
  IconAlertTriangle,
  IconEye,
  IconEyeOff,
  IconKey,
  IconLoader2,
  IconLock,
  IconServer,
} from '@tabler/icons-react'
import { isAuthedAtom, sessionAtom } from '../atoms/auth'
import { verifyCredentials } from '../lib/s3'

export const Route = createFileRoute('/login')({
  beforeLoad: () => {
    const store = (window as unknown as { __JOTAI_STORE__?: { get: (a: typeof isAuthedAtom) => boolean } }).__JOTAI_STORE__
    if (store?.get(isAuthedAtom)) {
      throw redirect({ to: '/' })
    }
  },
  component: LoginPage,
})

function LoginPage() {
  const navigate = useNavigate()
  const setSession = useSetAtom(sessionAtom)
  const authed = useAtomValue(isAuthedAtom)
  const [accessKey, setAccessKey] = useState('')
  const [secretKey, setSecretKey] = useState('')
  const [showSecret, setShowSecret] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  if (authed) {
    navigate({ to: '/' })
    return null
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault()
    setError(null)
    setSubmitting(true)
    try {
      const ak = accessKey.trim()
      await verifyCredentials(ak, secretKey)
      setSession({ accessKey: ak, secretKey })
      navigate({ to: '/' })
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Login failed'
      setError(msg.includes('SignatureDoesNotMatch') ? 'Invalid credentials' : msg)
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="bg-base-200 relative flex min-h-screen items-center justify-center overflow-hidden p-6">
      <div className="bg-primary/25 pointer-events-none absolute -top-40 -left-40 h-[28rem] w-[28rem] rounded-full blur-3xl" />
      <div className="bg-secondary/25 pointer-events-none absolute -right-40 -bottom-40 h-[28rem] w-[28rem] rounded-full blur-3xl" />
      <div className="bg-accent/15 pointer-events-none absolute top-1/2 left-1/2 h-[36rem] w-[36rem] -translate-x-1/2 -translate-y-1/2 rounded-full blur-3xl" />
      <div
        className="pointer-events-none absolute inset-0 opacity-[0.05]"
        style={{
          backgroundImage:
            'linear-gradient(rgba(255,255,255,0.6) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.6) 1px, transparent 1px)',
          backgroundSize: '40px 40px',
        }}
      />

      <div className="card bg-base-100/80 border-base-content/10 relative z-10 w-full max-w-md border shadow-2xl backdrop-blur-xl">
        <div className="card-body gap-6 p-8">
          <div className="flex flex-col items-center gap-3 text-center">
            <div className="bg-primary/10 text-primary flex h-14 w-14 items-center justify-center rounded-2xl">
              <IconServer size={28} stroke={1.6} />
            </div>
            <div>
              <h1 className="text-2xl font-semibold tracking-tight">rockboxd</h1>
              <p className="text-base-content/60 text-sm">
                Sign in with your S3 access credentials
              </p>
            </div>
          </div>

          {error && (
            <div className="alert alert-soft alert-error text-sm">
              <IconAlertTriangle size={18} />
              <span>{error}</span>
            </div>
          )}

          <form onSubmit={onSubmit} className="flex flex-col gap-4">
            <div>
              <label className="label-text mb-1 block text-sm font-medium" htmlFor="ak">
                Access key
              </label>
              <label className="input">
                <IconKey size={18} className="text-base-content/50" />
                <input
                  id="ak"
                  type="text"
                  required
                  autoComplete="username"
                  spellCheck={false}
                  value={accessKey}
                  onChange={(e) => setAccessKey(e.target.value)}
                  placeholder="AKIA…"
                />
              </label>
            </div>

            <div>
              <label className="label-text mb-1 block text-sm font-medium" htmlFor="sk">
                Secret key
              </label>
              <label className="input">
                <IconLock size={18} className="text-base-content/50" />
                <input
                  id="sk"
                  type={showSecret ? 'text' : 'password'}
                  required
                  autoComplete="current-password"
                  value={secretKey}
                  onChange={(e) => setSecretKey(e.target.value)}
                />
                <button
                  type="button"
                  className="btn btn-square btn-ghost btn-xs"
                  onClick={() => setShowSecret((v) => !v)}
                  aria-label={showSecret ? 'Hide secret' : 'Show secret'}
                >
                  {showSecret ? <IconEyeOff size={16} /> : <IconEye size={16} />}
                </button>
              </label>
            </div>

            <button
              type="submit"
              className="btn btn-primary mt-2"
              disabled={submitting || !accessKey || !secretKey}
            >
              {submitting && <IconLoader2 size={18} className="animate-spin" />}
              {submitting ? 'Signing in…' : 'Sign in'}
            </button>
          </form>

          <p className="text-base-content/50 text-center text-xs">
            Credentials match your <code className="font-mono">s3_access_key</code> and{' '}
            <code className="font-mono">s3_secret_key</code> in your rockbox settings.
          </p>
        </div>
      </div>
    </div>
  )
}
