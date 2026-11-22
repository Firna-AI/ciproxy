// The guide Add a machine, ciproxy.dev/docs/guides/add-a-machine: what a
// Linux machine and a Mac need, the install command with its token, the
// machines that run fewer jobs, what follows the connection, and a repeat
// install. The same command connects a Linux machine and a Mac. The step that
// shows the connected machine says that a new machine first downloads the job
// image. No text promises a time, because a slow link makes the download long.

import type { Viewport } from "@mokly/mokly";

import { TextLink } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { CodeBlock } from "../../components/figures.tsx";
import { docsInstallCommand } from "../../components/install-command.tsx";
import { paths } from "../../components/paths.ts";
import { unbroken } from "../../fixtures/format.ts";
import { machineSlots } from "../../fixtures/hardware.ts";
import { macMini } from "../../fixtures/macs.ts";
import { defaultWarmRunners } from "../../fixtures/machines.ts";
import { macosVmLimit, minimumMacos } from "../../fixtures/platform.ts";
import { DocsShell, type TocItem } from "./docs-shell.tsx";
import { DocsH2, DocsTitle } from "./docs-parts.tsx";

const toc: readonly TocItem[] = [
  { id: "what-the-machine-needs", label: "What the machine needs" },
  { id: "add-a-mac", label: "Add a Mac" },
  { id: "run-the-install-command", label: "Run the install command" },
  {
    id: "when-a-machine-runs-fewer-jobs",
    label: "When a machine runs fewer jobs",
  },
  { id: "after-the-machine-connects", label: "After the machine connects" },
  { id: "run-the-command-again", label: "Run the command again" },
];

const network = (
  <li>
    Outbound HTTPS to ciproxy, to GitHub, and to{" "}
    <span className="inline-code">get.ciproxy.dev</span>. The machine opens no
    inbound port.
  </li>
);

function MacNeeds() {
  return (
    <ul className="docs-list">
      <li>A Mac with Apple Silicon, M1 or later.</li>
      <li>
        {unbroken(minimumMacos)} or later. A macOS VM cannot run a newer macOS
        than its Mac, so the Mac needs the macOS of the job image.
      </li>
      <li>
        {unbroken("16 GB")} of memory and {unbroken("500 GB")} of disk or more.
      </li>
      {network}
    </ul>
  );
}

function Needs() {
  return (
    <ul className="docs-list">
      <li>x64 Linux with systemd, {unbroken("cgroup v2")}, and KVM.</li>
      <li>
        A supported distribution: Debian, Ubuntu, Fedora, RHEL, Rocky Linux, or
        AlmaLinux.
      </li>
      <li>
        {unbroken("4 cores")}, {unbroken("16 GB")} of memory, and{" "}
        {unbroken("250 GB")} of disk or more.
      </li>
      {network}
    </ul>
  );
}

function InstallSteps() {
  return (
    <ol className="docs-list">
      <li>
        In ciproxy, open <strong>Machines</strong> and select{" "}
        <strong>Add machine</strong>. For your first machine, Setup shows the
        same command.
      </li>
      <li>
        <div className="docs-flow">
          <p>Copy the command. It holds an install token of your workspace:</p>
          <CodeBlock label="Terminal" lines={[docsInstallCommand]} />
        </div>
      </li>
      <li>
        Run the command on the machine, in Terminal on a Mac. It installs the
        agent, prepares the machine, and connects it to your workspace.
      </li>
      <li>
        Keep the page open. It shows that it waits for the machine. Then it
        shows the host name of the machine and what the agent reported: threads
        or cores, memory, virtualization, and slots. A new machine first
        downloads the job image and takes jobs when it is ready.
      </li>
    </ol>
  );
}

function FewerJobs() {
  return (
    <ul className="docs-list">
      <li>
        <strong>No KVM:</strong> the machine connects, but it cannot start VMs,
        so it gets no slots. Turn on virtualization for the machine, then run
        the command again.
      </li>
      <li>
        <strong>A Mac with an older macOS:</strong> the Mac connects, but it
        cannot start VMs, so it gets no slots. Update the Mac to{" "}
        {unbroken(minimumMacos)} or later, then run the command again.
      </li>
      <li>
        <strong>Too little CPU or memory:</strong> a machine without room for
        one slot after the host reserve connects and gets no slots. ciproxy says
        why.
      </li>
      <li>
        <strong>Too little disk:</strong> each job also holds disk, so a machine
        can run fewer jobs of the label{" "}
        <span className="inline-code">ciproxy</span> at a time than its slots
        hold. ciproxy then says how many such jobs the machine runs at a time.
        Add disk to run more.
      </li>
    </ul>
  );
}

