#!/usr/bin/env bun
import { existsSync, readdirSync, statSync, rmSync, cpSync, mkdirSync } from "node:fs";
import { resolve, join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = resolve(__dirname, "..");
const TAURI_TARGET_DIR = resolve(ROOT_DIR, "src-tauri", "target");
const SVELTE_KIT_DIR = resolve(ROOT_DIR, ".svelte-kit");
const BUILD_DIR = resolve(ROOT_DIR, "build");
const DIST_DESKTOP_DIR = resolve(ROOT_DIR, "dist-desktop");

function getPathSize(targetPath) {
  if (!existsSync(targetPath)) return 0;
  try {
    const stats = statSync(targetPath);
    if (!stats.isDirectory()) {
      return stats.size;
    }
    let total = 0;
    const entries = readdirSync(targetPath, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = join(targetPath, entry.name);
      if (entry.isDirectory()) {
        total += getPathSize(fullPath);
      } else if (entry.isFile()) {
        try {
          total += statSync(fullPath).size;
        } catch {
          // ignore transient access errors
        }
      }
    }
    return total;
  } catch {
    return 0;
  }
}

function formatBytes(bytes) {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

function safeRemove(targetPath, label = targetPath) {
  if (!existsSync(targetPath)) return 0;
  const size = getPathSize(targetPath);
  try {
    rmSync(targetPath, { recursive: true, force: true, maxRetries: 3, retryDelay: 200 });
    console.log(`  [x] Removed ${label} (${formatBytes(size)})`);
    return size;
  } catch (err) {
    // If directory has a locked file (e.g. running process on Windows), try deleting children individually
    try {
      const stat = statSync(targetPath);
      if (stat.isDirectory()) {
        let partialFreed = 0;
        const entries = readdirSync(targetPath, { withFileTypes: true });
        for (const entry of entries) {
          const childPath = join(targetPath, entry.name);
          partialFreed += safeRemove(childPath, `${label}/${entry.name}`);
        }
        // Try removing the parent directory again after children have been removed
        try {
          rmSync(targetPath, { recursive: true, force: true });
        } catch {
          // Parent might still contain locked files
        }
        return partialFreed;
      }
    } catch {
      // ignore
    }
    console.warn(`  [!] Warning: Could not delete ${label} (file may be in use): ${err.message}`);
    return 0;
  }
}

function cleanIntermediateArtifacts() {
  console.log("🧹 Cleaning intermediate build artifacts...");
  let totalFreed = 0;

  if (!existsSync(TAURI_TARGET_DIR)) {
    console.log("  Nothing to clean in src-tauri/target.");
    return totalFreed;
  }

  // 1. Clean debug folder (usually 10-25+ GB from dev builds)
  const debugDir = join(TAURI_TARGET_DIR, "debug");
  if (existsSync(debugDir)) {
    totalFreed += safeRemove(debugDir, "src-tauri/target/debug");
  }

  // 2. Clean intermediate folders in release
  const releaseDir = join(TAURI_TARGET_DIR, "release");
  if (existsSync(releaseDir)) {
    const releaseSubdirsToClean = [
      "build",
      "deps",
      "incremental",
      ".fingerprint",
      "examples",
    ];

    for (const sub of releaseSubdirsToClean) {
      const p = join(releaseDir, sub);
      if (existsSync(p)) {
        totalFreed += safeRemove(p, `src-tauri/target/release/${sub}`);
      }
    }

    // 3. Clean heavy non-bundle files in release/ (like .pdb debug symbols, .rlib, .lib, .d)
    try {
      const entries = readdirSync(releaseDir, { withFileTypes: true });
      const heavyExtensions = [".pdb", ".rlib", ".lib", ".exp", ".d"];
      for (const entry of entries) {
        if (entry.isFile()) {
          const isHeavy = heavyExtensions.some((ext) => entry.name.toLowerCase().endsWith(ext));
          if (isHeavy) {
            totalFreed += safeRemove(join(releaseDir, entry.name), `src-tauri/target/release/${entry.name}`);
          }
        }
      }
    } catch (err) {
      console.warn(`  [!] Could not scan release directory: ${err.message}`);
    }
  }

  return totalFreed;
}

function exportAndCleanAll() {
  console.log("📦 Exporting bundle and purging target directory...");
  const releaseBundleDir = join(TAURI_TARGET_DIR, "release", "bundle");
  const releaseDir = join(TAURI_TARGET_DIR, "release");

  if (existsSync(releaseBundleDir)) {
    mkdirSync(DIST_DESKTOP_DIR, { recursive: true });
    cpSync(releaseBundleDir, join(DIST_DESKTOP_DIR, "bundle"), { recursive: true });
    console.log(`  [+] Copied bundle to dist-desktop/bundle`);
  }

  // Copy release executable if present
  if (existsSync(releaseDir)) {
    try {
      const entries = readdirSync(releaseDir, { withFileTypes: true });
      for (const entry of entries) {
        if (entry.isFile() && (entry.name.endsWith(".exe") || entry.name === "betteraimaira")) {
          mkdirSync(DIST_DESKTOP_DIR, { recursive: true });
          cpSync(join(releaseDir, entry.name), join(DIST_DESKTOP_DIR, entry.name));
          console.log(`  [+] Copied executable ${entry.name} to dist-desktop`);
        }
      }
    } catch {
      // ignore
    }
  }

  return cleanAll();
}

function cleanAll() {
  console.log("🧹 Running full cleanup...");
  let totalFreed = 0;
  totalFreed += safeRemove(TAURI_TARGET_DIR, "src-tauri/target");
  totalFreed += safeRemove(SVELTE_KIT_DIR, ".svelte-kit");
  totalFreed += safeRemove(BUILD_DIR, "build");
  return totalFreed;
}

function cleanFrontend() {
  console.log("🧹 Cleaning frontend build artifacts...");
  let totalFreed = 0;
  totalFreed += safeRemove(SVELTE_KIT_DIR, ".svelte-kit");
  totalFreed += safeRemove(BUILD_DIR, "build");
  return totalFreed;
}

function main() {
  const args = process.argv.slice(2);
  const isAll = args.includes("--all") || args.includes("-a");
  const isExport = args.includes("--export") || args.includes("-e");
  const isFrontend = args.includes("--frontend") || args.includes("-f");
  const isDesktop = args.includes("--desktop") || args.includes("-d") || args.includes("--intermediate") || args.includes("-i");

  const start = Date.now();
  let freed = 0;

  if (isExport) {
    freed = exportAndCleanAll();
  } else if (isAll) {
    freed = cleanAll();
  } else if (isFrontend) {
    freed = cleanFrontend();
  } else if (isDesktop || args.length === 0) {
    freed = cleanIntermediateArtifacts();
  }

  const duration = ((Date.now() - start) / 1000).toFixed(2);
  console.log(`✨ Cleanup completed in ${duration}s. Total space freed: ${formatBytes(freed)}.\n`);
}

main();
