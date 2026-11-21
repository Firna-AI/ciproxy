// The feature comparison of the pricing page: one column for each tier, rows
// by area. A check marks an included feature and a dash one that is not. A
// feature that comes later says so under its name, and a clock marks the
// tiers that will include it.

import { Check, Clock, Minus } from "lucide-react";

import { ComingLater } from "../../components/tier-feature-list.tsx";
import {
  compareGroups,
  type CompareValue,
} from "../../fixtures/plan-compare.ts";
import { formatMoney } from "../../fixtures/format.ts";
import { tierOrder, tiers } from "../../fixtures/tiers.ts";
import { Section, SectionHead } from "./section.tsx";
import "./plan-compare.css";

function CompareCell({
  value,
  later,
}: {
  value: CompareValue;
  later: boolean;
}) {
  if (value === true && later) {
    return (
      <Clock size={15} className="compare-later" aria-label="Coming later" />
    );
  }
  if (value === true) {
    return <Check size={16} className="compare-yes" aria-label="Included" />;
  }
  if (value === false) {
    return <Minus size={14} className="compare-no" aria-label="Not included" />;
  }
  return <span className="num">{value}</span>;
}

export function PlanCompare() {
  return (
    <Section id="compare">
      <SectionHead title="Compare plans" />
      <table className="compare">
        <colgroup>
          <col className="compare-label-col" />
          {tierOrder.map((key) => (
            <col key={key} />
          ))}
        </colgroup>
        <thead>
          <tr>
            <th scope="col">
              <span className="sr-only">Feature</span>
            </th>
            {tierOrder.map((key) => (
              <th key={key} scope="col" className="compare-tier">
                <span className="compare-tier-name">{tiers[key].name}</span>
                <span className="compare-tier-price num">
                  {formatMoney(tiers[key].price)}
                </span>
              </th>
            ))}
          </tr>
        </thead>
        {compareGroups.map((group) => (
          <tbody key={group.title}>
            <tr className="compare-group">
              <th scope="colgroup" colSpan={tierOrder.length + 1}>
                {group.title}
              </th>
            </tr>
            {group.rows.map((row) => (
              <tr key={row.label}>
                <th scope="row">
                  <span className="compare-label">{row.label}</span>
                  {row.later ? <ComingLater /> : null}
                </th>
                {tierOrder.map((key) => (
                  <td key={key}>
                    <CompareCell
                      value={row.values[key]}
                      later={row.later ?? false}
                    />
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        ))}
      </table>
    </Section>
  );
}
