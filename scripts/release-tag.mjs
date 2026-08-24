#!/usr/bin/env bun
/**
 * Tags the checked-out commit with the version in `package.json` and pushes it.
 *
 * The release workflow only ever runs on a pushed `v*` tag. Bumping the version
 * and pushing the commit alone therefore ships nothing: the versions 0.1.1-beta.7
 * and 0.1.1-beta.8 were bumped, committed and pushed with no tag, and no release
 * was ever built for them. This script is the missing half of a bump.
 *
 *   bun run release:tag            # tag v<package version> and push it
 *   bun run release:tag -- --dry-run
 *
 * It refuses to run on a dirty tree, on an unpushed commit or on a version that
 * is already tagged, so a mistake fails here rather than half-way through a
 * release build.
 */
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = resolve(__dirname, "..");

function git(...args) {
  return execFileSync("git", args, {
    cwd: ROOT_DIR,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  }).trim();
}

function tryGit(...args) {
  try {
    return git(...args);
  } catch {
    return null;
  }
}

function fail(message) {
  console.error(message);
  process.exit(1);
}

const dryRun = process.argv.includes("--dry-run");

const version = JSON.parse(readFileSync(resolve(ROOT_DIR, "package.json"), "utf8")).version;
const tag = `v${version}`;

if (git("status", "--porcelain")) {
  fail("Working tree is not clean. Commit or stash before tagging.");
}

if (tryGit("rev-parse", "--verify", `refs/tags/${tag}`)) {
  fail(`${tag} already exists locally. Bump the version in package.json first.`);
}

if (tryGit("ls-remote", "--exit-code", "--tags", "origin", tag)) {
  fail(`${tag} already exists on origin. Bump the version in package.json first.`);
}

// The tag must point at a commit the remote has, or the release builds a tree
// nobody else can see.
const head = git("rev-parse", "HEAD");
const branch = git("rev-parse", "--abbrev-ref", "HEAD");
const upstream = tryGit("rev-parse", "--verify", `origin/${branch}`);
if (upstream !== head) {
  fail(`HEAD is not pushed: origin/${branch} is at ${upstream ?? "nothing"}, HEAD at ${head.slice(0, 7)}.`);
}

if (dryRun) {
  console.log(`Would tag ${head.slice(0, 7)} as ${tag} and push it to origin.`);
  process.exit(0);
}

git("tag", "-a", tag, "-m", `BetterAimaira ${version}`);
git("push", "origin", tag);
console.log(`Tagged ${head.slice(0, 7)} as ${tag} and pushed it. The release workflow takes over from here.`);
