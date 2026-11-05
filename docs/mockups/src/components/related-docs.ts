// The docs that a mockup entry names in `relatedDocs`: the protocol docs that
// hold the requirements that its screens show, then the plan, which holds the
// decisions and the design calls that the docs do not, such as the claims of
// the public site.

const docs = {
  architecture: "docs/architecture.md",
  agent: "docs/protocol/agent.md",
  appRemoval: "docs/protocol/app-removal.md",
  billing: "docs/protocol/billing.md",
  cache: "docs/protocol/cache.md",
  jobRouting: "docs/protocol/job-routing.md",
  labels: "docs/protocol/labels.md",
  metering: "docs/protocol/metering.md",
  overflow: "docs/protocol/overflow.md",
  security: "docs/protocol/security.md",
} as const;

export type DocKey = keyof typeof docs;

/** The planning record of the MVP foundation. */
const planDoc = "plans/ciproxy-mvp-foundation.md";

/** The MVP plan, which holds the design calls from item 30 on. */
const mvpPlanDoc = "plans/ciproxy-mvp.md";

/** The `relatedDocs` of an entry: its protocol docs, then the plan. */
export function related(...keys: readonly DocKey[]): readonly string[] {
  return [...keys.map((key) => docs[key]), planDoc];
}

/**
 * The `relatedDocs` of an entry that shows a design call of the MVP plan: its
 * protocol docs, then that plan, then the plan of the foundation.
 */
export function relatedMvp(...keys: readonly DocKey[]): readonly string[] {
  return [...keys.map((key) => docs[key]), mvpPlanDoc, planDoc];
}
