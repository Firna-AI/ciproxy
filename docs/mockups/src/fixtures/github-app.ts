// The permissions that the ciproxy GitHub App asks for, as the install page
// and the landing page list them. The answer A to item 32 adds "Actions",
// read, so that ciproxy reads the branch and the event of each run from
// GitHub.

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
    use: "to read the branch and the event of each run",
  },
  {
    name: "Members",
    access: "read",
    use: "see who belongs to your organization, so that ciproxy can give roles",
  },
];
