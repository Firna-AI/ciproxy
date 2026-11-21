// The headline figures above a table: one strip divided into cells by
// hairlines, each cell with a label, a value, and an optional line under the
// value, such as the machines that a figure leaves out.

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
  detail,
}: {
  label: string;
  /** A number to format, or the words of a figure, such as "Unknown". */
  value: number | string;
  valueFormat?: (value: number) => string;
  /** A short line under the value. */
  detail?: ReactNode;
}) {
  return (
    <div className="kpi">
      <span className="kpi-label">{label}</span>
      <span className="kpi-value">
        {typeof value === "number" ? valueFormat(value) : value}
      </span>
      {detail ? <span className="caption">{detail}</span> : null}
    </div>
  );
}
