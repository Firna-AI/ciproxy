// The own machines as a table on desktop and as cards on mobile.

import type { Viewport } from "@mokly/mokly";
import { MockLink } from "@mokly/mokly";

import { DataTable } from "../../components/data-table.tsx";
import { SlotLegend, SlotStrip } from "../../components/figures.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { MachineStatusBadge } from "../../components/status.tsx";
import { ax42, type Machine } from "../../fixtures/machines.ts";

/** The detail screens that exist: hetzner-ax42-1, and the offline machine. */
function detailPath(machine: Machine): CataloguePath | undefined {
  if (machine.status === "offline") return paths.machineOffline;
  return machine.name === "hetzner-ax42-1" ? paths.machine : undefined;
}

function MachineName({ machine }: { machine: Machine }) {
  const to = detailPath(machine);
  return (
    <span className="machine-name">
      {to ? (
        <MockLink to={to} className="machine-link">
          {machine.name}
        </MockLink>
      ) : (
        <span className="strong">{machine.name}</span>
      )}
      <span className="caption">
        {ax42.threads} threads · {ax42.memoryGb} GB
      </span>
    </span>
  );
}

function slotText(machine: Machine): string {
  if (machine.status === "offline") return "No slots available";
  const busy = machine.slots.filter((slot) => slot === "busy").length;
  const warm = machine.slots.filter((slot) => slot === "warm").length;
  return `${busy} busy · ${warm} warm`;
}

function cell(machine: Machine, key: string) {
  switch (key) {
    case "machine":
      return <MachineName machine={machine} />;
    case "status":
      return <MachineStatusBadge status={machine.status} />;
    case "slots":
      return (
        <span className="stack-tight slots-cell">
          {machine.status === "offline" ? null : (
            <SlotStrip slots={machine.slots} />
          )}
          <span className="caption">{slotText(machine)}</span>
        </span>
      );
    case "cache":
      return (
        <span className="num">
          {machine.cacheUsedGb} of {machine.cacheSizeGb} GB
        </span>
      );
    default:
      return <span className="muted last-seen">{machine.lastSeen}</span>;
  }
}

const columns = [
  { key: "machine", label: "Machine", flex: 2 },
  { key: "status", label: "Health", width: 120 },
  { key: "slots", label: "Slots", flex: 2 },
  { key: "cache", label: "Cache", width: 130 },
  { key: "seen", label: "Last seen", width: 110, align: "right" as const },
];

export function MachineTable({
  viewport,
  machines,
}: {
  viewport: Viewport;
  machines: readonly Machine[];
}) {
  if (viewport === "mobile") {
    return (
      <div className="machine-cards">
        {machines.map((machine) => (
          <div key={machine.name} className="machine-card">
            <div className="spread">
              <MachineName machine={machine} />
              <MachineStatusBadge status={machine.status} />
            </div>
            <div className="spread">
              {machine.status === "offline" ? (
                <span />
              ) : (
                <SlotStrip slots={machine.slots} />
              )}
              <span className="caption">{slotText(machine)}</span>
            </div>
          </div>
        ))}
        <div className="machine-legend">
          <SlotLegend />
        </div>
      </div>
    );
  }
  return (
    <>
      <DataTable
        columns={columns}
        rows={[...machines]}
        rowKey={(machine) => machine.name}
        cell={cell}
      />
      <div className="machine-legend">
        <SlotLegend />
      </div>
    </>
  );
}

export function LoadingMachineTable() {
  return (
    <DataTable
      columns={columns.slice(0, 3)}
      rows={[]}
      rowKey={(_row, index) => String(index)}
      cell={() => null}
      loading
      loadingRowCount={3}
    />
  );
}
