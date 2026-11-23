// The dialog before an Admin gives a person the Admin role. The Admin role
// alone adds machines and connects Boat, which run the jobs of every
// organization of the workspace with their secrets, as the answer to item 50
// says. So the dialog names that trust, and that an organization whose
// runners other people control uses a workspace of its own. It also names
// the power of each Admin to accept new versions of the terms for the
// customer that holds the workspace.

import { LinkAction, TextLink } from "../../components/actions.tsx";
import { Dialog } from "../../components/dialog.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import type { Member } from "../../fixtures/members.ts";
import { organizationList } from "../../fixtures/org.ts";

export function GrantAdminDialog({ person }: { person: Member }) {
  return (
    <Dialog
      title={`Give ${person.name} the Admin role?`}
      footer={
        <>
          <LinkAction to={paths.members} tone="ghost">
            Cancel
          </LinkAction>
          <LinkAction to={paths.members} tone="primary">
            Give the Admin role
          </LinkAction>
        </>
      }
    >
      <Notice
        tone="warning"
        title="An Admin controls the runners of every organization"
      >
        An Admin can add machines and connect Boat. They run the jobs of{" "}
        {organizationList("and")}, with their secrets.
      </Notice>
      <p className="small">
        If other people should control the runners of an organization, give it a
        workspace of its own.
      </p>
      <p className="small">
        An Admin also accepts new versions of the ciproxy{" "}
        <TextLink to={paths.terms} newTab>
          Terms
        </TextLink>{" "}
        for the customer that holds this workspace.
      </p>
    </Dialog>
  );
}
