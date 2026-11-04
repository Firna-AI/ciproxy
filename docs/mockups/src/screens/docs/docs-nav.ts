// The sidebar of the docs, in the order of the Starlight sidebar config. An
// item with a path has a page in the catalogue; the other items are planned
// pages that show in the sidebar only.

import { paths, type CataloguePath } from "../../components/paths.ts";

export type DocsNavItem = { label: string; to?: CataloguePath };

export type DocsNavGroup = { title: string; items: readonly DocsNavItem[] };

export const docsNav: readonly DocsNavGroup[] = [
  {
    title: "Start here",
    items: [
      { label: "Overview", to: paths.docsHome },
      { label: "Quickstart", to: paths.quickstart },
      { label: "How ciproxy works" },
    ],
  },
  {
    title: "Guides",
    items: [
      { label: "Add a machine" },
      { label: "Connect Boat" },
      { label: "Move your workflows" },
      { label: "Set a spend limit" },
      { label: "Remove a machine" },
    ],
  },
  {
    title: "Reference",
    items: [
      { label: "Labels and VM sizes", to: paths.labelsReference },
      { label: "Agent install options" },
      { label: "GitHub App permissions" },
      { label: "Plans and starts" },
    ],
  },
  {
    title: "Security",
    items: [{ label: "Job isolation" }, { label: "The agent connection" }],
  },
];

/** The pages before and after a page, in sidebar order. */
export function docsNeighbours(page: CataloguePath): {
  prev?: DocsNavItem;
  next?: DocsNavItem;
} {
  const items = docsNav.flatMap((group) => group.items);
  const index = items.findIndex((item) => item.to === page);
  if (index < 0) return {};
  return { prev: items[index - 1], next: items[index + 1] };
}
