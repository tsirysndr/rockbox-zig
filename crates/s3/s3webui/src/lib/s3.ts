import {
  DeleteObjectCommand,
  GetObjectCommand,
  HeadBucketCommand,
  ListObjectsV2Command,
  PutObjectCommand,
  S3Client,
} from '@aws-sdk/client-s3'
import { getSignedUrl } from '@aws-sdk/s3-request-presigner'
import { getDefaultStore } from 'jotai'
import { sessionAtom, type AuthSession } from '../atoms/auth'

export const BUCKET = 'music'
export const REGION = 'us-east-1'

function endpoint(): string {
  if (typeof window === 'undefined') return ''
  return window.location.origin
}

function clientFor(session: AuthSession): S3Client {
  return new S3Client({
    region: REGION,
    endpoint: endpoint(),
    forcePathStyle: true,
    credentials: {
      accessKeyId: session.accessKey,
      secretAccessKey: session.secretKey,
    },
  })
}

function currentClient(): S3Client {
  const session = getDefaultStore().get(sessionAtom)
  if (!session) throw new Error('not authenticated')
  return clientFor(session)
}

export async function verifyCredentials(
  accessKey: string,
  secretKey: string,
): Promise<void> {
  const client = clientFor({ accessKey, secretKey })
  await client.send(new HeadBucketCommand({ Bucket: BUCKET }))
}

export type ObjectEntry = {
  key: string
  name: string
  size: number
  last_modified: string
  etag: string
}

export type PrefixEntry = {
  prefix: string
  name: string
}

export type ListResult = {
  prefix: string
  delimiter: string
  objects: ObjectEntry[]
  prefixes: PrefixEntry[]
  is_truncated: boolean
  next_token: string | null
}

export async function listObjects(
  prefix: string,
  opts: { delimiter?: string; continuationToken?: string | null } = {},
): Promise<ListResult> {
  const delimiter = opts.delimiter ?? '/'
  const client = currentClient()
  const res = await client.send(
    new ListObjectsV2Command({
      Bucket: BUCKET,
      Prefix: prefix || undefined,
      Delimiter: delimiter || undefined,
      ContinuationToken: opts.continuationToken ?? undefined,
      MaxKeys: 1000,
    }),
  )

  const objects: ObjectEntry[] = (res.Contents ?? []).map((c) => {
    const key = c.Key ?? ''
    const name = key.replace(/\/$/, '').split('/').pop() ?? key
    return {
      key,
      name,
      size: Number(c.Size ?? 0),
      last_modified: c.LastModified
        ? new Date(c.LastModified).toISOString()
        : '',
      etag: (c.ETag ?? '').replace(/"/g, ''),
    }
  })

  const prefixes: PrefixEntry[] = (res.CommonPrefixes ?? []).map((p) => {
    const full = p.Prefix ?? ''
    const trimmed = full.replace(/\/$/, '')
    const name = trimmed.split('/').pop() ?? trimmed
    return { prefix: full, name }
  })

  return {
    prefix,
    delimiter,
    objects,
    prefixes,
    is_truncated: !!res.IsTruncated,
    next_token: res.NextContinuationToken ?? null,
  }
}

export async function listAllObjects(): Promise<ObjectEntry[]> {
  const client = currentClient()
  const out: ObjectEntry[] = []
  let token: string | undefined
  do {
    const res = await client.send(
      new ListObjectsV2Command({
        Bucket: BUCKET,
        MaxKeys: 1000,
        ContinuationToken: token,
      }),
    )
    for (const c of res.Contents ?? []) {
      const key = c.Key ?? ''
      out.push({
        key,
        name: key.split('/').pop() ?? key,
        size: Number(c.Size ?? 0),
        last_modified: c.LastModified ? new Date(c.LastModified).toISOString() : '',
        etag: (c.ETag ?? '').replace(/"/g, ''),
      })
    }
    token = res.IsTruncated ? res.NextContinuationToken : undefined
  } while (token)
  return out
}

export async function deleteObject(key: string): Promise<void> {
  const client = currentClient()
  await client.send(new DeleteObjectCommand({ Bucket: BUCKET, Key: key }))
}

export async function getDownloadUrl(key: string): Promise<string> {
  const client = currentClient()
  return getSignedUrl(
    client,
    new GetObjectCommand({ Bucket: BUCKET, Key: key }),
    { expiresIn: 300 },
  )
}

export async function uploadObject(
  key: string,
  file: Blob,
  onProgress?: (loaded: number, total: number) => void,
): Promise<void> {
  // Use a presigned PUT so we can drive an XHR for progress events.
  const client = currentClient()
  const contentType = file.type || 'application/octet-stream'
  const url = await getSignedUrl(
    client,
    new PutObjectCommand({ Bucket: BUCKET, Key: key, ContentType: contentType }),
    {
      expiresIn: 3600,
      unhoistableHeaders: new Set(['content-type']),
    },
  )

  await new Promise<void>((resolve, reject) => {
    const xhr = new XMLHttpRequest()
    xhr.open('PUT', url)
    xhr.setRequestHeader('Content-Type', contentType)
    xhr.upload.onprogress = (e) => {
      if (e.lengthComputable && onProgress) onProgress(e.loaded, e.total)
    }
    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) resolve()
      else
        reject(
          new Error(
            `upload failed: ${xhr.status} ${xhr.statusText}${
              xhr.responseText ? ` — ${xhr.responseText.slice(0, 200)}` : ''
            }`,
          ),
        )
    }
    xhr.onerror = () => reject(new Error('network error'))
    xhr.send(file)
  })
}
