import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { TanStackRouterVite } from '@tanstack/router-plugin/vite'

// SPA is served at /admin/ inside the S3 server, and the built bundle is
// embedded directly into the Rust binary via rust-embed.
//
// The dashboard talks to the S3 API directly (SigV4-signed via the AWS SDK).
// During `vite dev`, requests to the bucket path (/music, /music/*) are
// proxied to the local smolsonic S3 server so the dashboard works against a
// real backend without rebuilding.
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  const apiTarget = env.VITE_API_TARGET ?? 'http://localhost:9000'

  return {
    base: '/admin/',
    plugins: [
      TanStackRouterVite({
        target: 'react',
        autoCodeSplitting: true,
        routesDirectory: './src/routes',
        generatedRouteTree: './src/routeTree.gen.ts',
      }),
      react(),
      tailwindcss(),
    ],
    build: {
      outDir: 'dist',
      emptyOutDir: true,
    },
    server: {
      port: 5173,
      proxy: {
        // NOTE: changeOrigin must stay false. The AWS SDK signs the request
        // with the original Host header (localhost:5173) — rewriting it would
        // invalidate the SigV4 signature.
        '/music': {
          target: apiTarget,
          changeOrigin: false,
          secure: false,
        },
      },
    },
  }
})
