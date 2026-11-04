// The reference page of labels and VM sizes,
// ciproxy.dev/docs/reference/labels: the default labels with their sizes and
// slots, how slots work, custom labels, and where a job with a label runs. A
// running job also holds the disk of its size, as design call 34 says, and a
// label keeps its name, as design call 33 says.

import type { Viewport } from "@mokly/mokly";

import { Notice } from "../../components/feedback.tsx";
import { CodeBlock } from "../../components/figures.tsx";
import { paths } from "../../components/paths.ts";
import { Tag } from "../../components/status.tsx";
import { unbroken } from "../../fixtures/format.ts";
import {
  ax42,
  machineSlots,
  slotSize,
  slotsFor,
} from "../../fixtures/hardware.ts";
import { boat } from "../../fixtures/machines.ts";
import { labels, waitChoices } from "../../fixtures/settings.ts";
import { sizePhrase, sizeText } from "../../fixtures/sizes.ts";
import { DocsShell, type TocItem } from "./docs-shell.tsx";
import { DocsH2, DocsTitle } from "./docs-parts.tsx";

const toc: readonly TocItem[] = [
  { id: "default-labels", label: "Default labels" },
  { id: "slots", label: "Slots" },
  { id: "custom-labels", label: "Custom labels" },
  { id: "where-jobs-run", label: "Where a job runs" },
];

/** The largest default label, for the examples. */
const large = labels.reduce((largest, label) =>
  label.vcpu > largest.vcpu ? label : largest,
);

/** "0, 15, 30, 60, or 120 seconds". */
function waitText(): string {
  const values = waitChoices.map(String);
  const last = values.pop();
  return `${values.join(", ")}, or ${last ?? ""} seconds`;
}

function LabelTable() {
  return (
    <div className="docs-table-frame">
      <table className="docs-table">
        <thead>
          <tr>
            <th scope="col">Label</th>
            <th scope="col">Size</th>
            <th scope="col">Slots</th>
          </tr>
        </thead>
        <tbody>
          {labels.map((label) => (
            <tr key={label.name}>
              <td>
                <span className="row-wrap">
                  <span className="inline-code">{label.name}</span>
                  {label.name === "ciproxy" ? <Tag>Default</Tag> : null}
                </span>
              </td>
              <td>{sizeText(label)}</td>
              <td className="num">{slotsFor(label)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function LabelsReferenceScreen({ viewport }: { viewport: Viewport }) {
  return (
    <DocsShell viewport={viewport} page={paths.labelsReference} toc={toc}>
      <DocsTitle
        title="Labels and VM sizes"
        lead={
          <>
            The label in <span className="inline-code">runs-on</span> picks the
            size of the VM that runs the job.
          </>
        }
      />
      <div className="docs-flow">
        <p>
          Each job starts in a fresh VM of the size of its label: on your
          machines first, and on Boat when they are full.
        </p>
        <DocsH2 id="default-labels">Default labels</DocsH2>
        <p>Every workspace starts with these labels.</p>
        <LabelTable />
        <CodeBlock
          label=".github/workflows/e2e.yml"
          lines={[
            "jobs:",
            "  e2e:",
            `    runs-on: ${large.name}`,
            "    steps:",
            "      - uses: actions/checkout@v4",
            "      - run: npm run test:e2e",
          ]}
        />
        <DocsH2 id="slots">Slots</DocsH2>
        <p>
          Each machine splits its CPU and memory into slots of{" "}
          {sizePhrase(slotSize)}. A job takes as many slots as its size needs,
          so a job on <span className="inline-code">{large.name}</span> takes{" "}
          {slotsFor(large)} slots.
        </p>
        <p>
          A running job also holds the disk of its size. The host keeps a
          quarter of the disk, and the machine cache uses the rest.
        </p>
        <p>
          The agent reports the slots of a machine when it connects. A Hetzner
          AX42 with {unbroken(`${ax42.threads} threads`)} and{" "}
          {unbroken(`${ax42.memoryGb} GB`)} has{" "}
          {unbroken(`${machineSlots(ax42)} slots`)}.
        </p>
        <DocsH2 id="custom-labels">Custom labels</DocsH2>
        <p>
          Add a label with its own size in <strong>Settings</strong>, under{" "}
          <strong>Labels</strong>. Each label runs on your machines, on Boat, or
          on both.
        </p>
        <p>
          A label name follows the rules of GitHub for runner labels, and it
          never changes. To rename a label, add a new one, move your workflows
          to it, and delete the old one.
        </p>
        <Notice tone="info" title="Sizes larger than your machines">
          A job whose size fits none of your machines runs on Boat only. When no
          Boat size fits either, the label setting{" "}
          <strong>When nothing fits</strong> decides: its jobs wait, or run on
          the largest Boat size.
        </Notice>
        <DocsH2 id="where-jobs-run">Where a job runs</DocsH2>
        <p>
          A job waits up to {boat.waitSeconds} seconds for a free slot on your
          machines, then starts on Boat. Set the wait in{" "}
          <strong>Settings</strong>, under <strong>Overflow</strong>:{" "}
          {waitText()}.
        </p>
        <p>
          At the Boat spend limit, jobs run on your machines only and wait there
          for a free slot.
        </p>
      </div>
    </DocsShell>
  );
}
