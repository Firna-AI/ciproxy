// The quickstart, ciproxy.dev/docs/quickstart: what a team needs, then five
// steps from sign-in to the first job on an own machine. The code blocks are
// the install command of Add machine and the one-line workflow change.

import type { Viewport } from "@mokly/mokly";

import { TextLink } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { CodeBlock } from "../../components/figures.tsx";
import { installCommand } from "../../components/install-command.tsx";
import { paths } from "../../components/paths.ts";
import { unbroken } from "../../fixtures/format.ts";
import { defaultWarmRunners } from "../../fixtures/machines.ts";
import { jobSteps, runsOnDiff, workflowFile } from "../../fixtures/workflow.ts";
import { DocsShell, type TocItem } from "./docs-shell.tsx";
import {
  CardGrid,
  DocsH2,
  DocsTitle,
  LinkCard,
  Step,
  Steps,
} from "./docs-parts.tsx";

const toc: readonly TocItem[] = [
  { id: "before-you-start", label: "Before you start" },
  { id: "set-up", label: "Set up ciproxy" },
  { id: "sign-in", label: "Sign in", nested: true },
  { id: "add-a-machine", label: "Add a machine", nested: true },
  { id: "connect-boat", label: "Connect Boat", nested: true },
  { id: "change-runs-on", label: "Change runs-on", nested: true },
  { id: "push", label: "Push the change", nested: true },
  { id: "next-steps", label: "Next steps" },
];

export function QuickstartScreen({ viewport }: { viewport: Viewport }) {
  return (
    <DocsShell viewport={viewport} page={paths.quickstart} toc={toc}>
      <DocsTitle
        title="Quickstart"
        lead="Run your first GitHub Actions job on your own machine."
      />
      <div className="docs-flow">
        <DocsH2 id="before-you-start">Before you start</DocsH2>
        <Notice tone="info" title="You need">
          A GitHub organization with private repositories, and an x64 Linux
          machine with KVM, {unbroken("4 cores")}, {unbroken("16 GB")} of
          memory, and {unbroken("250 GB")} of disk or more.
        </Notice>
        <DocsH2 id="set-up">Set up ciproxy</DocsH2>
        <Steps>
          <Step id="sign-in" title="Sign in and install the GitHub App">
            <p>
              <TextLink to={paths.signIn}>Sign in to ciproxy</TextLink> with
              GitHub. Then install the ciproxy GitHub App on your organization,
              and pick the repositories that may use it.
            </p>
          </Step>
          <Step id="add-a-machine" title="Add a machine">
            <p>
              In ciproxy, open <strong>Machines</strong> and select{" "}
              <strong>Add machine</strong>. Run the command that it shows on
              your Linux machine:
            </p>
            <CodeBlock label="Terminal" lines={[installCommand]} />
            <p>
              The command holds a token for your workspace, which works for one
              hour. The machine shows in Machines with its slots and{" "}
              {defaultWarmRunners} warm runners.
            </p>
          </Step>
          <Step id="connect-boat" title="Connect Boat">
            <p>
              To start jobs on Boat when your machines are full, paste a Boat
              API key in <strong>Settings</strong>, under{" "}
              <strong>Overflow</strong>. You can connect Boat later.
            </p>
          </Step>
          <Step id="change-runs-on" title="Change runs-on">
            <p>
              In a workflow file, change the{" "}
              <span className="inline-code">runs-on</span> line of a job:
            </p>
            <CodeBlock
              label={workflowFile}
              lines={[...runsOnDiff, ...jobSteps]}
            />
          </Step>
          <Step id="push" title="Push the change">
            <p>
              Push to GitHub. The job starts on a warm runner on your machine,
              and <strong>Jobs</strong> in ciproxy shows where it ran.
            </p>
          </Step>
        </Steps>
        <DocsH2 id="next-steps">Next steps</DocsH2>
        <CardGrid>
          <LinkCard
            title="Labels and VM sizes"
            text="Give each job the VM size that it needs."
            to={paths.labelsReference}
          />
          <LinkCard
            title="How ciproxy works"
            text="Warm runners, slots, overflow, and the cache."
          />
        </CardGrid>
      </div>
    </DocsShell>
  );
}
