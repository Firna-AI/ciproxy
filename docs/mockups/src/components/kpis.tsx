// The headline figures above a table: one strip divided into cells by
// hairlines, each cell with a label and a value.

import type { ReactNode } from "react";

import { formatCount } from "../fixtures/format.ts";
import "./kpis.css";

export function KpiRow({ children }: { children: ReactNode }) {
  return <div className="kpis">{children}</div>;
}

export function Kpi({
  label,
  value,
  valueFormat = formatCount,
}: {
  label: string;
  value: number;
  valueFormat?: (value: number) => string;
}) {
  return (
    <div className="kpi">
      <span className="kpi-label">{label}</span>
      <span className="kpi-value">{valueFormat(value)}</span>
    </div>
  );
}
