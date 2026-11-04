// Overflow settings: the Boat account, how long a job waits for an own
// machine before it starts on Boat, and the monthly Boat spend limit. Before
// Boat connects, the page asks for the key and the limit, as setup does.
// After Boat rejects the key, it asks for a new key and the limit again, and
// the limit starts with the saved limit.

import type { Viewport } from "@mokly/mokly";
import { DropdownSelector } from "@firna/ui/dropdown";
import { Switch } from "@firna/ui/switch";

import { Action, noop } from "../../components/actions.tsx";
import { BoatFields } from "../../components/boat-fields.tsx";
import { Notice } from "../../components/feedback.tsx";
import { TextField } from "../../components/fields.tsx";
import { Meter } from "../../components/figures.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { formatMoney } from "../../fixtures/format.ts";
import { boat } from "../../fixtures/machines.ts";
import { waitChoices } from "../../fixtures/settings.ts";
import { SettingsFrame } from "./settings-frame.tsx";

export const overflowStates = [
  "settings",
  "limit-reached",
  "boat-rejected",
  "not-connected",
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

/** Boat is not connected yet, for example after Skip for now in setup. */
function ConnectBoat() {
  return (
    <Panel title="Boat">
      <div className="stack settings-form">
        <p className="small">
          When your machines are busy, ciproxy starts jobs in your own Boat
          account. Boat bills you by the second.
        </p>
        <BoatFields />
        <div className="row">
          <Action tone="primary">Connect Boat</Action>
        </div>
      </div>
    </Panel>
  );
}

export function OverflowScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: OverflowState;
}) {
  if (state === "not-connected") {
    return (
      <SettingsFrame viewport={viewport} active={paths.overflow}>
        <ConnectBoat />
      </SettingsFrame>
    );
  }
  const spent =
    state === "limit-reached" ? boat.spendLimit : boat.spentThisMonth;
  const rejected = state === "boat-rejected";
  return (
    <SettingsFrame viewport={viewport} active={paths.overflow}>
      {state === "limit-reached" ? (
        <Notice
          tone="warning"
          title={`Boat spend reached the ${formatMoney(boat.spendLimit)} limit on ${boat.limitReachedOn}`}
        >
          Jobs run on your machines only, and wait there for a free slot. Boat
          takes jobs again on {boat.nextMonth}, or when you raise the limit.
        </Notice>
      ) : null}
      {rejected ? (
        <Notice
          tone="danger"
          title={`Boat rejected the API key on ${boat.keyRejectedOn}`}
        >
          Jobs run on your machines only until you connect Boat with a new key.
        </Notice>
      ) : null}
      <Panel title="Boat">
        {rejected ? (
          <div className="stack settings-form">
            <BoatFields reconnect limit={boat.spendLimit} />
            <div className="row">
              <Action tone="primary">Connect Boat</Action>
            </div>
          </div>
        ) : (
          <div className="spread spread-stack">
            <div className="stack-tight">
              <p className="strong">Connected to the account {boat.account}</p>
              <p className="caption">
                ciproxy starts overflow jobs in this account. Boat bills it.
              </p>
            </div>
            <div className="row-wrap">
              <Action size="sm">Change key</Action>
              <Action size="sm" tone="danger">
                Disconnect
              </Action>
            </div>
          </div>
        )}
      </Panel>
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
      {rejected ? null : (
        <SpendLimit spent={spent} reached={state === "limit-reached"} />
      )}
    </SettingsFrame>
  );
}

/** The limit and the spend of the month, while Boat is connected. */
function SpendLimit({ spent, reached }: { spent: number; reached: boolean }) {
  return (
    <Panel title="Monthly spend limit">
      <div className="stack settings-form">
        <TextField label="Limit, US dollars" value={String(boat.spendLimit)} />
        <Meter
          label="Spent this month"
          used={spent}
          max={boat.spendLimit}
          tone={reached ? "warning" : "normal"}
          value={`${formatMoney(spent)} of ${formatMoney(boat.spendLimit)}`}
          detail="At the limit, jobs run on your machines only."
        />
        <div className="row">
          <Action tone="primary">Save</Action>
        </div>
      </div>
    </Panel>
  );
}
