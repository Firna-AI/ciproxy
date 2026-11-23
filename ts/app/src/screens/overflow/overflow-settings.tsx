// The body of Settings under Overflow. With no connected provider, the page
// asks for a provider and its key and limit, as Setup does, or says that the
// deployment can start no provider now. Otherwise it shows a notice for each
// provider that needs one, then the switch and the wait, which hold for every
// provider, then one panel for each provider that is connected or available.
// After a change in place, focus moves to the title of the panel that
// changed.

import { useEffect, useRef, useState } from "react";
import type { View } from "react-native";

import type { ApiFailure } from "../../api/client.ts";
import type { OverflowProvider, OverflowSettings } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { CloudOffIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { StateBlock } from "../../components/page-state.tsx";
import { Panel } from "../../components/panel.tsx";
import { useValue } from "../../services/hooks.ts";
import type { OverflowStore } from "../../state/overflow.ts";
import {
  connectedProviders,
  overflowNotices,
  panelProviders,
} from "../../view/overflow.ts";
import { DisconnectDialog } from "./disconnect-dialog.tsx";
import { ProviderConnect } from "./provider-choice.tsx";
import { ProviderPanel } from "./provider-panel.tsx";
import { usePolicy } from "./use-policy.ts";
import { WaitPanel } from "./wait-panel.tsx";

/** A request to focus the title of a panel, counted so each one moves it. */
type FocusRequest = { provider: OverflowProvider | null; count: number };

function ConnectPanel({
  store,
  settings,
  focusRequest,
  onConnected,
}: {
  store: OverflowStore;
  settings: OverflowSettings;
  focusRequest: number;
  onConnected: (provider: OverflowProvider) => void;
}) {
  const title = useRef<View>(null);
  const locked = useValue(store.pending);
  useEffect(() => {
    if (focusRequest > 0) title.current?.focus();
  }, [focusRequest]);
  const available = settings.providers.some((item) => item.available);
  return (
    <Panel title="Connect an overflow provider" titleRef={title}>
      {available ? (
        <ProviderConnect
          store={store}
          providers={settings.providers}
          locked={locked}
          onConnected={onConnected}
        />
      ) : (
        <StateBlock
          icon={CloudOffIcon}
          title="No overflow provider is available right now"
          action={
            <ActionButton
              label="Try again"
              tone="primary"
              onPress={() => void store.load()}
            />
          }
        >
          Jobs run on your machines only.
        </StateBlock>
      )}
    </Panel>
  );
}

function Policy({
  workspaceId,
  store,
  settings,
}: {
  workspaceId: string;
  store: OverflowStore;
  settings: OverflowSettings;
}) {
  const editor = usePolicy(store, settings);
  return (
    <WaitPanel workspaceId={workspaceId} settings={settings} editor={editor} />
  );
}

export function OverflowSettingsView({
  workspaceId,
  store,
  settings,
  refreshFailure,
}: {
  workspaceId: string;
  store: OverflowStore;
  settings: OverflowSettings;
  refreshFailure: ApiFailure | null;
}) {
  const [disconnecting, setDisconnecting] = useState<OverflowProvider | null>(
    null,
  );
  const [focus, setFocus] = useState<FocusRequest>({
    provider: null,
    count: 0,
  });
  const moveFocus = (provider: OverflowProvider | null) =>
    setFocus((current) => ({ provider, count: current.count + 1 }));
  const connected = connectedProviders(settings).length > 0;
  return (
    <>
      {refreshFailure ? (
        <Notice
          tone="warning"
          title="Unable to refresh Overflow"
          actions={
            <ActionButton
              label="Try again"
              size="sm"
              onPress={() => void store.load()}
            />
          }
        >
          The page shows the last settings that it read.
        </Notice>
      ) : null}
      {connected ? (
        <>
          {overflowNotices(settings).map((notice) => (
            <Notice key={notice.title} tone={notice.tone} title={notice.title}>
              {notice.body}
            </Notice>
          ))}
          <Policy workspaceId={workspaceId} store={store} settings={settings} />
          {panelProviders(settings).map((provider) => (
            <ProviderPanel
              key={provider.provider}
              store={store}
              provider={provider}
              focusRequest={
                focus.provider === provider.provider ? focus.count : 0
              }
              onDisconnect={() => setDisconnecting(provider.provider)}
            />
          ))}
        </>
      ) : (
        <ConnectPanel
          store={store}
          settings={settings}
          focusRequest={focus.provider === null ? focus.count : 0}
          onConnected={moveFocus}
        />
      )}
      {disconnecting === null ? null : (
        <DisconnectDialog
          store={store}
          provider={disconnecting}
          onClose={() => setDisconnecting(null)}
          onDisconnected={() => {
            setDisconnecting(null);
            const left = store.state.get();
            const still =
              left.status === "ready" &&
              connectedProviders(left.settings).length > 0;
            moveFocus(still ? disconnecting : null);
          }}
        />
      )}
    </>
  );
}
