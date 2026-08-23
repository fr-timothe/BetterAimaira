#!/usr/bin/env bun
/**
 * Generates the minisign keypair the desktop updater verifies bundles with, and
 * writes the public half into `src-tauri/tauri.conf.json`.
 *
 * The private key stays in `.tauri/` (git-ignored) and must be copied into the
 * `TAURI_SIGNING_PRIVATE_KEY` secret used by the release workflow. The public
 * key is not a secret: it ships inside the app.
 *
 *   bun run updater:keygen              # keeps an existing key
 *   bun run updater:keygen --force      # overwrites it (invalidates old feeds)
 */
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = resolve(__dirname, "..");
const KEY_DIR = resolve(ROOT_DIR, ".tauri");
const KEY_PATH = resolve(KEY_DIR, "betteraimaira.key");
const PUB_PATH = `${KEY_PATH}.pub`;
const CONFIG_PATH = resolve(ROOT_DIR, "src-tauri", "tauri.conf.json");

const force = process.argv.includes("--force");

if (existsSync(KEY_PATH) && !force) {
  console.log(`Existing key kept: ${KEY_PATH}`);
} else {
  mkdirSync(KEY_DIR, { recursive: true });
  // `--password=` keeps the empty value: a separate "" argument is dropped by
  // the shell on Windows and the CLI then reports a missing value.
  const args = ["tauri", "signer", "generate", "-w", KEY_PATH, "--password="];
  if (force) args.push("--force");

  const result = spawnSync("bunx", args, { stdio: "inherit", shell: true });
  if (result.status !== 0) {
    console.error("tauri signer generate failed");
    process.exit(result.status ?? 1);
  }
}

if (!existsSync(PUB_PATH)) {
  console.error(`Public key missing at ${PUB_PATH}`);
  process.exit(1);
}

const publicKey = readFileSync(PUB_PATH, "utf8").trim();
const config = JSON.parse(readFileSync(CONFIG_PATH, "utf8"));

config.plugins ??= {};
config.plugins.updater ??= {};
if (config.plugins.updater.pubkey === publicKey) {
  console.log("tauri.conf.json already carries this public key.");
} else {
  config.plugins.updater.pubkey = publicKey;
  writeFileSync(CONFIG_PATH, `${JSON.stringify(config, null, 2)}\n`, "utf8");
  console.log("Public key written to src-tauri/tauri.conf.json.");
}

console.log(
  [
    "",
    "Next steps:",
    `  1. Keep ${KEY_PATH} out of git (already ignored) and back it up.`,
    "  2. Add these repository secrets for the release workflow:",
    "       TAURI_SIGNING_PRIVATE_KEY          contents of the .key file",
    "       TAURI_SIGNING_PRIVATE_KEY_PASSWORD empty unless you set one",
    "  3. Losing the private key breaks updates for every installed build.",
  ].join("\n"),
);
