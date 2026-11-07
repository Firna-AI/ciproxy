// The setup that the landing page shows: the permissions of the GitHub App,
// the install command of an own machine, and the workflow change that moves
// a job to ciproxy. The docs show the same change inside its whole job.
// "GitHub App permissions" in docs/protocol/security.md states the
// permissions.

/** A permission that the ciproxy GitHub App asks for. */
export type AppPermission = { name: string; access: string };

export const appPermissions: readonly AppPermission[] = [
  { name: "Self-hosted runners", access: "read and write" },
  { name: "Metadata", access: "read" },
  { name: "Actions", access: "read" },
];

/**
 * The install command, on three lines so that it fits a narrow column. The
 * dashboard shows the command with the token of the workspace, so this page
 * shows "…" in the place of the token.
 */
export const installCommandLines: readonly string[] = [
  "curl -fsSL https://get.ciproxy.dev \\",
  "  | sudo sh -s -- install \\",
  "    --token …",
];

/** The workflow file of the example. */
export const workflowFile = ".github/workflows/ci.yml";

/**
 * The one line that changes, with the lines around it. The first two
 * characters of a line are the diff column: "- ", "+ ", or two spaces.
 */
export const runsOnDiff: readonly string[] = [
  "  jobs:",
  "    test:",
  "-     runs-on: ubuntu-latest",
  "+     runs-on: ciproxy",
];

/** The lines after the change, without the diff column, to copy. */
export function afterChange(diff: readonly string[]): string {
  return diff
    .filter((line) => !line.startsWith("-"))
    .map((line) => line.slice(2))
    .join("\n");
}

/** The steps of the job of the example, after its runs-on line. */
export const jobSteps: readonly string[] = [
  "      steps:",
  "        - uses: actions/checkout@v4",
  "        - run: cargo test --workspace",
];

/** The runs-on change inside its whole job, as the docs show it. */
const runsOnJobLines: readonly string[] = [...runsOnDiff, ...jobSteps];

/**
 * The props of CodeBlock for the docs: the page shows the change, and Copy
 * gives the whole job after the change, so that the copied YAML is valid.
 */
export const runsOnJob: {
  label: string;
  lines: readonly string[];
  copy: string;
} = {
  label: workflowFile,
  lines: runsOnJobLines,
  copy: afterChange(runsOnJobLines),
};
