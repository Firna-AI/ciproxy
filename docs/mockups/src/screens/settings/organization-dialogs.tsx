// The dialogs of the Organizations tab. A new workspace takes the name of its
// first organization, and an Admin can rename it; the GitHub organizations
// keep their names. A take-out removes the GitHub App from the organization,
// as the answer D to item 51 says: no new job of it starts on ciproxy, its
// running jobs finish, ciproxy deletes its scale sets and then removes the
// App, its jobs stay in the job history, and it can join a workspace again
// with a new install. Turn on asks first, because the organization then uses
// the machines, the labels, the settings, and the Boat account.

import { LinkAction } from "../../components/actions.tsx";
import { Dialog } from "../../components/dialog.tsx";
import { TextField } from "../../components/fields.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { formerAdmin, studioOrg } from "../../fixtures/org-links.ts";
import { labsOrg, newWorkspaceName } from "../../fixtures/org.ts";

/** Cancel and the action of a dialog, which lead back to the tab. */
function Footer({
  action,
  tone,
  back = paths.organizations,
  to = paths.organizations,
}: {
  action: string;
  tone: "primary" | "danger";
  /** The screen that Cancel opens. */
  back?: CataloguePath;
  /** The screen that the action opens. */
  to?: CataloguePath;
}) {
  return (
    <>
      <LinkAction to={back} tone="ghost">
        Cancel
      </LinkAction>
      <LinkAction to={to} tone={tone}>
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

/** A current Admin turns on an organization that a former Admin added. */
export function TurnOnDialog() {
  const login = <span className="nowrap">{studioOrg.login}</span>;
  return (
    <Dialog
      title={`Turn on ${studioOrg.login}?`}
      footer={
        <Footer
          action="Turn on"
          tone="primary"
          back={paths.inactiveAdmin}
          to={paths.inactiveAfterTeam}
        />
      }
    >
      <p className="small">
        The jobs of {login} then run on the machines of this workspace, with its
        labels, settings, and Boat account.
      </p>
      <p className="small">
        {formerAdmin.name} added {login} on {studioOrg.joined} and is no longer
        an Admin.
      </p>
    </Dialog>
  );
}
