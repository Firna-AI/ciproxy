// Setup without its Boat step: the GitHub App, a machine, and the first
// job. The page reads Setup again while it waits, and a failed read keeps the
// last state with Try again. A machine that joins while the page is open
// keeps its step open, with the machine, until the person continues.

import { useEffect, useState } from "react";

import { ActionButton } from "../../components/actions.tsx";
import { Notice } from "../../components/notice.tsx";
import {
  PageHeader,
  ReadFailure,
  WaitingLine,
} from "../../components/page-state.tsx";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { setupModel } from "../../view/setup-steps.ts";
import {
  WorkspacePageView,
  type WorkspaceContext,
} from "../workspace-page.tsx";
import { Steps } from "./setup-list.tsx";

function Setup({ context }: { context: WorkspaceContext }) {
  const services = useServices();
  const store = services.setup(context.workspace.id);
  const snapshot = useValue(store.state);
  const [sawWaiting, setSawWaiting] = useState(false);
  const [continued, setContinued] = useState(false);
  useEffect(() => {
    void store.load();
    return store.watch();
  }, [store]);
  const waiting =
    snapshot.status === "ready" &&
    !snapshot.machines.some((machine) => machine.kvm);
  useEffect(() => {
    if (waiting) setSawWaiting(true);
  }, [waiting]);

  const title = "Set up ciproxy";
  if (snapshot.status === "loading") {
    return (
      <>
        <PageHeader title={title} />
        <WaitingLine>Loading Setup…</WaitingLine>
      </>
    );
  }
  if (snapshot.status === "failed") {
    return (
      <>
        <PageHeader title={title} />
        <ReadFailure subject="Setup" onRetry={() => void store.load()} />
      </>
    );
  }
  const hold = sawWaiting && !waiting && !continued;
  const model = setupModel(snapshot.setup, snapshot.machines, hold);
  return (
    <>
      <PageHeader
        title={title}
        subtitle={`${model.done} of ${model.total} steps done for ${context.workspace.name}`}
      />
      {snapshot.refreshFailure ? (
        <Notice
          tone="warning"
          title="Unable to refresh Setup"
          actions={
            <ActionButton
              label="Try again"
              size="sm"
              onPress={() => void store.load()}
            />
          }
        >
          Setup shows the last state that it read, and tries again by itself.
        </Notice>
      ) : null}
      <Steps
        model={model}
        context={context}
        onContinue={hold ? () => setContinued(true) : null}
      />
    </>
  );
}

export function SetupScreen() {
  return (
    <WorkspacePageView page="setup">
      {(context) => <Setup context={context} />}
    </WorkspacePageView>
  );
}
