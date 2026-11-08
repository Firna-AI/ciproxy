// The Boat account that takes overflow: where jobs go when the own machines
// are busy, and the spend of the month against its limit, with the confirmed
// and the unconfirmed costs apart. Only an Admin gets the link to the
// overflow settings.

import { Cloud } from "lucide-react";

import { TextLink } from "../../components/actions.tsx";
import { BoatSpendFigures } from "../../components/boat-spend.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { spendScenes } from "../../fixtures/boat-spend.ts";
import { dotJoin, unbroken } from "../../fixtures/format.ts";
import { boat } from "../../fixtures/machines.ts";

export function OverflowPanel({ settings }: { settings: boolean }) {
  return (
    <Panel
      title="Overflow"
      action={
        settings ? (
          <TextLink to={paths.overflow}>Overflow settings</TextLink>
        ) : undefined
      }
    >
      <div className="overflow-row">
        <span className="icon-tile">
          <Cloud size={16} />
        </span>
        <div className="stack-tight overflow-text">
          <p className="strong">Boat</p>
          <p className="caption">
            {dotJoin(
              `Connected to the account ${boat.account}`,
              `jobs start there after ${unbroken(`${boat.waitSeconds} seconds`)}`,
            )}
          </p>
        </div>
        <div className="overflow-meter">
          <BoatSpendFigures
            spend={spendScenes.steady}
            limit={boat.spendLimit}
            compact
          />
        </div>
      </div>
    </Panel>
  );
}
