// The mockup policy check: the rules of the mockups and typescript skills that
// the Mokly build does not enforce. Run after `mokly build`; it reads the
// built catalogue index.
//
// - A page renders at most five screens. A screen and its variants count on
//   the page of the screen's folder; an index screen is its folder's page.
// - A page with sub-pages renders exactly one canonical screen. The top-level
//   folders are catalogue indexes and are exempt.
// - Mockup source files stay at or under 300 lines.
// - The generated output stays untracked and ignored by Git.

import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { basename, join, posix, relative } from "node:path";

const mockups = join(import.meta.dirname, "..");
const repo = join(mockups, "..", "..");
const indexPath = join(
  mockups,
  "mokly-generated",
  "mokly-viewer",
  "index.json",
);

type Screen = { path: string; variantOf?: string; sourcePath: string };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function readScreens(): Screen[] {
  const raw: unknown = JSON.parse(readFileSync(indexPath, "utf8"));
  if (!isRecord(raw) || !Array.isArray(raw.screens)) {
    throw new Error(
      "The catalogue index has no screens. Build the mockups first.",
    );
  }
  return raw.screens.map((entry: unknown) => {
    if (
      !isRecord(entry) ||
      typeof entry.path !== "string" ||
      !isRecord(entry.details)
    ) {
      throw new Error(
        "A screen of the catalogue index has no path or details.",
      );
    }
    const { sourcePath } = entry.details;
    if (typeof sourcePath !== "string")
      throw new Error(`No source for ${entry.path}.`);
    return {
      path: entry.path,
      sourcePath,
      ...(typeof entry.variantOf === "string"
        ? { variantOf: entry.variantOf }
        : {}),
    };
  });
}

function pageOf(screen: Screen, byPath: Map<string, Screen>): string {
  const base = screen.variantOf ? byPath.get(screen.variantOf) : screen;
  if (!base) throw new Error(`Variant without parent: ${screen.path}`);
  const isIndex = /^index\.mockup\.tsx?$/.test(basename(base.sourcePath));
  return isIndex ? base.path : posix.dirname(base.path);
}

function ancestors(path: string): string[] {
  const parts = path.split("/");
  return parts.slice(1).map((_, i) => parts.slice(0, i + 1).join("/"));
}

function pageViolations(screens: Screen[]): string[] {
  const byPath = new Map(screens.map((screen) => [screen.path, screen]));
  const counts = new Map<string, number>();
  for (const screen of screens) {
    const page = pageOf(screen, byPath);
    counts.set(page, (counts.get(page) ?? 0) + 1);
  }
  const pages = new Set(
    [...counts.keys()].flatMap((page) => [page, ...ancestors(page)]),
  );
  const violations: string[] = [];
  for (const page of [...pages].sort()) {
    const count = counts.get(page) ?? 0;
    const hasSubPages = [...pages].some(
      (other) => posix.dirname(other) === page,
    );
    const topLevel = !page.includes("/");
    if (count > 5)
      violations.push(
        `${page}: ${count} screens; split it into sub-pages of at most 5.`,
      );
    if (hasSubPages && !topLevel && count !== 1) {
      violations.push(
        `${page}: has sub-pages, so it needs exactly 1 canonical screen, not ${count}.`,
      );
    }
  }
  return violations;
}

function sourceFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory())
      return entry.name === "mokly-generated" ? [] : sourceFiles(path);
    return /\.(ts|tsx|css)$/.test(entry.name) ? [path] : [];
  });
}

function lengthViolations(): string[] {
  return sourceFiles(mockups).flatMap((file) => {
    const lines = readFileSync(file, "utf8").split("\n").length - 1;
    return lines > 300
      ? [`${relative(repo, file)}: ${lines} lines; split it below 300.`]
      : [];
  });
}

function gitViolations(): string[] {
  const git = (args: string[]) =>
    execFileSync("git", args, { cwd: repo, encoding: "utf8" });
  const violations: string[] = [];
  if (git(["ls-files", "--", "docs/mockups/mokly-generated"]).trim()) {
    violations.push(
      "docs/mockups/mokly-generated is tracked; remove it from Git.",
    );
  }
  for (const probe of [
    "docs/mockups/mokly-generated/index.html",
    "docs/mockups/.mokly-export-reservations/probe",
  ]) {
    try {
      git(["check-ignore", "--quiet", probe]);
    } catch {
      violations.push(
        `${probe} is not ignored; add its directory to .gitignore.`,
      );
    }
  }
  return violations;
}

const screens = readScreens();
const violations = [
  ...pageViolations(screens),
  ...lengthViolations(),
  ...gitViolations(),
];
if (violations.length > 0) {
  console.error(
    `Mockup policy failed:\n${violations.map((line) => `  - ${line}`).join("\n")}`,
  );
  process.exit(1);
}
console.log(`Mockup policy passed: ${screens.length} screens.`);
