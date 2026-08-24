#!/usr/bin/env bun
/**
 * Keeps `bundle.android.versionCode` in `tauri.conf.json` in step with the
 * package version.
 *
 * Android decides what an update *is* from the version code alone: the package
 * installer refuses an APK whose code is lower than the installed one, and two
 * builds sharing a code are the same build as far as the system is concerned.
 * The code was hand-written, so `0.1.1-beta.4` and `0.1.1-beta.5` both shipped
 * `1004` and the second was not, to Android, a newer build at all.
 *
 * The code is derived, never chosen:
 *
 *   major * 1_000_000 + minor * 10_000 + patch * 100 + prerelease
 *
 * where `prerelease` is the trailing number of a prerelease tag (`beta.5` -> 5)
 * and `99` for a final release, so `0.1.1` outranks every `0.1.1-beta.n` that
 * led to it. `0.1.1-beta.5` is `10105`, `0.1.1` is `10199`, `0.2.0` is `20099`.
 *
 *   bun run android:version-code           write the derived code
 *   bun run android:version-code --check   fail if the file is out of step
 */
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = resolve(__dirname, "..");
const PACKAGE_FILE = resolve(ROOT_DIR, "package.json");
const CONFIG_FILE = resolve(ROOT_DIR, "src-tauri/tauri.conf.json");

/** A final release sits above the prereleases carrying the same version. */
const FINAL_RELEASE_RANK = 99;
const MAX_PRERELEASE_RANK = FINAL_RELEASE_RANK - 1;

function fail(message) {
  console.error(`android-version-code: ${message}`);
  process.exit(1);
}

/**
 * The rank of a prerelease tag, from its trailing number: `beta.5` is 5, and a
 * tag without one (`beta`) is a first prerelease.
 */
function prereleaseRank(prerelease) {
  if (!prerelease) return FINAL_RELEASE_RANK;
  const numbers = prerelease.match(/\d+/g);
  const rank = numbers === null ? 1 : Number(numbers.at(-1));
  if (rank < 1 || rank > MAX_PRERELEASE_RANK) {
    fail(
      `prerelease number ${rank} in "${prerelease}" is outside 1..${MAX_PRERELEASE_RANK};` +
        " release the final version or start a new patch level"
    );
  }
  return rank;
}

export function versionCodeOf(version) {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/);
  if (match === null) fail(`"${version}" is not a semantic version`);

  const [major, minor, patch] = match.slice(1, 4).map(Number);
  if (minor > 99) fail(`minor ${minor} does not fit: the derivation allows 0..99`);
  if (patch > 99) fail(`patch ${patch} does not fit: the derivation allows 0..99`);

  return major * 1_000_000 + minor * 10_000 + patch * 100 + prereleaseRank(match[4]);
}

/** Rewrites the config in place, or reports what is out of step. */
function main() {
  const checkOnly = process.argv.includes("--check");

  const packageVersion = JSON.parse(readFileSync(PACKAGE_FILE, "utf8")).version;
  const configSource = readFileSync(CONFIG_FILE, "utf8");
  const config = JSON.parse(configSource);

  // One version, in two files that both feed the build: a mismatch here would
  // silently ship a code derived from a version nothing else uses.
  if (config.version !== packageVersion) {
    fail(
      `package.json is ${packageVersion} but tauri.conf.json is ${config.version};` +
        " bump both before deriving the version code"
    );
  }

  const expected = versionCodeOf(packageVersion);
  const current = config.bundle?.android?.versionCode;

  if (current === expected) {
    console.log(`android-version-code: ${packageVersion} -> ${expected} (unchanged)`);
    return;
  }

  if (checkOnly) {
    fail(
      `tauri.conf.json carries versionCode ${current}, but ${packageVersion} derives ${expected};` +
        " run `bun run android:version-code`"
    );
  }

  // Rewritten in place rather than re-serialised, so the file keeps its tabs and
  // its key order instead of being reformatted by every version bump.
  const replaced = configSource.replace(
    /("versionCode"\s*:\s*)\d+/,
    (_match, key) => `${key}${expected}`
  );
  if (replaced === configSource) fail("no `versionCode` entry found in tauri.conf.json");

  writeFileSync(CONFIG_FILE, replaced);
  console.log(`android-version-code: ${packageVersion} -> ${expected} (was ${current})`);
}

// Importing this file to reuse `versionCodeOf` must not rewrite the config.
if (import.meta.main) main();
