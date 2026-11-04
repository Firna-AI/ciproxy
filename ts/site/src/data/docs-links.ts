// The docs pages that the marketing pages link to. A link shows only when
// its page exists in the docs collection, so that no page links to a page
// that is not written yet. The address of each page is the one that the docs
// mockups give it.

import { getCollection } from "astro:content";

export type DocsLink = { label: string; href: string };

/** The docs pages that the marketing pages name, by their entry id. */
const docsPages = {
  quickstart: { label: "Quickstart", id: "docs/quickstart" },
  labels: { label: "Labels and VM sizes", id: "docs/reference/labels" },
} as const;

export type DocsPageKey = keyof typeof docsPages;

/** The links of these docs pages whose pages exist, in the given order. */
export async function docsLinks(
  keys: readonly DocsPageKey[],
): Promise<DocsLink[]> {
  const entries = await getCollection("docs");
  const ids = new Set(entries.map((entry) => entry.id));
  return keys.flatMap((key) => {
    const page = docsPages[key];
    return ids.has(page.id)
      ? [{ label: page.label, href: `/${page.id}/` }]
      : [];
  });
}
