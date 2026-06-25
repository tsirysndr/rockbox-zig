import { atom } from 'jotai'
import { atomWithStorage } from 'jotai/utils'

export type AuthSession = {
  accessKey: string
  secretKey: string
}

const STORAGE_KEY = 'rockbox.admin.session'

export const sessionAtom = atomWithStorage<AuthSession | null>(
  STORAGE_KEY,
  null,
  undefined,
  { getOnInit: true },
)

export const isAuthedAtom = atom((get) => !!get(sessionAtom))
