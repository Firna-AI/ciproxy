// The cache hits of the last 7 days on Machines: the share of the cache
// restores of the machines that measured their machine cache. The line under
// the figure names each machine that the share leaves out, so the share never
// hides one. With no measured machine, the figure is Unknown, never a share
// or "No restores yet".

import { Fragment } from "react";

import { Kpi } from "../../components/kpis.tsx";
import { cacheHitsPercent, type Machine } from "../../fixtures/machines.ts";

/** The words before the name at `index` of `count` names. */
function joiner(index: number, count: number): string {
  if (index === 0) return "";
  if (index < count - 1) return ", ";
  return count === 2 ? " and " : ", and ";
}

/** "Excludes hetzner-ax42-1 and hetzner-ax42-2", each name whole in mono. */
function Excludes({ machines }: { machines: readonly Machine[] }) {
  return (
    <>
      Excludes{" "}
      {machines.map((machine, index) => (
        <Fragment key={machine.name}>
          {joiner(index, machines.length)}
          <span className="mono nowrap">{machine.name}</span>
        </Fragment>
      ))}
    </>
  );
}

export function CacheHitsKpi({ machines }: { machines: readonly Machine[] }) {
  const excluded = machines.filter(
    (machine) => machine.cache.status !== "measured",
  );
  const measured = excluded.length < machines.length;
  return (
    <Kpi
      label="Cache hits, 7 days"
      value={measured ? `${cacheHitsPercent}%` : "Unknown"}
      detail={
        measured && excluded.length > 0 ? (
          <Excludes machines={excluded} />
        ) : undefined
      }
    />
  );
}
