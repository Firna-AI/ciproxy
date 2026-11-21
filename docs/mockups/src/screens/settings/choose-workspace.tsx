// Choose a workspace: the page of the return from GitHub after Add an
// organization. GitHub picks the organization, and ciproxy checks the install
// with the person's own GitHub sign-in before it lists anything. With one
// workspace where the person is an Admin, the organization links at once and
// this page does not show. With several, the page lists them with none
// picked, and the button names the organization and then the workspace. On a
// workspace whose plan includes no more organizations, the new one joins
// inactive. A request to the owners waits: after the approval, the person who
// asked chooses at the next sign-in. With no Admin workspace, the install
// waits for that person. A failed check offers a new GitHub sign-in.

import type { ReactNode } from "react";
import { Github } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { RadioCard, RadioCardGroup } from "@firna/ui/radio";

import { Action, LinkAction, noop } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import {
  adminWorkspaces,
  joinsInactive,
  noAdminRequester,
  pickedWorkspace,
} from "../../fixtures/org-links.ts";
import { labsOrg, user, type WorkspaceChoice } from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import { AuthShell } from "../auth/auth-shell.tsx";

export const chooseWorkspaceStates = [
  "returned",
  "approved",
  "request-wait",
  "not-confirmed",
  "no-admin-workspace",
] as const;
export type ChooseWorkspaceState = (typeof chooseWorkspaceStates)[number];

/** The organization that GitHub installed, or that the request names. */
function Login() {
  return <span className="nowrap">{labsOrg.login}</span>;
}

/** On a full plan, that the new organization stays inactive until Team. */
function choiceNote(choice: WorkspaceChoice): string | undefined {
  return joinsInactive(choice)
    ? `${labsOrg.login} stays inactive here until the workspace moves to ${tiers.team.name}.`
    : undefined;
}

function Heading({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <div className="stack-tight">
      <h1 className="auth-title">{title}</h1>
      {children ? <p className="auth-lead">{children}</p> : null}
    </div>
  );
}

/** The workspaces where the person is an Admin. None is picked at first. */
function WorkspacePicker({ picked }: { picked?: WorkspaceChoice }) {
  return (
    <>
      <RadioCardGroup label="Workspaces where you are an Admin">
        {adminWorkspaces.map((choice) => (
          <RadioCard
            key={choice.name}
            title={choice.name}
            body={choiceNote(choice)}
            checked={choice.name === picked?.name}
            onPress={noop}
          />
        ))}
      </RadioCardGroup>
      <div className="auth-actions">
        {picked ? (
          <LinkAction to={paths.organizations} tone="primary" size="lg" block>
            {`Add ${labsOrg.login} to ${picked.name}`}
          </LinkAction>
        ) : (
          <Action tone="primary" size="lg" block disabled>
            {`Add ${labsOrg.login}`}
          </Action>
        )}
      </div>
    </>
  );
}

function RequestWait() {
  return (
    <>
      <Heading title={`Waiting for an owner of ${labsOrg.login}`}>
        GitHub sent your request to the owners of <Login />. When one of them
        approves it, sign in to ciproxy again to choose its workspace.
      </Heading>
      <div className="auth-actions">
        <LinkAction to={paths.addApproved} tone="primary" size="lg" block>
          Check again
        </LinkAction>
        <LinkAction
          to={paths.organizationsOne}
          tone="secondary"
          size="lg"
          block
        >
          Back to Organizations
        </LinkAction>
      </div>
    </>
  );
}

function NotConfirmed() {
  return (
    <>
      <Heading title="Add an organization" />
      <Notice tone="danger" title="Unable to confirm the install on GitHub">
        Sign in with GitHub again, so that ciproxy can check the install with
        your account.
      </Notice>
      <LinkAction
        to={paths.addOrganization}
        tone="primary"
        size="lg"
        icon={Github}
        block
      >
        Sign in with GitHub
      </LinkAction>
    </>
  );
}

function NoAdminWorkspace() {
  return (
    <>
      <Heading title={`${labsOrg.login} waits for a workspace`}>
        GitHub installed ciproxy on <Login />. To add it, ask an Admin of your
        workspace for the Admin role, then check again.
      </Heading>
      <div className="auth-actions">
        <Action tone="primary" size="lg" block>
          Check again
        </Action>
      </div>
    </>
  );
}

function Body({ state }: { state: ChooseWorkspaceState }) {
  switch (state) {
    case "returned":
      return (
        <>
          <Heading title={`Choose a workspace for ${labsOrg.login}`}>
            GitHub installed ciproxy on <Login />. Pick the workspace that runs
            its jobs.
          </Heading>
          <WorkspacePicker />
        </>
      );
    case "approved":
      return (
        <>
          <Heading title={`Choose a workspace for ${labsOrg.login}`}>
            An owner of <Login /> approved your request, and GitHub installed
            ciproxy on it. Pick the workspace that runs its jobs.
          </Heading>
          <WorkspacePicker picked={pickedWorkspace} />
        </>
      );
    case "request-wait":
      return <RequestWait />;
    case "not-confirmed":
      return <NotConfirmed />;
    case "no-admin-workspace":
      return <NoAdminWorkspace />;
  }
}

export function ChooseWorkspaceScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: ChooseWorkspaceState;
}) {
  const person = state === "no-admin-workspace" ? noAdminRequester : user;
  return (
    <AuthShell
      viewport={viewport}
      footer={`Signed in as ${person.login} on GitHub.`}
    >
      <Body state={state} />
    </AuthShell>
  );
}
