#!/usr/bin/env bun
/**
 * Writes the two update manifests published with every GitHub release:
 *
 *   latest.json    Tauri updater feed. Read by the desktop updater plugin and,
 *                  through the `android-universal` entry, by the Android check.
 *   altstore.json  AltStore/SideStore source. Read by the store and by the iOS
 *                  check, which can only report a newer build.
 *
 * The release workflow copies both onto the `gh-pages` branch, under the channel
 * the tag belongs to, and that GitHub Pages copy is what the app reads. This
 * script does not know about channels: the tag decides, so the workflow decides.
 *
 * Artifacts are discovered under the usual build output paths, so the common
 * case needs no flags:
 *
 *   bun run release:manifest
 *   bun run release:manifest -- --version 0.2.0 --notes "Grades sync fixes"
 *   bun run release:manifest -- --apk path/to/app.apk --ipa path/to/app.ipa
 *
 * Every download URL points at `releases/download/v<version>/<asset>`, so a
 * manifest stays valid after `latest` moves on.
 */
import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = resolve(__dirname, "..");

const REPOSITORY = "fr-timothe/BetterAimaira";
const BUNDLE_IDENTIFIER = "com.betteraimaira.app";
const APP_NAME = "BetterAimaira";
const DEVELOPER_NAME = "Timothé Montfrond";
const ICON_URL = `https://raw.githubusercontent.com/${REPOSITORY}/master/src-tauri/icons/128x128@2x.png`;
const DESCRIPTION =
  "BetterAimaira reads your Aimaira campus portal: schedule, grades, absences and documents, in one app.";

function parseArguments(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (!token.startsWith("--")) continue;
    const key = token.slice(2);
    const next = argv[index + 1];
    if (next === undefined || next.startsWith("--")) {
      options[key] = true;
    } else {
      options[key] = next;
      index += 1;
    }
  }
  return options;
}

/** First file matching `pattern` in `directory`, or null. Never recurses. */
function findFile(directory, pattern) {
  if (!existsSync(directory)) return null;
  const match = readdirSync(directory).find((entry) => pattern.test(entry));
  return match ? join(directory, match) : null;
}

function findRecursive(directory, pattern, depth = 4) {
  if (depth < 0 || !existsSync(directory)) return null;
  const entries = readdirSync(directory, { withFileTypes: true });
  for (const entry of entries) {
    const path = join(directory, entry.name);
    if (entry.isFile() && pattern.test(entry.name)) return path;
  }
  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const found = findRecursive(join(directory, entry.name), pattern, depth - 1);
    if (found) return found;
  }
  return null;
}

const options = parseArguments(process.argv.slice(2));

const tauriConfig = JSON.parse(
  readFileSync(resolve(ROOT_DIR, "src-tauri", "tauri.conf.json"), "utf8"),
);
const version = String(options.version ?? tauriConfig.version);
const tag = `v${version}`;
const notes = typeof options.notes === "string" ? options.notes : `BetterAimaira ${version}`;
const publishedAt = options.date ? new Date(String(options.date)) : new Date();
const outputDirectory = resolve(ROOT_DIR, String(options.out ?? "dist-release"));

/** GitHub serves release assets under the tag, which never moves. */
function assetUrl(fileName) {
  return `https://github.com/${REPOSITORY}/releases/download/${tag}/${encodeURIComponent(fileName)}`;
}

const BUNDLE_DIR = resolve(ROOT_DIR, "src-tauri", "target", "release", "bundle");
const ANDROID_OUTPUT_DIR = resolve(
  ROOT_DIR,
  "src-tauri",
  "gen",
  "android",
  "app",
  "build",
  "outputs",
  "apk",
);

const windowsInstaller =
  (typeof options["windows-installer"] === "string" && options["windows-installer"]) ||
  findFile(join(BUNDLE_DIR, "nsis"), /-setup\.exe$/i);
const windowsSignature =
  (typeof options["windows-signature"] === "string" && options["windows-signature"]) ||
  (windowsInstaller && existsSync(`${windowsInstaller}.sig`) ? `${windowsInstaller}.sig` : null);

const apk =
  (typeof options.apk === "string" && options.apk) ||
  findRecursive(ANDROID_OUTPUT_DIR, /\.apk$/i);

const ipa = typeof options.ipa === "string" ? options.ipa : null;

const platforms = {};
const missing = [];

if (windowsInstaller && windowsSignature) {
  platforms["windows-x86_64"] = {
    signature: readFileSync(windowsSignature, "utf8").trim(),
    url: assetUrl(basename(windowsInstaller)),
  };
} else {
  missing.push(
    "windows-x86_64 (need the NSIS installer and its .sig — build with createUpdaterArtifacts on)",
  );
}

if (apk) {
  // The Android entry is read by our own check, not by the updater plugin: the
  // APK is verified by its Android signing key, so there is no minisign
  // signature. The empty field stays because the plugin refuses to parse a
  // platform entry without it, which would break the desktop feed.
  platforms["android-universal"] = {
    signature: "",
    url: assetUrl(basename(apk)),
    size: statSync(apk).size,
  };
} else {
  missing.push("android-universal (no APK found)");
}

const latest = {
  version,
  notes,
  pub_date: publishedAt.toISOString(),
  platforms,
};

const altstore = {
  name: `${APP_NAME} (sideload)`,
  identifier: "fr.timothe.betteraimaira.source",
  subtitle: "Aimaira campus portal, unofficial client",
  description: DESCRIPTION,
  iconURL: ICON_URL,
  website: `https://github.com/${REPOSITORY}`,
  apps: [
    {
      name: APP_NAME,
      bundleIdentifier: BUNDLE_IDENTIFIER,
      developerName: DEVELOPER_NAME,
      subtitle: "Schedule, grades and absences",
      localizedDescription: DESCRIPTION,
      iconURL: ICON_URL,
      category: "education",
      appPermissions: { entitlements: [], privacy: {} },
      versions: ipa
        ? [
            {
              version,
              buildVersion: String(options.build ?? version.replaceAll(".", "")),
              date: publishedAt.toISOString(),
              localizedDescription: notes,
              downloadURL: assetUrl(basename(ipa)),
              size: statSync(ipa).size,
              minOSVersion: String(options["min-ios"] ?? "15.0"),
            },
          ]
        : [],
    },
  ],
  news: [],
};

if (!ipa) {
  missing.push("iOS version entry (no IPA passed with --ipa)");
}

mkdirSync(outputDirectory, { recursive: true });
writeFileSync(join(outputDirectory, "latest.json"), `${JSON.stringify(latest, null, 2)}\n`, "utf8");
writeFileSync(
  join(outputDirectory, "altstore.json"),
  `${JSON.stringify(altstore, null, 2)}\n`,
  "utf8",
);

console.log(`Manifests for ${tag} written to ${outputDirectory}`);
for (const entry of Object.keys(platforms)) console.log(`  included ${entry}`);
for (const entry of missing) console.log(`  skipped  ${entry}`);

// An empty feed would silently tell every client it is up to date.
if (Object.keys(platforms).length === 0 && !ipa) {
  console.error("No artifact found: the manifests would announce nothing.");
  process.exit(1);
}
