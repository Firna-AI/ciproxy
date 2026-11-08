// The monthly spend limit of Settings under Overflow, with the spend of the
// month: the costs that Boat confirmed and the costs that it has not
// confirmed yet, apart. When ciproxy cannot read the spend, the panel says
// so, and the limit still applies. A failed save keeps the typed limit.

import { CircleAlert } from "lucide-react";

import { Action } from "../../components/actions.tsx";
import { BoatSpendFigures } from "../../components/boat-spend.tsx";
import { Notice } from "../../components/feedback.tsx";
import { TextField } from "../../components/fields.tsx";
import { Panel } from "../../components/page.tsx";
import { unsavedLimit } from "../../fixtures/boat-form.ts";
import { spendScenes } from "../../fixtures/boat-spend.ts";
import { boat } from "../../fixtures/machines.ts";

export type SpendState =
  "steady" | "limit-reached" | "unavailable" | "save-failed";

function SpendUnavailable() {
  return (
    <div className="spend-missing">
      <CircleAlert size={16} className="icon-muted" />
      <div className="stack-tight">
        <p className="strong">Unable to read the Boat spend</p>
        <p className="caption">The limit still applies.</p>
      </div>
      <Action size="sm">Try again</Action>
    </div>
  );
}

export function SpendLimitPanel({ state }: { state: SpendState }) {
  const reached = state === "limit-reached";
  const failed = state === "save-failed";
  return (
    <Panel title="Monthly spend limit">
      <div className="stack settings-form">
        <TextField
          label="Limit, US dollars"
          value={String(failed ? unsavedLimit : boat.spendLimit)}
        />
        {state === "unavailable" ? (
          <SpendUnavailable />
        ) : (
          <BoatSpendFigures
            spend={reached ? spendScenes.limitReached : spendScenes.steady}
            limit={boat.spendLimit}
            tone={reached ? "warning" : "normal"}
          />
        )}
        {failed ? (
          <Notice tone="danger" title="Unable to save the limit">
            Try Save again.
          </Notice>
        ) : null}
        <div className="row">
          <Action tone="primary">Save</Action>
        </div>
      </div>
    </Panel>
  );
}
