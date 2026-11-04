// The own machines as a table on desktop and as a dense list on mobile.

import type { Viewport } from "@mokly/mokly";
import { MockLink } from "@mokly/mokly";

import {
  DataTable,
  LoadingRows,
  type Column,
} from "../../components/data-table.tsx";
import { SlotLegend, SlotStrip } from "../../components/figures.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { MachineStatusLabel } from "../../components/status.tsx";
import { dotJoin, unbroken } from "../../fixtures/format.ts";
import { hardwareSize } from "../../fixtures/hardware.ts";
import { officeMini, type Machine } from "../../fixtures/machines.ts";

/**
 * The detail screens that exist: hetzner-ax42-1, the offline machine, and
 * office-mini-1, whose disk limits its slots. A person who cannot change
 * machines opens the read-only machine page.
 */
function detailPath(
  machine: Machine,
  readOnly: boolean,
): CataloguePath | undefined {
  const first = machine.name === "hetzner-ax42-1";
  if (readOnly) return first ? paths.machineViewer : undefined;
  if (machine.status === "offline") return paths.machineOffline;
  if (machine.name === officeMini.name) return paths.machineDiskLimit;
  return first ? paths.machine : undefined;
}

function MachineName({
  machine,
  readOnly,
}: {
  machine: Machine;
  readOnly: boolean;
}) {
  const to = detailPath(machine, readOnly);
  return (
    <span className="machine-name">
      {to ? (
        <MockLink to={to} className="machine-link mono">
          {machine.name}
        </MockLink>
      ) : (
        <span className="machine-link mono">{machine.name}</span>
      )}
      <span className="caption">{hardwareSize(machine.hardware)}</span>
    </span>
  );
}

function slotText(machine: Machine): string {
  if (machine.status === "offline") return "No slots available";
  if (machine.slots.length === 0) return "No slots";
  const busy = machine.slots.filter((slot) => slot === "busy").length;
  const warm = machine.slots.filter((slot) => slot === "warm").length;
  return dotJoin(unbroken(`${busy} busy`), unbroken(`${warm} warm`));
}

function cell(machine: Machine, key: string, readOnly: boolean) {
  switch (key) {
    case "machine":
      return <MachineName machine={machine} readOnly={readOnly} />;
    case "status":
      return <MachineStatusLabel status={machine.status} />;
    case "slots":
      return (
        <span className="slots-cell">
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
      return <span className="muted num">{machine.lastSeen}</span>;
  }
}

const columns: readonly Column[] = [
  { key: "machine", label: "Machine", flex: 2 },
  { key: "status", label: "Health", width: 112 },
  { key: "slots", label: "Slots", flex: 2 },
  { key: "cache", label: "Cache", width: 124 },
  { key: "seen", label: "Last seen", width: 96, align: "right" },
];

export function MachineTable({
  viewport,
  machines,
  readOnly = false,
}: {
  viewport: Viewport;
  machines: readonly Machine[];
  /** The person cannot change machines, so a row opens the read-only page. */
  readOnly?: boolean;
}) {
  if (viewport === "mobile") {
    return (
      <div className="machine-cards">
        {machines.map((machine) => (
          <div key={machine.name} className="machine-card">
            <div className="spread">
              <MachineName machine={machine} readOnly={readOnly} />
              <MachineStatusLabel status={machine.status} />
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
        rows={machines}
        rowKey={(machine) => machine.name}
        cell={(machine, key) => cell(machine, key, readOnly)}
      />
      <div className="machine-legend">
        <SlotLegend />
      </div>
    </>
  );
}

/** The machines while they load: a table on desktop, bar rows on mobile. */
export function LoadingMachineTable({ viewport }: { viewport: Viewport }) {
  if (viewport === "mobile") return <LoadingRows count={3} />;
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
