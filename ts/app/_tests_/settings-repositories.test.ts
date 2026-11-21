import assert from "node:assert/strict";
import test from "node:test";

import type { RepositoryView } from "../src/api/settings-schema.ts";
import {
  accessDraft,
  accessTexts,
  changed,
  selectionOf,
  toggled,
} from "../src/view/settings-repositories.ts";
import { linked } from "./settings-support.ts";

const repositories: RepositoryView[] = [
  { id: "7001", full_name: "mokly-ai/mokly", private: true, allowed: true },
  { id: "7002", full_name: "mokly-ai/infra", private: true, allowed: false },
  { id: "7003", full_name: "mokly-ai/site", private: false, allowed: false },
  { id: "7004", full_name: "mokly-ai/tools", private: true, allowed: true },
];

test("a switch from all to selected starts with every private repository", () => {
  const organization = linked(1, "mokly-ai", {
    access: { mode: "all_private" },
  });
  const draft = accessDraft(organization, repositories);
  assert.equal(changed(organization, draft, repositories), false);
  const selected = { ...draft, mode: "selected" as const };
  assert.deepEqual(selectionOf(selected, repositories), {
    mode: "selected",
    ids: ["7001", "7002", "7004"],
  });
  assert.equal(changed(organization, selected, repositories), true);
});

test("the selected repositories save in list order, and public ones never", () => {
  const organization = linked(1, "mokly-ai", {
    access: { mode: "selected", ids: ["7004", "7001"] },
  });
  const draft = accessDraft(organization, repositories);
  assert.equal(changed(organization, draft, repositories), false);
  const next = toggled(toggled(draft, "7003"), "7004");
  assert.deepEqual(selectionOf(next, repositories), {
    mode: "selected",
    ids: ["7001"],
  });
  assert.equal(changed(organization, next, repositories), true);
  assert.deepEqual(
    selectionOf({ ...next, mode: "all_private" }, repositories),
    { mode: "all_private" },
  );
});

test("the access texts count the private repositories and name the public ones", () => {
  const organization = linked(1, "mokly-ai", {
    access: { mode: "selected", ids: ["7001"] },
  });
  assert.deepEqual(
    accessTexts(
      "mokly-ai",
      accessDraft(organization, repositories),
      repositories,
    ),
    {
      allBody: "3 repositories, and new ones as you add them",
      selectedBody: "1 of 3 repositories",
      publicNote:
        "ciproxy runs jobs from private repositories. The public repository of mokly-ai keeps its current runners.",
    },
  );
  const privates = repositories.filter((repo) => repo.private);
  assert.equal(
    accessTexts("mokly-ai", accessDraft(organization, privates), privates)
      .publicNote,
    null,
  );
});
