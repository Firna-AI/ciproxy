// Settings under Overflow, for Admins: the Boat account, the overflow switch
// and wait, and the monthly spend limit with the spend of the month. The page
// reads the settings at each visit. A change that the API accepted is never
// sent again: when the read after it fails, the page names the change and
// offers only Try again, which reads. Each workspace gets a fresh page, so
// no typed key or limit passes to another workspace.

import { useEffect, useRef, useState } from "react";
import type { View } from "react-native";

import type { ApiFailure } from "../../api/client.ts";
import type { OverflowSettings } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { AlertIcon, CloudOffIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import {
  PageHeader,
  ReadFailure,
  StateBlock,
} from "../../components/page-state.tsx";
import { Panel } from "../../components/panel.tsx";
import { SettingsTabs } from "../../components/settings-tabs.tsx";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import type { AcceptedChange, OverflowStore } from "../../state/overflow.ts";
import { activeAccount, overflowNotice } from "../../view/boat.ts";
import {
  WorkspacePageView,
  type WorkspaceContext,
} from "../workspace-page.tsx";
import { BoatPanel } from "./boat-panel.tsx";
import { DisconnectDialog } from "./disconnect-dialog.tsx";
import { OverflowLoading } from "./overflow-loading.tsx";
import { SpendPanel } from "./spend-panel.tsx";
import { usePolicy } from "./use-policy.ts";
import { WaitPanel } from "./wait-panel.tsx";

const acceptedText: Readonly<Record<AcceptedChange, string>> = {
  connected: "Boat connected.",
  disconnected: "Boat disconnected.",
  saved: "The change is saved.",
};

function retry(store: OverflowStore) {
  return () => void store.load();
}

function Policy(props: {
  store: OverflowStore;
  settings: OverflowSettings;
  limitCents: string;
  spend: boolean;
}) {
  const editor = usePolicy(props.store, props.settings, props.limitCents);
  return (
    <>
      <WaitPanel settings={props.settings} editor={editor} />
      {props.spend ? (
        <SpendPanel
          settings={props.settings}
          limitCents={props.limitCents}
          editor={editor}
          onRetry={retry(props.store)}
        />
      ) : null}
    </>
  );
}

function Settings(props: {
  store: OverflowStore;
  settings: OverflowSettings;
  refreshFailure: ApiFailure | null;
}) {
  const { store, settings } = props;
  const title = useRef<View>(null);
  const [disconnecting, setDisconnecting] = useState(false);
  const [moved, setMoved] = useState(0);
  useEffect(() => {
    if (moved > 0) title.current?.focus();
  }, [moved]);
  if (!settings.available) {
    return (
      <Panel title="Boat">
        <StateBlock
          icon={CloudOffIcon}
          title="Boat is unavailable right now"
          action={
            <ActionButton
              label="Try again"
              tone="primary"
              onPress={retry(store)}
            />
          }
        >
          Jobs run on your machines only.
        </StateBlock>
      </Panel>
    );
  }
  const notice = overflowNotice(settings);
  const account = settings.account ?? null;
  const limit = settings.spend_limit_cents ?? null;
  return (
    <>
      {props.refreshFailure ? (
        <Notice
          tone="warning"
          title="Unable to refresh Overflow"
          actions={
            <ActionButton label="Try again" size="sm" onPress={retry(store)} />
          }
        >
          The page shows the last settings that it read.
        </Notice>
      ) : null}
      {notice ? (
        <Notice tone={notice.tone} title={notice.title}>
          {notice.body}
        </Notice>
      ) : null}
      <BoatPanel
        store={store}
        settings={settings}
        titleRef={title}
        onChanged={() => setMoved((count) => count + 1)}
        onDisconnect={() => setDisconnecting(true)}
      />
      {account !== null && limit !== null ? (
        <Policy
          store={store}
          settings={settings}
          limitCents={limit}
          spend={activeAccount(settings) !== null}
        />
      ) : null}
      {disconnecting ? (
        <DisconnectDialog
          store={store}
          onClose={() => setDisconnecting(false)}
          onDisconnected={() => {
            setDisconnecting(false);
            setMoved((count) => count + 1);
          }}
        />
      ) : null}
    </>
  );
}

function OverflowBody({ workspaceId }: { workspaceId: string }) {
  const store = useServices().overflow(workspaceId);
  const snapshot = useValue(store.state);
  useEffect(() => {
    void store.load();
  }, [store]);
  if (snapshot.status === "loading") return <OverflowLoading />;
  if (snapshot.status === "failed") {
    if (snapshot.accepted === null) {
      return <ReadFailure subject="Overflow" onRetry={retry(store)} />;
    }
    return (
      <StateBlock
        icon={AlertIcon}
        danger
        title="Unable to load Overflow"
        action={
          <ActionButton
            label="Try again"
            tone="primary"
            onPress={retry(store)}
          />
        }
      >
        {`${acceptedText[snapshot.accepted]} Check the connection, then try again.`}
      </StateBlock>
    );
  }
  return (
    <Settings
      store={store}
      settings={snapshot.settings}
      refreshFailure={snapshot.refreshFailure}
    />
  );
}

function Overflow({ context }: { context: WorkspaceContext }) {
  const workspaceId = context.workspace.id;
  return (
    <>
      <PageHeader title="Settings" />
      <SettingsTabs workspaceId={workspaceId} active="overflow" />
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
