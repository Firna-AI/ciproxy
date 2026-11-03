// The Boat account that takes overflow: where jobs go when the own machines
// are busy, and the spend of the month against its limit.

import { Cloud } from "lucide-react";

import { TextLink } from "../../components/actions.tsx";
import { Meter } from "../../components/figures.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { formatMoneyCents } from "../../fixtures/format.ts";
import { boat } from "../../fixtures/machines.ts";

export function OverflowPanel() {
  return (
    <Panel
      title="Overflow"
      action={<TextLink to={paths.overflow}>Overflow settings</TextLink>}
    >
      <div className="overflow-row">
        <span className="where-badge">
          <Cloud size={18} />
        </span>
        <div className="stack-tight overflow-text">
          <p className="strong">Boat</p>
          <p className="caption">
            Connected to the account {boat.account} · jobs start there after{" "}
            {boat.waitSeconds} seconds
          </p>
        </div>
        <div className="overflow-meter">
          <Meter
            label="Spend this month"
            used={boat.spentThisMonth}
            max={boat.spendLimit}
            value={`${formatMoneyCents(boat.spentThisMonth)} of ${formatMoneyCents(boat.spendLimit)}`}
          />
        </div>
      </div>
    </Panel>
  );
}
