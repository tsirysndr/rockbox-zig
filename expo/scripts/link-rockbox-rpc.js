#!/usr/bin/env node
// Replace `node_modules/rockbox-rpc` with a symlink to `modules/rockbox-rpc`.
//
// Bun resolves `"file:./modules/rockbox-rpc"` by *copying* the directory into
// `node_modules`, which means edits to the source module aren't visible until
// you re-run `bun install`. We need a live symlink so TS picks up new exports
// and Metro picks up native module changes immediately. This runs as the
// `postinstall` hook so every install converges on the symlinked layout.

const fs = require("node:fs");
const path = require("node:path");

const expoRoot = path.resolve(__dirname, "..");
const target = path.join(expoRoot, "node_modules", "rockbox-rpc");
const source = path.join(expoRoot, "modules", "rockbox-rpc");
const relativeSource = path.relative(path.dirname(target), source);

if (!fs.existsSync(source)) {
  console.warn(`[link-rockbox-rpc] source missing: ${source}`);
  process.exit(0);
}

// Skip if it's already the right symlink.
try {
  const stat = fs.lstatSync(target);
  if (stat.isSymbolicLink()) {
    const current = fs.readlinkSync(target);
    if (path.resolve(path.dirname(target), current) === source) {
      console.log("[link-rockbox-rpc] already linked → ok");
      process.exit(0);
    }
  }
  fs.rmSync(target, { recursive: true, force: true });
} catch (err) {
  if (err.code !== "ENOENT") throw err;
}

fs.mkdirSync(path.dirname(target), { recursive: true });
fs.symlinkSync(relativeSource, target, "dir");
console.log(`[link-rockbox-rpc] linked ${target} → ${relativeSource}`);
