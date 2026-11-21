// Overflow settings: the Boat account, how long a job waits for an own
// machine before it starts on Boat, and the monthly Boat spend limit with the
// spend of the month. Before Boat connects, the page asks for the key and the
// limit, as setup does. After Boat rejects the key, it asks for a new key and
// the limit again, and the limit starts with the saved limit.

import type { Viewport } from "@mokly/mokly";
import { DropdownSelector } from "@firna/ui/dropdown";
import { Switch } from "@firna/ui/switch";

import { noop } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { formatMoney } from "../../fixtures/format.ts";
import { boat } from "../../fixtures/machines.ts";
import { waitChoices } from "../../fixtures/settings.ts";
import {
  AccountPanel,
  BoatUnavailablePanel,
  ChangeKeyPanel,
  ConnectBoatPanel,
  DisconnectDialog,
  ReconnectBoatPanel,
} from "./overflow-boat.tsx";
import { SpendLimitPanel, type SpendState } from "./overflow-spend.tsx";
import { SettingsFrame } from "./settings-frame.tsx";

export const overflowStates = [
  "settings",
  "limit-reached",
  "boat-rejected",
  "not-connected",
  "change-key",
  "disconnect",
  "unavailable",
  "unknown-costs",
  "spend-unavailable",
  "save-failed",
  "wallet-connect",
  "wallet-reconnect",
  "wallet-change-key",
] as const;
export type OverflowState = (typeof overflowStates)[number];

function waitLabel(seconds: number): string {
  if (seconds === 0) return "At once";
  if (seconds < 60) return `${seconds} seconds`;
  return seconds === 60 ? "1 minute" : `${seconds / 60} minutes`;
}

const waitOptions = waitChoices.map((seconds) => ({
  label: waitLabel(seconds),
  value: String(seconds),
}));

function WaitPanel() {
  return (
    <Panel title="When jobs start on Boat">
      <div className="stack settings-form">
        <div className="toggle-row">
          <span className="strong">Use Boat when your machines are busy</span>
          <Switch
            value
            onValueChange={noop}
            accessibilityLabel="Use Boat when your machines are busy"
          />
        </div>
        <DropdownSelector
          variant="field"
          label="Start on Boat after a job waits"
          options={waitOptions}
          value={String(boat.waitSeconds)}
          onValueChange={noop}
          hint="Warm runners on your machines take jobs first."
        />
      </div>
    </Panel>
  );
}

function spendState(state: OverflowState): SpendState {
  if (state === "limit-reached") return "limit-reached";
  if (state === "spend-unavailable") return "unavailable";
  if (state === "save-failed") return "save-failed";
  return "steady";
}

function PageNotice({ state }: { state: OverflowState }) {
  switch (state) {
    case "limit-reached":
      return (
        <Notice
          tone="warning"
          title={`Boat spend reached the ${formatMoney(boat.spendLimit)} limit for ${boat.budgetMonth}`}
        >
          Jobs run on your machines only, and wait there for a free slot. Boat
          takes jobs again on {boat.nextMonth}, or when you raise the limit.
        </Notice>
      );
    case "boat-rejected":
    case "wallet-reconnect":
      return (
        <Notice
          tone="danger"
          title={`Boat rejected the API key on ${boat.keyRejectedOn}`}
        >
          Jobs run on your machines only until you connect Boat with a new key.
        </Notice>
      );
    case "unknown-costs":
      return (
        <Notice tone="warning" title="Boat costs are not known yet">
          Jobs run on your machines only until Boat reports the cost of each
          start.
        </Notice>
      );
    default:
      return null;
  }
}

function BoatPanel({ state }: { state: OverflowState }) {
  if (state === "boat-rejected") return <ReconnectBoatPanel />;
  if (state === "wallet-reconnect") return <ReconnectBoatPanel checked />;
  if (state === "change-key") return <ChangeKeyPanel />;
  if (state === "wallet-change-key") return <ChangeKeyPanel checked />;
  return <AccountPanel />;
}

export function OverflowScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: OverflowState;
}) {
  if (
    state === "not-connected" ||
    state === "wallet-connect" ||
    state === "unavailable"
  ) {
    return (
      <SettingsFrame viewport={viewport} active={paths.overflow}>
        {state === "unavailable" ? (
          <BoatUnavailablePanel />
        ) : (
          <ConnectBoatPanel checked={state === "wallet-connect"} />
        )}
      </SettingsFrame>
    );
  }
  return (
    <SettingsFrame
      viewport={viewport}
      active={paths.overflow}
      overlay={state === "disconnect" ? <DisconnectDialog /> : undefined}
    >
      <PageNotice state={state} />
      <BoatPanel state={state} />
      <WaitPanel />
      {state === "boat-rejected" || state === "wallet-reconnect" ? null : (
        <SpendLimitPanel state={spendState(state)} />
      )}
    </SettingsFrame>
  );
}
