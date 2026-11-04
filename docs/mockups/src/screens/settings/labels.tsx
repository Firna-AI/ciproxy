// Labels and machine shapes. A workflow picks a label in runs-on; each label
// starts a VM of its shape on an own machine, or on Boat. Every label has an
// Edit control; the mockup opens the dialog for ciproxy-8vcpu.

import { Pencil, Plus } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import {
  DataTable,
  LoadingRows,
  type Column,
} from "../../components/data-table.tsx";
import { ReadFailure } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { Tag } from "../../components/status.tsx";
import { dotJoin, formatCount, unbroken } from "../../fixtures/format.ts";
import { labels, type Label } from "../../fixtures/settings.ts";
import { SettingsFrame } from "./settings-frame.tsx";
import { ShapeDialog } from "./shape-dialog.tsx";

export const labelsStates = [
  "list",
  "edit-shape",
  "no-machine-fits",
  "loading",
  "error",
] as const;
export type LabelsState = (typeof labelsStates)[number];

const columns: readonly Column[] = [
  { key: "label", label: "Label", flex: 2 },
  { key: "shape", label: "Shape", flex: 2 },
  { key: "jobs", label: "Jobs, 30 days", width: 120, align: "right" },
  { key: "edit", label: "", width: 90, align: "right" },
];

function shapeText(label: Label): string {
  return dotJoin(
    unbroken(`${label.vcpu} vCPU`),
    unbroken(`${label.memoryGb} GB`),
    unbroken(`${label.diskGb} GB disk`),
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

function EditLabel({ label }: { label: Label }) {
  return label.name === "ciproxy-8vcpu" ? (
    <LinkAction to={paths.editShape} size="sm" icon={Pencil}>
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
      return <LabelName label={label} />;
    case "shape":
      return <span>{shapeText(label)}</span>;
    case "jobs":
      return <span className="num">{formatCount(label.jobs)}</span>;
    default:
      return <EditLabel label={label} />;
  }
}

function LabelList({
  viewport,
  loading,
}: {
  viewport: Viewport;
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
            <span className="small">{shapeText(label)}</span>
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

export function LabelsScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: LabelsState;
}) {
  const overlay =
    state === "edit-shape" || state === "no-machine-fits" ? (
      <ShapeDialog tooLarge={state === "no-machine-fits"} />
    ) : undefined;
  return (
    <SettingsFrame
      viewport={viewport}
      active={paths.settings}
      overlay={overlay}
    >
      {state === "error" ? (
        <Panel>
          <ReadFailure subject="settings" retry={paths.settings} />
        </Panel>
      ) : (
        <>
          <p className="small muted settings-intro">
            A workflow picks a label in{" "}
            <span className="inline-code">runs-on</span>. Each label starts a
            fresh VM of its shape, on your machines first and on Boat when they
            are busy.
          </p>
          <Panel
            title="Labels"
            flush
            action={
              <LinkAction to={paths.shapeTooLarge} size="sm" icon={Plus}>
                Add label
              </LinkAction>
            }
          >
            <LabelList viewport={viewport} loading={state === "loading"} />
          </Panel>
          <p className="caption">
            A slot on your machines is {unbroken("2 vCPU")} and{" "}
            {unbroken("8 GB")}. A larger shape takes several slots.
          </p>
        </>
      )}
    </SettingsFrame>
  );
}
