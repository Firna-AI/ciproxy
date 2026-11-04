// The setup that the landing page shows: the permissions of the GitHub App,
// the install command of an own machine, and the workflow change that moves
// a job to ciproxy. "GitHub App permissions" in docs/protocol/security.md
// states the permissions.

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
