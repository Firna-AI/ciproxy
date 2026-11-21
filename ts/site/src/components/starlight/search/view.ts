import type { SearchPage, SearchState } from "./model.ts";

export function element<T extends Element>(
  root: ParentNode,
  selector: string,
): T {
  const found = root.querySelector<T>(selector);
  if (!found) throw new Error(`Missing search element: ${selector}`);
  return found;
}

export class SearchView {
  links: HTMLAnchorElement[] = [];
  private pages?: SearchPage[];
  private root: HTMLElement;
  private input: HTMLInputElement;
  private results: HTMLElement;
  private select: (index: number) => void;
  private navigate: (event: MouseEvent) => void;

  constructor(
    root: HTMLElement,
    select: (index: number) => void,
    navigate: (event: MouseEvent) => void,
  ) {
    this.root = root;
    this.input = element(root, "input");
    this.results = element(root, '[role="grid"]');
    this.select = select;
    this.navigate = navigate;
  }

  render(state: SearchState): void {
    const pages = state.status === "ready" ? state.pages : undefined;
    if (pages !== this.pages) {
      this.pages = pages;
      this.build(pages ?? []);
    }
    const count = this.links.length;
    const active = state.status === "ready" ? state.active : -1;
    const selected = this.links[active]?.parentElement;
    for (const [index, link] of this.links.entries()) {
      link.classList.toggle("is-selected", index === active);
      link.parentElement?.setAttribute(
        "aria-selected",
        String(index === active),
      );
    }
    if (selected) this.input.setAttribute("aria-activedescendant", selected.id);
    else this.input.removeAttribute("aria-activedescendant");
    this.input.setAttribute("aria-expanded", String(count > 0));
    this.results.hidden = count === 0;
    this.results.setAttribute("aria-busy", String(state.status === "loading"));
    element<HTMLElement>(this.root, ".ds-keys").hidden = count === 0;
    const messages = {
      empty: "Search the docs.",
      loading: "Searching…",
      error: "Search is unavailable. Try your search again.",
      ready: count
        ? `${count} ${count === 1 ? "result" : "results"} for ${state.query}`
        : `No results for ${state.query}`,
    };
    element(this.root, '[role="status"]').textContent = messages[state.status];
  }

  scrollActive(index: number): void {
    if (index === 0) element<HTMLElement>(this.root, ".ds-body").scrollTop = 0;
    else this.links[index]?.scrollIntoView({ block: "nearest" });
  }

  private build(pages: SearchPage[]): void {
    this.links = [];
    const fragment = document.createDocumentFragment();
    const pageTemplate = element<HTMLTemplateElement>(this.root, "[data-page]");
    const hitTemplate = element<HTMLTemplateElement>(this.root, "[data-hit]");
    for (const [pageIndex, page] of pages.entries()) {
      const group = document.importNode(pageTemplate.content, true);
      const section = element(group, '[role="rowgroup"]');
      const title = element<HTMLElement>(group, "[data-title]");
      title.textContent = page.title;
      title.id = `docs-search-page-${pageIndex}`;
      for (const hit of page.hits) {
        const row = document.importNode(hitTemplate.content, true);
        const index = this.links.length;
        const cell = element(row, '[role="gridcell"]');
        const link = element<HTMLAnchorElement>(row, "a");
        const heading = element<HTMLElement>(row, ".ds-hit-heading");
        const excerpt = element<HTMLElement>(row, ".ds-hit-excerpt");
        cell.id = `docs-search-hit-${index}`;
        cell.setAttribute("aria-describedby", title.id);
        link.href = hit.url;
        heading.textContent = hit.title;
        // Pagefind escapes the indexed content before it adds its mark tags.
        excerpt.innerHTML = hit.excerpt;
        link.addEventListener("focus", () => this.select(index));
        link.addEventListener("pointermove", () => this.select(index));
        link.addEventListener("click", this.navigate);
        this.links.push(link);
        section.append(row);
      }
      fragment.append(group);
    }
    this.results.replaceChildren(fragment);
  }
}
