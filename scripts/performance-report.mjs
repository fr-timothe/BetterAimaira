#!/usr/bin/env bun
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { extname, join, relative, resolve } from "node:path";
import { gzipSync } from "node:zlib";

const root = resolve(import.meta.dir, "..");
const buildDirectory = join(root, "build");

if (!existsSync(buildDirectory)) {
  console.error("Build output is missing. Run `bun run build` first.");
  process.exit(1);
}

function collectFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...collectFiles(path));
    else if (entry.isFile()) files.push(path);
  }
  return files;
}

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KiB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MiB`;
}

const files = collectFiles(buildDirectory).map((path) => {
  const bytes = statSync(path).size;
  const extension = extname(path);
  const gzipBytes = extension === ".js" || extension === ".css"
    ? gzipSync(readFileSync(path)).byteLength
    : null;
  return {
    path: relative(root, path).replaceAll("\\", "/"),
    bytes,
    gzipBytes,
  };
});

const totals = files.reduce(
  (result, file) => {
    result.bytes += file.bytes;
    if (file.gzipBytes !== null) result.gzipBytes += file.gzipBytes;
    return result;
  },
  { bytes: 0, gzipBytes: 0 }
);

console.log(`Build total: ${formatBytes(totals.bytes)}`);
console.log(`JavaScript/CSS gzip total: ${formatBytes(totals.gzipBytes)}`);
console.log("Largest files:");
for (const file of files.sort((left, right) => right.bytes - left.bytes).slice(0, 10)) {
  const gzip = file.gzipBytes === null ? "" : `, ${formatBytes(file.gzipBytes)} gzip`;
  console.log(`  ${file.path}: ${formatBytes(file.bytes)}${gzip}`);
}
