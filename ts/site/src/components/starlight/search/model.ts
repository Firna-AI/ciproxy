export type SearchHit = { title: string; url: string; excerpt: string };
export type SearchPage = { title: string; hits: SearchHit[] };
export type SearchSource = (query: string) => Promise<SearchPage[]>;
export type SearchState = { query: string } & (
  | { status: "empty" | "loading" | "error" }
  | { status: "ready"; pages: SearchPage[]; active: number }
);

export class SearchModel {
  state: SearchState = { status: "empty", query: "" };
  private source: SearchSource;
  private changed: (state: SearchState) => void;
  private generation = 0;

  constructor(source: SearchSource, changed: (state: SearchState) => void) {
    this.source = source;
    this.changed = changed;
  }

  get hits(): SearchHit[] {
    return this.state.status === "ready"
      ? this.state.pages.flatMap((page) => page.hits)
      : [];
  }

  async search(rawQuery: string): Promise<void> {
    const query = rawQuery.trim();
    const generation = ++this.generation;
    this.state = { status: query ? "loading" : "empty", query };
    this.changed(this.state);
    if (!query) return;

    try {
      const pages = await this.source(query);
      if (generation !== this.generation) return;
      this.state = {
        status: "ready",
        query,
        pages,
        active: pages.some((page) => page.hits.length) ? 0 : -1,
      };
    } catch {
      if (generation !== this.generation) return;
      this.state = { status: "error", query };
    }
    this.changed(this.state);
  }

  move(direction: -1 | 1): void {
    if (this.state.status !== "ready" || !this.hits.length) return;
    this.select(
      (this.state.active + direction + this.hits.length) % this.hits.length,
    );
  }

  select(index: number): void {
    if (this.state.status !== "ready" || !this.hits[index]) return;
    this.state = { ...this.state, active: index };
    this.changed(this.state);
  }
}
