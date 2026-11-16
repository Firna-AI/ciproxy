// The machine cache of one machine: its use, the cache hits, and the Docker
// disks. Each part shows the figures that the machine measured in its last
// report. A part that the machine did not measure shows Unknown, never a zero
// or a share, and a note says that its figures show after the machine
// measures them. Clear cache deletes every entry of the machine cache, for
// example to drop a bad entry that the machine kept after GitHub deleted it.
// It never deletes an entry of the GitHub cache. Only an Admin clears a cache,
// also while its figures are unknown, because a clear needs no figures.

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

/** A figure that the machine did not measure. */
export function UnknownFigure() {
  return <span className="muted">Unknown</span>;
}

/** The disk of the machine cache, as a meter, or Unknown with no bar. */
function CacheDisk({ cache }: { cache: Machine["cache"] }) {
  if (cache.status === "measured") {
    return (
      <Meter
        label="Cache disk"
        used={cache.usedGb}
        max={cache.sizeGb}
        value={`${cache.usedGb} of ${cache.sizeGb} GB`}
      />
    );
  }
  return (
    <div className="spread">
      <span className="meter-label">Cache disk</span>
      <UnknownFigure />
    </div>
  );
}

/** The note under the figures that names the parts that show Unknown. */
function unknownNote(machine: Machine): string | null {
  const cache = machine.cache.status === "unavailable";
  const docker = machine.dockerDisks.status === "unavailable";
  const after = "show after the machine measures them.";
  if (cache && docker) {
    return `Cache disk, cache hits, and Docker disks ${after}`;
  }
  if (cache) return `Cache disk and cache hits ${after}`;
  if (docker) return `Docker disks ${after}`;
  return null;
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
  const { cache, dockerDisks } = machine;
  const note = unknownNote(machine);
  return (
    <Panel
      title="Cache"
      action={
        offline || !canClear ? undefined : <ClearCache machine={machine} />
      }
    >
      <div className="stack">
        <CacheDisk cache={cache} />
        <DetailList>
          <DetailRow label="Cache hits, 7 days">
            {cache.status === "measured" ? (
              `${cacheHitsPercent}%`
            ) : (
              <UnknownFigure />
            )}
          </DetailRow>
          <DetailRow label="Docker disks">
            {dockerDisks.status === "measured" ? (
              dotJoin(
                unbroken(`${dockerDisks.repositories} repositories`),
                unbroken(`${dockerDisks.sizeGb} GB`),
              )
            ) : (
              <UnknownFigure />
            )}
          </DetailRow>
        </DetailList>
      </div>
      {note ? <p className="caption cache-note">{note}</p> : null}
    </Panel>
  );
}

export function ClearCacheDialog({ machine }: { machine: Machine }) {
  const { cache } = machine;
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
        ciproxy deletes{" "}
        {cache.status === "measured"
          ? `the ${unbroken(`${cache.usedGb} GB`)} of cache entries`
          : "the cache entries"}{" "}
        on this machine. The GitHub cache keeps its entries, so the next jobs
        restore from there.
      </p>
    </Dialog>
  );
}
