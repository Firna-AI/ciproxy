// The docs that a mockup entry names in `relatedDocs`: the protocol docs that
// hold the requirements that its screens show, then the plan, which holds the
// decisions and the design calls that the docs do not, such as the claims of
// the public site.

const docs = {
  architecture: "docs/architecture.md",
  agent: "docs/protocol/agent.md",
  billing: "docs/protocol/billing.md",
  cache: "docs/protocol/cache.md",
  jobRouting: "docs/protocol/job-routing.md",
  metering: "docs/protocol/metering.md",
  overflow: "docs/protocol/overflow.md",
  security: "docs/protocol/security.md",
} as const;

export type DocKey = keyof typeof docs;

/** The planning record of the MVP foundation. */
const planDoc = "plans/ciproxy-mvp-foundation.md";

/** The `relatedDocs` of an entry: its protocol docs, then the plan. */
export function related(...keys: readonly DocKey[]): readonly string[] {
  return [...keys.map((key) => docs[key]), planDoc];
}
