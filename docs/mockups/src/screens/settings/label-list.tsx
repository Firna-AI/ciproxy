// The labels of the organization: a table on desktop and cards on mobile,
// each with its size, where its jobs run now, and its jobs of 30 days. As in
// the machine table, the size is a caption under the name.

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
import { formatCount } from "../../fixtures/format.ts";
import {
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
  "boat-only": "Boat only",
  waits: "Waits: nothing fits",
};

/** The labels whose dialog has a screen of its own. */
const editPaths: Readonly<Record<string, CataloguePath>> = {
  "ciproxy-8vcpu": paths.editLabel,
  [nothingFitsLabel.name]: paths.nothingFits,
};

function LabelName({ label }: { label: Label }) {
  return (
    <span className="row">
      <span className="inline-code">{label.name}</span>
      {label.name === "ciproxy" ? <Tag>Default</Tag> : null}
    </span>
  );
}

/** The name, and the size under it. */
function LabelCell({ label }: { label: Label }) {
  return (
    <span className="label-cell">
      <LabelName label={label} />
      <span className="caption">{sizeText(label)}</span>
    </span>
  );
}

function Placed({ label }: { label: Label }) {
  const placement = placementOf(label);
  return placement === "waits" ? (
    <DotLabel tone="warn">{placementText.waits}</DotLabel>
  ) : (
    <span>{placementText[placement]}</span>
  );
}

function EditLabel({ label }: { label: Label }) {
  const to = editPaths[label.name];
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

function cell(label: Label, key: string) {
  switch (key) {
    case "label":
      return <LabelCell label={label} />;
    case "where":
      return <Placed label={label} />;
    case "jobs":
      return <span className="num">{formatCount(label.jobs)}</span>;
    default:
      return <EditLabel label={label} />;
  }
}

export function LabelList({
  viewport,
  labels,
  loading,
}: {
  viewport: Viewport;
  labels: readonly Label[];
  loading: boolean;
}) {
  if (viewport === "desktop") {
    return (
      <DataTable
        columns={columns}
        rows={loading ? [] : labels}
        rowKey={(label) => label.name}
        cell={cell}
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
            <span className="small">{sizeText(label)}</span>
            <span className="small">
              <Placed label={label} />
            </span>
            <span className="caption">
              {formatCount(label.jobs)} jobs in 30 days
            </span>
          </span>
          <EditLabel label={label} />
        </li>
      ))}
    </ul>
  );
}
