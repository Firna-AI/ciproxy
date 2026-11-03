// Labels and machine shapes. A workflow picks a label in runs-on; each label
// starts a VM of its shape on an own machine, or on Boat.

import { Pencil, Plus } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { Badge } from "@firna/ui/badge";
import { Table } from "@firna/ui/table";

import { LinkAction } from "../../components/actions.tsx";
import { ReadFailure } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { formatCount } from "../../fixtures/format.ts";
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

const columns = [
  { key: "label", label: "Label", flex: 2 },
  { key: "shape", label: "Shape", flex: 2 },
  { key: "jobs", label: "Jobs, 30 days", width: 120, align: "right" as const },
  { key: "edit", label: "", width: 90, align: "right" as const },
];

function shapeText(label: Label): string {
  return `${label.vcpu} vCPU · ${label.memoryGb} GB · ${label.diskGb} GB disk`;
}

function cell(label: Label, key: string) {
  switch (key) {
    case "label":
      return (
        <span className="row">
          <span className="inline-code">{label.name}</span>
          {label.name === "ciproxy" ? (
            <Badge tone="neutral" size="sm">
              Default
            </Badge>
          ) : null}
        </span>
      );
    case "shape":
      return <span>{shapeText(label)}</span>;
    case "jobs":
      return <span className="num">{formatCount(label.jobs)}</span>;
    default:
      return label.name === "ciproxy-8vcpu" ? (
        <LinkAction to={paths.editShape} size="sm" icon={Pencil}>
          Edit
        </LinkAction>
      ) : null;
  }
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
          <ReadFailure retry={paths.settings} />
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
            {viewport === "mobile" ? (
              <ul className="label-cards">
                {labels.map((label) => (
                  <li key={label.name} className="label-card">
                    <span className="inline-code">{label.name}</span>
                    <span className="small">{shapeText(label)}</span>
                    <span className="caption">
                      {formatCount(label.jobs)} jobs in 30 days
                    </span>
                  </li>
                ))}
              </ul>
            ) : (
              <Table
                columns={columns}
                rows={state === "loading" ? [] : [...labels]}
                rowKey={(label) => label.name}
                cell={cell}
                loading={state === "loading"}
                loadingRowCount={3}
              />
            )}
          </Panel>
          <p className="caption">
            A slot on your machines is 2 vCPU and 8 GB. A larger shape takes
            several slots.
          </p>
        </>
      )}
    </SettingsFrame>
  );
}
