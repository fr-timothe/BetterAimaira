#!/usr/bin/env bun
/**
 * Writes the body of a GitHub release from the commits a tag adds.
 *
 * `gh release --generate-notes` lists pull requests, and this repository ships
 * straight from `master`: it produced a release page with nothing but a compare
 * link. This script reads the commits between the previous tag and the one being
 * released, groups them by conventional-commit type and renders the markdown the
 * release workflow feeds to `gh release create --notes-file`.
 *
 *   bun run release:notes
 *   bun run release:notes -- --tag v0.2.0
 *   bun run release:notes -- --tag v0.2.0 --previous v0.1.9 --output notes.md
 *
 * With no `--previous`, the closest `v*` tag reachable from the tag's first
 * parent is used, so a beta lists only what it adds to the beta before it.
 */
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = resolve(__dirname, "..");

const REPOSITORY = "fr-timothe/BetterAimaira";

// Order matters: it is the order of the sections in the rendered notes.
const SECTIONS = [
  { title: "New", types: ["feat"] },
  { title: "Fixed", types: ["fix"] },
  { title: "Performance", types: ["perf"] },
  { title: "Under the hood", types: ["refactor", "build", "ci", "style", "test", "chore"] },
  { title: "Documentation", types: ["docs"] },
];
const OTHER_SECTION_TITLE = "Other changes";

// A bump commit says nothing a reader of the release page does not already know
// from its title.
const NOISE_SUBJECT = /^chore(\([^)]*\))?!?:\s*bump version\b/i;

const CONVENTIONAL_SUBJECT = /^(?<type>[a-z]+)(?:\((?<scope>[^)]*)\))?(?<breaking>!)?:\s*(?<rest>.+)$/;

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

function git(...args) {
  // stderr is piped, not inherited: a probe for a tag that does not exist yet
  // must not print a git error into the workflow log.
  return execFileSync("git", args, {
    cwd: ROOT_DIR,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  }).trim();
}

/** Same call, but a failure is an answer rather than a crash. */
function tryGit(...args) {
  try {
    return git(...args);
  } catch {
    return null;
  }
}

function readPackageVersion() {
  const manifest = JSON.parse(readFileSync(resolve(ROOT_DIR, "package.json"), "utf8"));
  return manifest.version;
}

/**
 * The tag this release is for. In the workflow it comes from the ref; run by
 * hand it falls back to the version in `package.json`, tagged or not.
 */
function resolveTag(options) {
  if (typeof options.tag === "string") return options.tag;
  if (process.env.GITHUB_REF_NAME) return process.env.GITHUB_REF_NAME;
  return `v${readPackageVersion()}`;
}

/**
 * The tag the notes start from. `git describe` walks the first parent, so a
 * merge does not drag in the tags of the branch it merged.
 */
function resolvePreviousTag(options, tag, range) {
  if (typeof options.previous === "string") return options.previous;
  const previous = tryGit("describe", "--tags", "--abbrev=0", "--match", "v*", "--first-parent", `${range}^`);
  return previous && previous !== tag ? previous : null;
}

function readCommits(from, to) {
  // A unit separator keeps a subject holding anything from breaking the split.
  const output = tryGit("log", "--no-merges", "--pretty=%H%x1f%s%x1f%b%x1e", from ? `${from}..${to}` : to);
  if (!output) return [];
  return output
    .split("\x1e")
    .map((entry) => entry.trim())
    .filter(Boolean)
    .map((entry) => {
      const [hash, subject, body = ""] = entry.split("\x1f");
      return { hash, subject, body };
    });
}

function classify(commit) {
  const match = CONVENTIONAL_SUBJECT.exec(commit.subject);
  const breaking = Boolean(match?.groups.breaking) || /^BREAKING[ -]CHANGE:/m.test(commit.body);
  if (!match) return { type: null, scope: null, description: commit.subject, breaking };
  return {
    type: match.groups.type,
    scope: match.groups.scope || null,
    description: match.groups.rest,
    breaking,
  };
}

function renderEntry(commit, change) {
  const short = commit.hash.slice(0, 7);
  const link = `https://github.com/${REPOSITORY}/commit/${commit.hash}`;
  const description = change.description.charAt(0).toUpperCase() + change.description.slice(1);
  const scope = change.scope ? `**${change.scope}** — ` : "";
  return `- ${scope}${description} ([\`${short}\`](${link}))`;
}

function render(tag, previousTag, commits) {
  const kept = commits.filter((commit) => !NOISE_SUBJECT.test(commit.subject));
  const breaking = [];
  const bySection = new Map();

  for (const commit of kept) {
    const change = classify(commit);
    const entry = renderEntry(commit, change);
    if (change.breaking) breaking.push(entry);
    const section = SECTIONS.find((candidate) => change.type && candidate.types.includes(change.type));
    const title = section ? section.title : OTHER_SECTION_TITLE;
    if (!bySection.has(title)) bySection.set(title, []);
    bySection.get(title).push(entry);
  }

  const lines = [];
  if (breaking.length > 0) {
    lines.push("## Breaking changes", "", ...breaking, "");
  }
  for (const { title } of SECTIONS) {
    const entries = bySection.get(title);
    if (entries?.length) lines.push(`## ${title}`, "", ...entries, "");
  }
  const others = bySection.get(OTHER_SECTION_TITLE);
  if (others?.length) lines.push(`## ${OTHER_SECTION_TITLE}`, "", ...others, "");

  if (lines.length === 0) {
    lines.push("No commit carries a user-visible change in this release.", "");
  }

  const compare = previousTag
    ? `https://github.com/${REPOSITORY}/compare/${previousTag}...${tag}`
    : `https://github.com/${REPOSITORY}/commits/${tag}`;
  lines.push(`**Full changelog**: ${compare}`);

  return `${lines.join("\n")}\n`;
}

function main() {
  const options = parseArguments(process.argv.slice(2));
  const tag = resolveTag(options);
  // Notes can be written before the tag exists, from whatever is checked out.
  const range = tryGit("rev-parse", "--verify", `${tag}^{commit}`) ? tag : "HEAD";
  const previousTag = resolvePreviousTag(options, tag, range);
  const commits = readCommits(previousTag, range);
  const notes = render(tag, previousTag, commits);

  const output = typeof options.output === "string" ? resolve(ROOT_DIR, options.output) : null;
  if (output) {
    mkdirSync(dirname(output), { recursive: true });
    writeFileSync(output, notes, "utf8");
    console.log(`Release notes for ${tag} written to ${output} (${commits.length} commits since ${previousTag ?? "the first commit"})`);
  } else {
    process.stdout.write(notes);
  }
}

main();
