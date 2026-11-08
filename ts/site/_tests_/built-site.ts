// A fresh production build of the site, for the tests that read the pages as
// a browser gets them. Each build goes into its own temporary folder, so a
// stale `dist/` can never answer a test.

import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const siteRoot = fileURLToPath(new URL("..", import.meta.url));

export type BuiltSite = {
  /** The HTML of the page at a route, such as "/docs/quickstart/". */
  page(route: string): string;
  /** Deletes the build. */
  remove(): void;
};

/** The command-line program of Astro, from the `bin` field of its package. */
function astroCli(): string {
  const manifest = createRequire(import.meta.url).resolve("astro/package.json");
  const parsed: unknown = JSON.parse(readFileSync(manifest, "utf8"));
  const bin =
    typeof parsed === "object" && parsed !== null && "bin" in parsed
      ? parsed.bin
      : undefined;
  const cli =
    typeof bin === "object" && bin !== null && "astro" in bin
      ? bin.astro
      : undefined;
  if (typeof cli !== "string") throw new Error(`No astro bin in ${manifest}`);
  return join(dirname(manifest), cli);
}

/** Builds the site with `astro build`, as the deployment does. */
export function buildSite(): BuiltSite {
  const outDir = mkdtempSync(join(tmpdir(), "ciproxy-site-"));
  const remove = () => rmSync(outDir, { recursive: true, force: true });
  const build = spawnSync(
    process.execPath,
    [astroCli(), "build", "--outDir", outDir],
    {
      cwd: siteRoot,
      encoding: "utf8",
      env: { ...process.env, ASTRO_TELEMETRY_DISABLED: "1" },
    },
  );
  if (build.status !== 0) {
    remove();
    throw new Error(`astro build failed:\n${build.stdout}${build.stderr}`);
  }
  return {
    page: (route) => readFileSync(join(outDir, route, "index.html"), "utf8"),
    remove,
  };
}

const namedReferences: Readonly<Record<string, string>> = {
  amp: "&",
  apos: "'",
  gt: ">",
  lt: "<",
  quot: '"',
};

/** An attribute value with its character references decoded. */
function decodeAttribute(value: string): string {
  return value.replace(
    /&(#x[0-9a-f]+|#[0-9]+|[a-z]+);/gi,
    (reference: string, name: string) => {
      if (name.startsWith("#x") || name.startsWith("#X")) {
        return String.fromCodePoint(Number.parseInt(name.slice(2), 16));
      }
      if (name.startsWith("#")) {
        return String.fromCodePoint(Number.parseInt(name.slice(1), 10));
      }
      return namedReferences[name.toLowerCase()] ?? reference;
    },
  );
}

/**
 * The text that the first Copy button after a heading puts on the clipboard.
 * A CodeBlock button holds the text in `data-copy`. An Expressive Code button
 * holds it in `data-code`, with U+007F for each line break, which its script
 * turns back into a line break.
 */
export function copiedAfterHeading(html: string, headingId: string): string {
  const heading = html.indexOf(` id="${headingId}"`);
  if (heading < 0) throw new Error(`No heading with the id "${headingId}"`);
  const button = /<button\b[^>]*?\sdata-(copy|code)="([^"]*)"/.exec(
    html.slice(heading),
  );
  if (!button) throw new Error(`No Copy button after "${headingId}"`);
  const [, kind, value = ""] = button;
  const text = decodeAttribute(value);
  return kind === "code" ? text.replaceAll("\u007f", "\n") : text;
}
