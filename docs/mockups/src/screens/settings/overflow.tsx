// Overflow settings: the Boat account, how long a job waits for an own
// machine before it starts on Boat, and the monthly Boat spend limit.

import type { Viewport } from "@mokly/mokly";
import { DropdownSelector } from "@firna/ui/dropdown";
import { Switch } from "@firna/ui/switch";

import { Action, noop } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { TextField } from "../../components/fields.tsx";
import { Meter } from "../../components/figures.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { formatMoneyCents } from "../../fixtures/format.ts";
import { boat } from "../../fixtures/machines.ts";
import { waitChoices } from "../../fixtures/settings.ts";
import { SettingsFrame } from "./settings-frame.tsx";

export const overflowStates = [
  "settings",
  "limit-reached",
  "boat-rejected",
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

export function OverflowScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: OverflowState;
}) {
  const spent =
    state === "limit-reached" ? boat.spendLimit : boat.spentThisMonth;
  return (
    <SettingsFrame viewport={viewport} active={paths.overflow}>
      {state === "limit-reached" ? (
        <Notice
          tone="warning"
          title="Boat spend reached the $200 limit on 26 November"
        >
          Jobs wait for a free slot on your machines until 1 December, or until
          you raise the limit.
        </Notice>
      ) : null}
      {state === "boat-rejected" ? (
        <Notice tone="danger" title="Boat rejected the API key on 12 November">
          Jobs wait for your machines until you connect Boat again with a new
          key.
        </Notice>
      ) : null}
      <Panel title="Boat">
        {state === "boat-rejected" ? (
          <div className="stack settings-form">
            <TextField
              label="New Boat API key"
              placeholder="boat_live_…"
              hint="Create a key in the Boat console, under Settings."
            />
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
      <Panel title="Monthly spend limit">
        <div className="stack settings-form">
          <TextField
            label="Limit, US dollars"
            value={String(state === "limit-reached" ? 200 : boat.spendLimit)}
          />
          <Meter
            label="Spent this month"
            used={spent}
            max={boat.spendLimit}
            tone={state === "limit-reached" ? "warning" : "normal"}
            value={`${formatMoneyCents(spent)} of ${formatMoneyCents(boat.spendLimit)}`}
            detail="At the limit, jobs wait for a free slot on your machines."
          />
          <div className="row">
            <Action tone="primary">Save</Action>
          </div>
        </div>
      </Panel>
    </SettingsFrame>
  );
}