function AfterItConnects() {
  return (
    <ul className="docs-list">
      <li>
        <strong>Warm runners:</strong> the warm runner count of each machine is{" "}
        {defaultWarmRunners} by default. ciproxy starts warm runners up to that
        count when the machine has free slots and disk for them. Change the
        count on the page of the machine.
      </li>
      <li>
        <strong>Cost:</strong> enter the monthly cost of the machine on its
        page. The savings report compares that cost with the cost of hosted
        runners.
      </li>
      <li>
        <strong>Plan:</strong> each machine counts toward the machines of your
        plan, also while it is offline or draining. Above your plan, jobs keep
        running on every machine. Plans and starts gives the numbers.
      </li>
      <li>
        <strong>Updates:</strong> the agent updates itself. The page of the
        machine shows whether its agent is up to date.
      </li>
    </ul>
  );
}

export function AddMachineGuideScreen({ viewport }: { viewport: Viewport }) {
  return (
    <DocsShell viewport={viewport} page={paths.addMachineGuide} toc={toc}>
      <DocsTitle
        title="Add a machine"
        lead="Connect a Linux machine or a Mac to your workspace with one command."
      />
      <div className="docs-flow">
        <p>
          ciproxy runs your jobs on your own machines first. Each machine runs
          the open-source agent, which starts a fresh VM for each job.
        </p>
        <DocsH2 id="what-the-machine-needs">What the machine needs</DocsH2>
        <p>A Linux machine needs:</p>
        <Needs />
        <p>
          A machine with more threads and memory has more slots, so it runs more
          jobs at a time.{" "}
          <TextLink to={paths.labelsReference} fragment="slots">
            Labels and VM sizes
          </TextLink>{" "}
          gives the rule.
        </p>
        <Notice tone="info" title="Combine the disks of a CI machine">
          A machine used only for CI can combine its disks into one filesystem,
          for example with RAID0 or one LVM volume. Its jobs, its machine cache,
          and its Docker disks are disposable.
        </Notice>
        <DocsH2 id="add-a-mac">Add a Mac</DocsH2>
        <p>
          A Mac runs the jobs of macOS labels, each in a fresh macOS VM with
          Xcode. A Mac needs:
        </p>
        <MacNeeds />
        <p>
          A Mac uses the same slots as a Linux machine, but it runs at most{" "}
          {macosVmLimit} jobs at a time, because Apple allows at most{" "}
          {macosVmLimit} macOS VMs on one Mac. A Mac with{" "}
          {unbroken(`${macMini.memoryGb} GB`)} of memory has room for{" "}
          {machineSlots(macMini)} slot, so it runs {machineSlots(macMini)} job
          at a time. Jobs of a macOS label never run on Boat. They wait for a
          free Mac. A Mac counts as one machine of your plan, as a Linux machine
          does.
        </p>
        <p>
          Run the same install command in Terminal on the Mac. The agent runs as
          a launchd service.
        </p>
        <DocsH2 id="run-the-install-command">Run the install command</DocsH2>
        <p>Only an Admin of the workspace sees the install command.</p>
        <InstallSteps />
        <p>
          The install token works for one hour. Within that hour, it adds any
          number of machines, so you can run the same command on each of them.
          To end the token before its hour ends, select{" "}
          <strong>Revoke token</strong> next to the command. The machines that
          joined with it stay connected.
        </p>
        <p>Agent install options says what the command does on the machine.</p>
        <DocsH2 id="when-a-machine-runs-fewer-jobs">
          When a machine runs fewer jobs
        </DocsH2>
        <FewerJobs />
        <p>The page of a machine that cannot run jobs offers to remove it.</p>
        <DocsH2 id="after-the-machine-connects">
          After the machine connects
        </DocsH2>
        <AfterItConnects />
        <DocsH2 id="run-the-command-again">Run the command again</DocsH2>
        <p>
          Run the install command again on a registered machine to repair the
          agent. The machine keeps its registration, its place in the machine
          count, its drain state, and its jobs.
        </p>
        <p>
          A machine that lost its registration, for example after a new install
          of its operating system, joins as a new machine. Its old entry stays
          in <strong>Machines</strong>, offline, until you remove it, as Remove
          a machine says.
        </p>
      </div>
    </DocsShell>
  );
}
