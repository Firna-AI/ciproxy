// The machine cache of one machine: its use, the cache hits, and the Docker
// disks. Clear cache deletes every entry of the machine cache, for example to
// drop a bad entry that the machine kept after GitHub deleted it. It never
// deletes an entry of the GitHub cache. Only an Admin clears a cache.

import { Action, LinkAction } from "../../components/actions.tsx";
import { Dialog } from "../../components/dialog.tsx";
import { Meter } from "../../components/figures.tsx";
import { DetailList, DetailRow, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { dotJoin, unbroken } from "../../fixtures/format.ts";
import { cacheHitsPercent, type Machine } from "../../fixtures/machines.ts";

/** Clear cache has its dialog screen for hetzner-ax42-1. */
function ClearCache({ machine }: { machine: Machine }) {
  return machine.name === "hetzner-ax42-1" ? (
    <LinkAction to={paths.machineClearCache} size="sm">
      Clear cache
    </LinkAction>
  ) : (
    <Action size="sm">Clear cache</Action>
  );
}

export function MachineCache({
  machine,
  canClear,
}: {
  machine: Machine;
  /** An Admin can clear the cache. */
  canClear: boolean;
}) {
  const offline = machine.status === "offline";
  const { repositories, sizeGb } = machine.dockerDisks;
  return (
    <Panel
      title="Cache"
      action={
        offline || !canClear ? undefined : <ClearCache machine={machine} />
      }
    >
      <div className="stack">
        <Meter
          label="Cache disk"
          used={machine.cacheUsedGb}
          max={machine.cacheSizeGb}
          value={`${machine.cacheUsedGb} of ${machine.cacheSizeGb} GB`}
        />
        <DetailList>
          <DetailRow label="Cache hits, 7 days">{cacheHitsPercent}%</DetailRow>
          <DetailRow label="Docker disks">
            {dotJoin(
              unbroken(`${repositories} repositories`),
              unbroken(`${sizeGb} GB`),
            )}
          </DetailRow>
        </DetailList>
      </div>
    </Panel>
  );
}

export function ClearCacheDialog({ machine }: { machine: Machine }) {
  return (
    <Dialog
      title={`Clear the cache of ${machine.name}?`}
      footer={
        <>
          <LinkAction to={paths.machine} tone="ghost">
            Cancel
          </LinkAction>
          <LinkAction to={paths.machineClearRequested} tone="danger">
            Clear cache
          </LinkAction>
        </>
      }
    >
      <p className="small">
        ciproxy deletes the {unbroken(`${machine.cacheUsedGb} GB`)} of cache
        entries on this machine. The GitHub cache keeps its entries, so the next
        jobs restore from there.
      </p>
    </Dialog>
  );
}
