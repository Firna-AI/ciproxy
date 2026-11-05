// The dialogs of the Organizations tab. A new workspace takes the name of its
// first organization, and an Admin can rename it; the GitHub organizations
// keep their names. A take-out removes the GitHub App from the organization,
// as the answer D to item 51 says: no new job of it starts on ciproxy, its
// running jobs finish, ciproxy deletes its scale sets and then removes the
// App, its jobs stay in the job history, and it can join a workspace again
// with a new install.

import { LinkAction } from "../../components/actions.tsx";
import { Dialog } from "../../components/dialog.tsx";
import { TextField } from "../../components/fields.tsx";
import { paths } from "../../components/paths.ts";
import { labsOrg, newWorkspaceName } from "../../fixtures/org.ts";

/** Cancel and the action of a dialog, which both lead back to the tab. */
function Footer({
  action,
  tone,
}: {
  action: string;
  tone: "primary" | "danger";
}) {
  return (
    <>
      <LinkAction to={paths.organizations} tone="ghost">
        Cancel
      </LinkAction>
      <LinkAction to={paths.organizations} tone={tone}>
        {action}
      </LinkAction>
    </>
  );
}

export function RenameWorkspaceDialog() {
  return (
    <Dialog
      title="Rename the workspace"
      footer={<Footer action="Save" tone="primary" />}
    >
      <TextField
        label="Name"
        value={newWorkspaceName}
        hint="The GitHub organizations keep their names."
      />
    </Dialog>
  );
}

export function TakeOutDialog() {
  const login = <span className="nowrap">{labsOrg.login}</span>;
  return (
    <Dialog
      title={`Take ${labsOrg.login} out of this workspace?`}
      footer={<Footer action="Take out" tone="danger" />}
    >
      <p className="small">
        No new job of {login} starts on ciproxy, and its running jobs finish.
        ciproxy removes the GitHub App from {login}.
      </p>
      <p className="small">
        Its jobs stay in the job history, and {login} can join a workspace again
        later with a new install.
      </p>
    </Dialog>
  );
}
