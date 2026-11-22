// The labels of the organization: a table on desktop and cards on mobile,
// each with its platform and size, where its jobs run now, and its jobs of 30
// days. As in the machine table, the platform and size are a caption under
// the name. Without a Boat account, the labels that fit a machine run on the
// machines only. A macOS label runs on the Macs only, with or without Boat.

import { Pencil } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import {
  DataTable,
  LoadingRows,
  type Column,
} from "../../components/data-table.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { DotLabel, Tag } from "../../components/status.tsx";
import { dotJoin, formatCount } from "../../fixtures/format.ts";
import type { Machine } from "../../fixtures/machines.ts";
import { platformName } from "../../fixtures/platform.ts";
import {
  macosLabel,
  nothingFitsLabel,
  placementOf,
  type Label,
  type Placement,
} from "../../fixtures/settings.ts";
import { sizeText } from "../../fixtures/sizes.ts";

const columns: readonly Column[] = [
  { key: "label", label: "Label", flex: 3 },
  { key: "where", label: "Where jobs run", flex: 2 },
  { key: "jobs", label: "Jobs, 30 days", width: 120, align: "right" },
  { key: "edit", label: "", width: 90, align: "right" },
];

const placementText: Record<Placement, string> = {
  "machines-then-boat": "Your machines, then Boat",
  "machines-only": "Your machines only",
  "macs-only": "Your Macs only",
  "boat-only": "Boat only",
  waits: "Waits: nothing fits",
};

/** The labels whose dialog has a screen of its own, with and without Boat. */
const editPaths: Readonly<Record<string, CataloguePath>> = {
  "ciproxy-large": paths.editLabel,
  [nothingFitsLabel.name]: paths.nothingFits,
  [macosLabel.name]: paths.editMacosLabel,
};
const editPathsNoBoat: Readonly<Record<string, CataloguePath>> = {
  "ciproxy-large": paths.editLabelNoBoat,
};

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

/** Where the jobs of a label run, and the machines that decide it. */
type Where = { boat: boolean; machines?: readonly Machine[] };

function Placed({ label, where }: { label: Label; where: Where }) {
  const placement = placementOf(label, where.boat, where.machines);
  return placement === "waits" ? (
    <DotLabel tone="warn">{placementText.waits}</DotLabel>
  ) : (
    <span>{placementText[placement]}</span>
  );
}

function EditLabel({ label, boat }: { label: Label; boat: boolean }) {
  const to = (boat ? editPaths : editPathsNoBoat)[label.name];
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

function cell(label: Label, key: string, where: Where) {
  switch (key) {
    case "label":
      return <LabelCell label={label} />;
    case "where":
      return <Placed label={label} where={where} />;
    case "jobs":
      return <span className="num">{formatCount(label.jobs)}</span>;
    default:
      return <EditLabel label={label} boat={where.boat} />;
  }
}

export function LabelList({
  viewport,
  labels,
  loading,
  boat = true,
  machines,
}: {
  viewport: Viewport;
  labels: readonly Label[];
  loading: boolean;
  /** The workspace has a Boat account. */
  boat?: boolean;
  /** The machines of the moment, when they are not the busy afternoon's. */
  machines?: readonly Machine[];
}) {
  const where: Where = { boat, machines };
  if (viewport === "desktop") {
    return (
      <DataTable
        columns={columns}
        rows={loading ? [] : labels}
        rowKey={(label) => label.name}
        cell={(label, key) => cell(label, key, where)}
        loading={loading}
        loadingRowCount={3}
      />
    );
  }
  if (loading) return <LoadingRows count={3} />;
  return (
    <ul className="label-cards">
      {labels.map((label) => (
        <li key={label.name} className="label-card">
          <span className="label-card-text">
            <LabelName label={label} />
            <span className="small">{labelText(label)}</span>
            <span className="small">
              <Placed label={label} where={where} />
            </span>
            <span className="caption">
              {formatCount(label.jobs)} jobs in 30 days
            </span>
          </span>
          <EditLabel label={label} boat={boat} />
        </li>
      ))}
    </ul>
  );
}
