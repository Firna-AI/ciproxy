import { deepStrictEqual, strictEqual } from "node:assert";
import { test } from "node:test";

import {
  SearchModel,
  type SearchPage,
} from "../src/components/starlight/search/model.ts";

const pages: SearchPage[] = [
  {
    title: "Quickstart",
    hits: [
      { title: "Change runs-on", url: "/quickstart/#runs-on", excerpt: "job" },
    ],
  },
  {
    title: "Labels",
    hits: [
      { title: "Labels", url: "/labels/", excerpt: "label" },
      { title: "Default labels", url: "/labels/#default", excerpt: "size" },
    ],
  },
];

test("search counts heading hits across page groups and selects them in order", async () => {
  const model = new SearchModel(
    async () => pages,
    () => {},
  );
  await model.search("runs-on");
  strictEqual(model.hits.length, 3);
  deepStrictEqual(model.state, {
    status: "ready",
    query: "runs-on",
    pages,
    active: 0,
  });
  model.move(1);
  model.move(1);
  strictEqual(model.state.status === "ready" && model.state.active, 2);
  strictEqual(model.hits[2]?.url, "/labels/#default");
  model.move(1);
  strictEqual(model.state.status === "ready" && model.state.active, 0);
  model.move(-1);
  strictEqual(model.state.status === "ready" && model.state.active, 2);
  model.select(1);
  strictEqual(model.state.status === "ready" && model.state.active, 1);
});

test("a later query and clear discard unfinished search results", async () => {
  const first = Promise.withResolvers<SearchPage[]>();
  const second = Promise.withResolvers<SearchPage[]>();
  const changes: string[] = [];
  const model = new SearchModel(
    (query) => (query === "first" ? first.promise : second.promise),
    (state) => changes.push(state.status),
  );
  const oldSearch = model.search("first");
  const newSearch = model.search("second");
  second.resolve(pages);
  await newSearch;
  first.resolve([]);
  await oldSearch;
  strictEqual(model.state.query, "second");
  strictEqual(model.hits.length, 3);
  deepStrictEqual(changes, ["loading", "loading", "ready"]);

  const unfinished = Promise.withResolvers<SearchPage[]>();
  const clearing = new SearchModel(
    () => unfinished.promise,
    () => {},
  );
  const pending = clearing.search("runs-on");
  await clearing.search("  ");
  unfinished.resolve(pages);
  await pending;
  deepStrictEqual(clearing.state, { status: "empty", query: "" });
  deepStrictEqual(clearing.hits, []);
});

test("no matches and failed searches leave no active result and can recover", async () => {
  const model = new SearchModel(
    async (query) => {
      if (query === "error") throw new Error("index unavailable");
      return query === "match" ? pages : [];
    },
    () => {},
  );
  await model.search("missing");
  model.move(1);
  deepStrictEqual(model.state, {
    status: "ready",
    query: "missing",
    pages: [],
    active: -1,
  });
  await model.search("error");
  deepStrictEqual(model.state, { status: "error", query: "error" });
  deepStrictEqual(model.hits, []);
  await model.search("match");
  strictEqual(model.hits.length, 3);
});
