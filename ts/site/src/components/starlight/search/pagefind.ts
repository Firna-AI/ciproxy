import type { SearchHit, SearchPage } from "./model.ts";

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function hit(value: unknown): SearchHit {
  if (
    !record(value) ||
    typeof value.title !== "string" ||
    typeof value.url !== "string" ||
    typeof value.excerpt !== "string"
  ) {
    throw new Error("Invalid search heading");
  }
  return { title: value.title, url: value.url, excerpt: value.excerpt };
}

function page(value: unknown): SearchPage {
  if (
    !record(value) ||
    !record(value.meta) ||
    typeof value.meta.title !== "string" ||
    !Array.isArray(value.sub_results)
  ) {
    throw new Error("Invalid search page");
  }
  return { title: value.meta.title, hits: value.sub_results.map(hit) };
}

/** Load the build's index only when a reader searches. */
export async function searchPagefind(query: string): Promise<SearchPage[]> {
  const bundle = `${import.meta.env.BASE_URL}pagefind/pagefind.js`;
  const api: unknown = await import(/* @vite-ignore */ bundle);
  if (
    !record(api) ||
    typeof api.search !== "function" ||
    typeof api.options !== "function"
  ) {
    throw new Error("Search index unavailable");
  }
  await api.options({ excerptLength: 15 });
  const response: unknown = await api.search(query);
  if (!record(response) || !Array.isArray(response.results)) {
    throw new Error("Invalid search response");
  }
  return Promise.all(
    response.results.map(async (result: unknown) => {
      if (!record(result) || typeof result.data !== "function") {
        throw new Error("Invalid search result");
      }
      const data: unknown = await result.data();
      return page(data);
    }),
  );
}
