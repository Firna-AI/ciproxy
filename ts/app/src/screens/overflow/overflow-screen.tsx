// Settings under Overflow, for Admins: the switch and the wait, which hold
// for every provider, and a panel for each provider with its account, its
// own monthly spend limit, and the spend of the month. The page reads the
// settings at each visit. A change that the API accepted is never sent
// again: when the read after it fails, the page names the change and offers
// only Try again, which reads. Each workspace gets a fresh page, so no typed
// key or limit passes to another workspace. The tabs are those of Settings,
// and Members counts the open requests from the settings of the workspace,
// which this page also reads at each visit.

import { useEffect } from "react";

import { ActionButton } from "../../components/actions.tsx";
import { AlertIcon } from "../../components/icons.tsx";
import {
  PageHeader,
  ReadFailure,
  StateBlock,
} from "../../components/page-state.tsx";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import type { AcceptedChange } from "../../state/overflow.ts";
import { settingsServices } from "../../state/settings-services.ts";
import { providerName } from "../../view/providers.ts";
import { SettingsTabs } from "../settings/settings-tabs.tsx";
import {
  WorkspacePageView,
  type WorkspaceContext,
} from "../workspace-page.tsx";
import { OverflowLoading } from "./overflow-loading.tsx";
import { OverflowSettingsView } from "./overflow-settings.tsx";

function acceptedText(change: AcceptedChange): string {
  switch (change.kind) {
    case "connected":
      return `${providerName(change.provider)} connected.`;
    case "disconnected":
      return `${providerName(change.provider)} disconnected.`;
    case "saved":
      return "The change is saved.";
  }
}

function OverflowBody({ workspaceId }: { workspaceId: string }) {
  const store = useServices().overflow(workspaceId);
  const snapshot = useValue(store.state);
  useEffect(() => {
    void store.load();
  }, [store]);
  const retry = () => void store.load();
  if (snapshot.status === "loading") return <OverflowLoading />;
  if (snapshot.status === "failed") {
    if (snapshot.accepted === null) {
      return <ReadFailure subject="Overflow" onRetry={retry} />;
    }
    return (
      <StateBlock
        icon={AlertIcon}
        danger
        title="Unable to load Overflow"
        action={
          <ActionButton label="Try again" tone="primary" onPress={retry} />
        }
      >
        {`${acceptedText(snapshot.accepted)} Check the connection, then try again.`}
      </StateBlock>
    );
  }
  return (
    <OverflowSettingsView
      workspaceId={workspaceId}
      store={store}
      settings={snapshot.settings}
      refreshFailure={snapshot.refreshFailure}
    />
  );
}

/**
 * The tabs of Settings with the open requests of the settings read. While
 * that read loads or after it failed, Members shows no count.
 */
function OverflowTabs({ workspaceId }: { workspaceId: string }) {
  const store = settingsServices(useServices()).settings(workspaceId);
  const snapshot = useValue(store.state);
  useEffect(() => {
    void store.load();
  }, [store]);
  return (
    <SettingsTabs
      workspaceId={workspaceId}
      active="overflow"
      openRequests={
        snapshot.status === "ready" ? snapshot.value.open_requests : null
      }
    />
  );
}

function Overflow({ context }: { context: WorkspaceContext }) {
  const workspaceId = context.workspace.id;
  return (
    <>
      <PageHeader title="Settings" />
      <OverflowTabs workspaceId={workspaceId} />
      <OverflowBody key={workspaceId} workspaceId={workspaceId} />
    </>
  );
}

export function OverflowScreen() {
  return (
    <WorkspacePageView page="settings">
      {(context) => <Overflow context={context} />}
    </WorkspacePageView>
  );
}
