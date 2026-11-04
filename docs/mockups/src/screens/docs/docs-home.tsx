// The docs home, ciproxy.dev/docs: what ciproxy does, where to start, how a
// job runs, and what a team needs. The search state opens the search dialog
// over the page.

import type { Viewport } from "@mokly/mokly";

import { TextLink } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import { boat } from "../../fixtures/machines.ts";
import { freePlanNote } from "../../fixtures/tiers.ts";
import { DocsShell, type TocItem } from "./docs-shell.tsx";
import { CardGrid, DocsH2, DocsTitle, LinkCard } from "./docs-parts.tsx";
import { SearchDialog } from "./search-dialog.tsx";

export const docsHomeStates = ["page", "search"] as const;
export type DocsHomeState = (typeof docsHomeStates)[number];

const toc: readonly TocItem[] = [
  { id: "start-here", label: "Start here" },
  { id: "how-a-job-runs", label: "How a job runs" },
  { id: "what-you-need", label: "What you need" },
];

export function DocsHomeScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: DocsHomeState;
}) {
  return (
    <DocsShell
      viewport={viewport}
      page={paths.docsHome}
      toc={toc}
      overlay={
        state === "search" ? <SearchDialog viewport={viewport} /> : undefined
      }
    >
      <DocsTitle
        title="Overview"
        lead="ciproxy runs your GitHub Actions jobs on your own machines first. When they are full, it starts runners in your Boat account."
      />
      <div className="docs-flow">
        <DocsH2 id="start-here">Start here</DocsH2>
        <CardGrid>
          <LinkCard
            title="Quickstart"
            text="Run your first job on your own machine."
            to={paths.quickstart}
          />
          <LinkCard
            title="How ciproxy works"
            text="Warm runners, slots, overflow, and the cache."
          />
          <LinkCard
            title="Labels and machine sizes"
            text="Pick the size of the VM for each job."
            to={paths.labelsReference}
          />
          <LinkCard
            title="Security"
            text="Job isolation, the agent connection, and GitHub access."
          />
        </CardGrid>
        <DocsH2 id="how-a-job-runs">How a job runs</DocsH2>
        <ol className="docs-list">
          <li>
            GitHub queues a job whose workflow says{" "}
            <span className="inline-code">runs-on: ciproxy</span>.
          </li>
          <li>
            A warm runner on one of your machines takes the job and runs it in a
            fresh VM.
          </li>
          <li>
            When every slot is busy for {boat.waitSeconds} seconds, ciproxy
            starts a runner in your Boat account instead.
          </li>
          <li>
            The job reads and writes its{" "}
            <span className="inline-code">actions/cache</span> entries on the
            disk of the machine.
          </li>
        </ol>
        <DocsH2 id="what-you-need">What you need</DocsH2>
        <ul className="docs-list">
          <li>A GitHub organization with private repositories.</li>
          <li>
            One or more x64 Linux machines with KVM, 4 cores, and 16 GB of
            memory or more.
          </li>
          <li>A Boat account for overflow. You can connect it later.</li>
        </ul>
        <p>
          {freePlanNote} See <TextLink to={paths.pricing}>Pricing</TextLink> for
          the plans.
        </p>
      </div>
    </DocsShell>
  );
}
