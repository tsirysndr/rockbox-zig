#!/usr/bin/env node
// Replace `node_modules/<name>` with a symlink to `modules/<name>` for each
// local Expo module declared as `file:./modules/<name>` in package.json.
//
// Bun resolves `"file:./modules/foo"` by *copying* the directory into
// `node_modules`, which means edits to the source module aren't visible
// until you re-run `bun install`. We need a live symlink so TS picks up new
// exports and Metro picks up native module changes immediately. This runs
// as the `postinstall` hook so every install converges on the symlinked
// layout.

const fs = require("node:fs");
const path = require("node:path");

const expoRoot = path.resolve(__dirname, "..");
const modulesRoot = path.join(expoRoot, "modules");
if (!fs.existsSync(modulesRoot)) {
  console.warn(`[link-local-modules] modules/ missing: ${modulesRoot}`);
  process.exit(0);
}

const moduleDirs = fs
  .readdirSync(modulesRoot, { withFileTypes: true })
  .filter((e) => e.isDirectory())
  .map((e) => e.name);

for (const name of moduleDirs) {
  const target = path.join(expoRoot, "node_modules", name);
  const source = path.join(modulesRoot, name);
  const relativeSource = path.relative(path.dirname(target), source);

  try {
    const stat = fs.lstatSync(target);
    if (stat.isSymbolicLink()) {
      const current = fs.readlinkSync(target);
      if (path.resolve(path.dirname(target), current) === source) {
        console.log(`[link-local-modules] ${name}: already linked → ok`);
        continue;
      }
    }
    fs.rmSync(target, { recursive: true, force: true });
  } catch (err) {
    if (err.code !== "ENOENT") throw err;
  }

  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.symlinkSync(relativeSource, target, "dir");
  console.log(`[link-local-modules] ${name}: linked → ${relativeSource}`);
}
