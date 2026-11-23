// The docs page How ciproxy works, ciproxy.dev/docs/how-ciproxy-works: the
// parts of ciproxy, the path of a job on your machines, warm runners, slots
// and disk, overflow to Boat, the cache, and the job image. The picture under
// the intro shows the parts and the path of one job, and its numbers are the
// steps of "A job on your machines". The text follows the page of the site
// sentence by sentence. On a Linux machine, warm runners serve the label
// ciproxy, and on a Mac the label ciproxy-macos, as the job routing doc says.

import type { Viewport } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { unbroken } from "../../fixtures/format.ts";
import { defaultWarmRunners } from "../../fixtures/machines.ts";
import { host } from "../../fixtures/org.ts";
import { macosLabel } from "../../fixtures/settings.ts";
import { DocsShell, type TocItem } from "./docs-shell.tsx";
import { DocsH2, DocsTitle } from "./docs-parts.tsx";
import {
  CacheSection,
  JobImageSection,
  OverflowSection,
  SlotsSection,
} from "./how-ciproxy-works-sections.tsx";
import { JobPathFigure } from "./job-path-figure.tsx";

const toc: readonly TocItem[] = [
  { id: "the-parts", label: "The parts" },
  { id: "a-job-on-your-machines", label: "A job on your machines" },
  { id: "warm-runners", label: "Warm runners" },
  { id: "slots-and-disk", label: "Slots and disk" },
  { id: "overflow-to-boat", label: "Overflow to Boat" },
  { id: "the-cache", label: "The cache" },
  { id: "the-job-image", label: "The job image" },
];

function Parts() {
  return (
    <ul className="docs-list">
      <li>
        <strong>The GitHub App</strong> connects your organizations to ciproxy.
        GitHub sends ciproxy the jobs of its labels, and gives each runner its
        job.
      </li>
      <li>
        <strong>The control plane</strong> is the hosted service of ciproxy. It
        decides how many runners start, and where.
      </li>
      <li>
        <strong>The dashboard</strong> at{" "}
        <span className="inline-code">{host}</span> is where you add machines,
        set the labels, and see the jobs.
      </li>
      <li>
        <strong>The agent</strong> runs on each of your machines. It connects
        out to ciproxy, and starts a fresh VM for each runner. The agent is open
        source under Apache-2.0.
      </li>
      <li>
        <strong>Your Boat account</strong> runs the jobs that your machines have
        no room for, and the jobs of labels that allow only Boat. Boat bills you
        for them.
      </li>
    </ul>
  );
}

function JobSteps() {
  return (
    <ol className="docs-list">
      <li>
        GitHub queues a job whose <span className="inline-code">runs-on</span>{" "}
        names a ciproxy label, and tells ciproxy about it.
      </li>
      <li>
        A ready warm runner takes a job of the label{" "}
        <span className="inline-code">ciproxy</span> or{" "}
        <span className="inline-code">{macosLabel.name}</span> at once. For a
        job of another label, ciproxy starts a runner in free slots of one of
        your machines.
      </li>
      <li>The runner runs the job in a fresh VM that no other job used.</li>
      <li>When the job ends, the agent ends the VM.</li>
    </ol>
  );
}

function WarmRunners() {
  return (
    <ul className="docs-list">
      <li>
        The warm runner count of each machine is {defaultWarmRunners} by
        default. Set the count on the page of the machine.
      </li>
      <li>
        ciproxy starts warm runners up to that count, only in free slots and
        disk that no waiting job needs.
      </li>
      <li>
        On a Linux machine, warm runners serve the label{" "}
        <span className="inline-code">ciproxy</span>. On a Mac, they serve the
        label <span className="inline-code">{macosLabel.name}</span>. A job of
        another label gets a new runner in a fresh VM.
      </li>
      <li>
        A warm runner never keeps a waiting job from your machines. ciproxy
        stops warm runners to free the slots and the disk that a waiting job
        needs.
      </li>
      <li>A draining machine keeps no warm runners.</li>
      <li>
        In a workspace with several organizations, the warm runners of a machine
        go to each organization by its jobs for the warm runner label in the
        last {unbroken("24 hours")}.
      </li>
    </ul>
  );
}

export function HowCiproxyWorksScreen({ viewport }: { viewport: Viewport }) {
  return (
    <DocsShell viewport={viewport} page={paths.howCiproxyWorks} toc={toc}>
      <DocsTitle
        title="How ciproxy works"
        lead="Warm runners, slots, overflow, and the cache."
      />
      <div className="docs-flow">
        <p>
          ciproxy is the runner provider of its labels on GitHub. A job uses
          ciproxy when its <span className="inline-code">runs-on</span> line
          names a ciproxy label. ciproxy then starts a runner for the job on one
          of your machines, or in your Boat account.
        </p>
        <JobPathFigure />
        <DocsH2 id="the-parts">The parts</DocsH2>
        <Parts />
        <DocsH2 id="a-job-on-your-machines">A job on your machines</DocsH2>
        <JobSteps />
        <p>
          GitHub gives each waiting job to any idle runner of its label. ciproxy
          decides how many runners start and where, not which job a runner
          takes. <strong>Jobs</strong> in the dashboard shows where each job
          ran, and how long it waited.
        </p>
        <DocsH2 id="warm-runners">Warm runners</DocsH2>
        <WarmRunners />
        <SlotsSection />
        <OverflowSection />
        <CacheSection />
        <JobImageSection />
      </div>
    </DocsShell>
  );
}
