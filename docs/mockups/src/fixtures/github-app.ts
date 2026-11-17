// The permissions that the ciproxy GitHub App asks for, as "GitHub App
// permissions" in docs/protocol/security.md states them. The install page
// lists them, and the landing page counts them. "Actions", read, lets
// ciproxy read the event and the head repository of each run from GitHub.

export type AppPermission = {
  name: string;
  access: string;
  /** What ciproxy uses the permission for. */
  use: string;
};

export const appPermissions: readonly AppPermission[] = [
  {
    name: "Self-hosted runners",
    access: "read and write",
    use: "to start a runner for each job",
  },
  { name: "Metadata", access: "read", use: "to list your repositories" },
  {
    name: "Actions",
    access: "read",
    use: "to read the event and the head repository of each run",
  },
  {
    name: "Members",
    access: "read",
    use: "see who belongs to your organization, so that ciproxy can give roles",
  },
];
