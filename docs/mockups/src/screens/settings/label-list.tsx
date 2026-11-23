// The labels of the organization: a table on desktop and cards on mobile,
// each with its platform and size, where its jobs run now, and its jobs of 30
// days. As in the machine table, the platform and size are a caption under
// the name. Where jobs run names the places of the order that can run the
// label now, in order, so a provider that cannot take a job or has no size
// that fits drops out. A label with an empty order, or one that no place
// fits, waits. So does a label whose fitting places cannot take a job now:
// the list gives the reason of the first place of its order that fits its
// size. A macOS label runs on the Macs only. The list shows the Linux labels
// first, then the macOS labels, each from the smallest size to the largest.

import { Pencil } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import {
  DataTable,
  LoadingRows,
  type Column,
} from "../../components/data-table.tsx";
import type { CataloguePath } from "../../components/paths.ts";
import { holdText } from "../../components/hold-text.ts";
import { DotLabel, Tag } from "../../components/status.tsx";
import { dotJoin, formatCount } from "../../fixtures/format.ts";
import { orderText } from "../../fixtures/label-order.ts";
import {
  placementOf,
  type PlaceContext,
  type Placement,
  type WaitReason,
} from "../../fixtures/placement.ts";
import { platformName, type Platform } from "../../fixtures/platform.ts";
import { providerName } from "../../fixtures/providers.ts";
import type { Label } from "../../fixtures/settings.ts";
import { sizeText } from "../../fixtures/sizes.ts";

const columns: readonly Column[] = [
  { key: "label", label: "Label", flex: 3 },
  { key: "where", label: "Where jobs run", flex: 2 },
  { key: "jobs", label: "Jobs, 30 days", width: 120, align: "right" },
  { key: "edit", label: "", width: 90, align: "right" },
];

/** Why the jobs of a label wait, after "Waits: ". */
function waitText(reason: WaitReason): string {
  switch (reason.kind) {
    case "empty-order":
      return "no place";
    case "nothing-fits":
      return "nothing fits";
    case "held":
      return holdText(reason.provider, reason.hold);
  }
}

/** "Your machines, then Boat", or why the jobs of the label wait. */
function placementText(placement: Placement, platform: Platform): string {
  switch (placement.kind) {
    case "places":
      return orderText(placement.places, platform);
    case "largest":
      return `${providerName(placement.provider)}, largest size`;
    case "waits":
      return `Waits: ${waitText(placement.reason)}`;
  }
}

/** The labels whose dialog has a screen of its own. */
export type EditPaths = Readonly<Record<string, CataloguePath>>;

/** Linux labels come before macOS labels in the list. */
const platformRank: Readonly<Record<Platform, number>> = {
  linux: 0,
  macos: 1,
};

/** Two names by character code, so the order is the same in every locale. */
function byCharacterCode(a: string, b: string): number {
  if (a === b) return 0;
  return a < b ? -1 : 1;
}

/**
 * The order of the list: by platform, then vCPU, memory, free disk, and
 * name, as "The label list" in docs/protocol/labels.md says.
 */
function listOrder(a: Label, b: Label): number {
  return (
    platformRank[a.platform] - platformRank[b.platform] ||
    a.vcpu - b.vcpu ||
    a.memoryGb - b.memoryGb ||
    a.freeDiskGb - b.freeDiskGb ||
    byCharacterCode(a.name, b.name)
  );
}

function LabelName({ label }: { label: Label }) {
  return (
    <span className="row">
      <span className="inline-code">{label.name}</span>
      {label.name === "ciproxy" ? <Tag>Default</Tag> : null}
    </span>
  );
}

/** "Linux · 2 vCPU · 8 GB · 20 GB free disk": the platform and the size. */
function labelText(label: Label): string {
  return dotJoin(platformName[label.platform], sizeText(label));
}

/** The name, and the platform and size under it. */
function LabelCell({ label }: { label: Label }) {
  return (
    <span className="label-cell">
      <LabelName label={label} />
      <span className="caption">{labelText(label)}</span>
    </span>
  );
}

function Placed({ label, context }: { label: Label; context: PlaceContext }) {
  const placement = placementOf(label, context);
  const text = placementText(placement, label.platform);
  return placement.kind === "waits" ? (
    <DotLabel tone="warn">{text}</DotLabel>
  ) : (
    <span>{text}</span>
  );
}

function EditLabel({ label, edits }: { label: Label; edits: EditPaths }) {
  const to = edits[label.name];
  return to ? (
    <LinkAction to={to} size="sm" icon={Pencil}>
      Edit
    </LinkAction>
  ) : (
    <Action size="sm" icon={Pencil}>
      Edit
    </Action>
  );
}

type ListScene = { context: PlaceContext; edits: EditPaths };

function cell(label: Label, key: string, scene: ListScene) {
  switch (key) {
    case "label":
      return <LabelCell label={label} />;
    case "where":
      return <Placed label={label} context={scene.context} />;
    case "jobs":
      return <span className="num">{formatCount(label.jobs)}</span>;
    default:
      return <EditLabel label={label} edits={scene.edits} />;
  }
}

export function LabelList({
  viewport,
  labels,
  loading,
  context,
  edits,
}: {
  viewport: Viewport;
  labels: readonly Label[];
  loading: boolean;
  /** The ready providers and the machines of the moment. */
  context: PlaceContext;
  edits: EditPaths;
}) {
  const scene: ListScene = { context, edits };
  const ordered = [...labels].sort(listOrder);
  if (viewport === "desktop") {
    return (
      <DataTable
        columns={columns}
        rows={loading ? [] : ordered}
        rowKey={(label) => label.name}
        cell={(label, key) => cell(label, key, scene)}
        loading={loading}
        loadingRowCount={3}
      />
    );
  }
  if (loading) return <LoadingRows count={3} />;
  return (
    <ul className="label-cards">
      {ordered.map((label) => (
        <li key={label.name} className="label-card">
          <span className="label-card-text">
            <LabelName label={label} />
            <span className="small">{labelText(label)}</span>
            <span className="small">
              <Placed label={label} context={context} />
            </span>
            <span className="caption">
              {formatCount(label.jobs)} jobs in 30 days
            </span>
          </span>
          <EditLabel label={label} edits={edits} />
        </li>
      ))}
    </ul>
  );
}
