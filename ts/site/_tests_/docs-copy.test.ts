import { deepStrictEqual, doesNotThrow } from "node:assert";
import { after, before, describe, test } from "node:test";

import { load } from "js-yaml";

import { buildSite, copiedAfterHeading, type BuiltSite } from "./built-site.ts";

/** The docs pages whose "Change runs-on" section moves a job to ciproxy. */
const examples = [
  { page: "Quickstart", route: "/docs/quickstart/" },
  { page: "Move your workflows", route: "/docs/guides/move-your-workflows/" },
];

/** The job after the change, as the reader pastes it into a workflow. */
const migratedJob = {
  jobs: {
    test: {
      "runs-on": "ciproxy",
      steps: [
        { uses: "actions/checkout@v4" },
        { run: "cargo test --workspace" },
      ],
    },
  },
};

describe("the runs-on change of the docs", () => {
  let site: BuiltSite | undefined;
  before(
    () => {
      site = buildSite();
    },
    { timeout: 180_000 },
  );
  after(() => site?.remove());

  /** The HTML of a page of the build of this suite. */
  function builtPage(route: string): string {
    if (!site) throw new Error("The site did not build");
    return site.page(route);
  }

  for (const example of examples) {
    test(`${example.page} copies only the job after the change, as valid YAML`, () => {
      const copied = copiedAfterHeading(
        builtPage(example.route),
        "change-runs-on",
      );
      let job: unknown;
      doesNotThrow(() => {
        job = load(copied, { json: false });
      }, "The copied job must be YAML with no duplicate key");
      deepStrictEqual(job, migratedJob);
    });
  }
});
