// The headline figures above a table. Outside a design direction they are
// Firna stat tiles. Inside one, the direction picks the form: one divided
// strip, stat tiles with a faint area trend, or cards with a trend line.

import type { ReactNode } from "react";
import { StatTile, StatTileRow } from "@firna/ui/chart";

import { formatCount } from "../fixtures/format.ts";
import { useDirection } from "../theme/direction.ts";
import { Trend } from "./trend.tsx";
import "./kpis.css";
import "./kpis-kinds.css";

export function KpiRow({ children }: { children: ReactNode }) {
  const direction = useDirection();
  if (!direction) return <StatTileRow>{children}</StatTileRow>;
  return <div className={`kpis kpis-${direction.kpi}`}>{children}</div>;
}

export function Kpi({
  label,
  value,
  valueFormat,
  trend,
}: {
  label: string;
  value: number;
  valueFormat?: (value: number) => string;
  /** Recent values, oldest first. A direction may draw them as a trend. */
  trend?: readonly number[];
}) {
  const direction = useDirection();
  if (!direction) {
    return <StatTile label={label} value={value} valueFormat={valueFormat} />;
  }
  const shown = (valueFormat ?? formatCount)(value);
  const drawn = direction.kpi === "strip" ? undefined : trend;
  return (
    <div className="kpi">
      <span className="kpi-label">{label}</span>
      <span className="kpi-value">{shown}</span>
      {drawn ? (
        <span className="kpi-trend">
          <Trend
            data={drawn}
            area={direction.kpi === "stat"}
            end={direction.kpi === "card"}
          />
        </span>
      ) : null}
    </div>
  );
}
