// The pages of the shell that later releases fill: Jobs, Machines, Savings,
// Plan and usage, and Settings. Each shows its title and an honest state,
// with no sample data, and leads an Admin back to Setup while it is open.

import { Link } from "expo-router";

import { ActionLink } from "../components/actions.tsx";
import { InfoIcon } from "../components/icons.tsx";
import { PageHeader, StateBlock } from "../components/page-state.tsx";
import { workspaceHref, type WorkspacePage } from "../routing/pages.ts";
import { useValue } from "../services/hooks.ts";
import { useServices } from "../services/context.tsx";
import { pageTitles } from "../view/shell.ts";
import { setupModel } from "../view/setup-steps.ts";
import { WorkspacePageView, type WorkspaceContext } from "./workspace-page.tsx";

function SetupAction({ workspaceId }: { workspaceId: string }) {
  const services = useServices();
  const snapshot = useValue(services.setup(workspaceId).state);
  if (snapshot.status !== "ready") return null;
  if (setupModel(snapshot.setup, snapshot.machines, false).complete)
    return null;
  const href = workspaceHref("setup", workspaceId);
  return (
    <Link href={href} asChild>
      <ActionLink label="Continue setup" href={href} tone="primary" />
    </Link>
  );
}

function ComingSoon({
  page,
  context,
}: {
  page: WorkspacePage;
  context: WorkspaceContext;
}) {
  return (
    <>
      <PageHeader title={pageTitles[page]} />
      <StateBlock
        icon={InfoIcon}
        title="This page is coming soon"
        action={
          context.role === "admin" ? (
            <SetupAction workspaceId={context.workspace.id} />
          ) : null
        }
      >
        ciproxy keeps running your jobs in the meantime.
      </StateBlock>
    </>
  );
}

export function ComingSoonScreen({ page }: { page: WorkspacePage }) {
  return (
    <WorkspacePageView page={page}>
      {(context) => <ComingSoon page={page} context={context} />}
    </WorkspacePageView>
  );
}
