// Setup of a new organization: install the GitHub App, add a machine with one
// command, connect an overflow provider, and run the first job. One screen;
// the state picks the open step. The provider step can be skipped: it then
// shows a dash and a link to Settings under Overflow, and it does not count
// as done. A machine that connected gets its first job image before it takes
// a job. Setup goes on meanwhile, and the machine step and the first job name
// the image. The same command connects a Mac, which runs the jobs of macOS
// labels.

import { Check, Minus } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { TextLink } from "../../components/actions.tsx";
import { PageHeader } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { dotJoin, monthly } from "../../fixtures/format.ts";
import { org, workspace } from "../../fixtures/org.ts";
import { boat } from "../../fixtures/overflow.ts";
import { ProviderStep } from "./provider-step.tsx";
import { FirstJobStep, MachineStep, setupMachine } from "./step-bodies.tsx";
import "./onboarding.css";

export const onboardingStates = [
  "machine-waiting",
  "machine-preparing",
  "machine-connected",
  "machine-no-kvm",
  "machine-mac",
  "provider",
  "boat-wallet",
  "boat-invalid",
  "boat-connect-failed",
  "boat-rejected",
  "second-provider",
  "provider-unavailable",
  "no-provider-available",
  "provider-load-failed",
  "provider-skipped",
  "first-job-waiting",
  "first-job-preparing",
  "first-job-passed",
] as const;

export type OnboardingState = (typeof onboardingStates)[number];

type StepKey = "github" | "machine" | "provider" | "job";
type StepStatus = "done" | "active" | "todo" | "skipped";

const steps: readonly {
  key: StepKey;
  title: string;
  todo: string;
  done: string;
}[] = [
  {
    key: "github",
    title: "Install the GitHub App",
    todo: "",
    done: `Installed on ${org.login} · ${org.repositories} repositories`,
  },
  {
    key: "machine",
    title: "Add a machine",
    todo: "Run one command on a Linux machine or a Mac.",
    done: `${setupMachine} · 6 slots`,
  },
  {
    key: "provider",
    title: "Connect an overflow provider",
    todo: "Jobs start on a provider when your machines are busy.",
    done: dotJoin(
      `Connected to the Boat account ${boat.account}`,
      `spend limit ${monthly(boat.spendLimit)}`,
    ),
  },
  {
    key: "job",
    title: "Run the first job",
    todo: "Change runs-on in one workflow.",
    done: "Passed in 6m 12s",
  },
];

const openStep: Record<OnboardingState, StepKey> = {
  "machine-waiting": "machine",
  "machine-preparing": "machine",
  "machine-connected": "machine",
  "machine-no-kvm": "machine",
  "machine-mac": "machine",
  provider: "provider",
  "boat-wallet": "provider",
  "boat-invalid": "provider",
  "boat-connect-failed": "provider",
  "boat-rejected": "provider",
  "second-provider": "provider",
  "provider-unavailable": "provider",
  "no-provider-available": "provider",
  "provider-load-failed": "provider",
  "provider-skipped": "job",
  "first-job-waiting": "job",
  "first-job-preparing": "job",
  "first-job-passed": "job",
};

/** The states after Skip for now, while no provider is connected. */
function skipsProvider(state: OnboardingState): boolean {
  return state === "provider-skipped" || state === "first-job-preparing";
}

/** The states in which the machine gets its first job image. */
function machinePreparing(state: OnboardingState): boolean {
  return state === "machine-preparing" || state === "first-job-preparing";
}

function statusOf(state: OnboardingState, key: StepKey): StepStatus {
  const order: StepKey[] = ["github", "machine", "provider", "job"];
  const open = openStep[state];
  if (skipsProvider(state) && key === "provider") return "skipped";
  if (state === "first-job-passed") return "done";
  if (open === key) return "active";
  return order.indexOf(key) < order.indexOf(open) ? "done" : "todo";
}

/** The line of a closed step. A done machine step names the image wait. */
function closedLine(
  step: (typeof steps)[number],
  status: StepStatus,
  state: OnboardingState,
): string {
  if (status !== "done") return step.todo;
  if (step.key === "machine" && machinePreparing(state)) {
    return dotJoin(setupMachine, "getting the job image");
  }
  return step.done;
}

function doneCount(state: OnboardingState): number {
  return steps.filter((step) => statusOf(state, step.key) === "done").length;
}

export function OnboardingScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: OnboardingState;
}) {
  const done = doneCount(state);
  return (
    <AppShell
      viewport={viewport}
      tier="free"
      setup={{ done, total: steps.length, active: true }}
    >
      <PageHeader
        title="Set up ciproxy"
        subtitle={`${done} of ${steps.length} steps done for ${workspace.name}`}
      />
      <ol className="setup-steps">
        {steps.map((step, index) => {
          const status = statusOf(state, step.key);
          const open = status === "active" || openStep[state] === step.key;
          return (
            <li key={step.key} className={`setup-step is-${status}`}>
              <span className="setup-marker" aria-hidden="true">
                {status === "done" ? (
                  <Check size={14} strokeWidth={2.5} />
                ) : status === "skipped" ? (
                  <Minus size={14} strokeWidth={2.5} />
                ) : (
                  index + 1
                )}
              </span>
              <div className="setup-content">
                <h2 className="setup-title">{step.title}</h2>
                {open ? (
                  <StepBody state={state} step={step.key} />
                ) : status === "skipped" ? (
                  <SkippedLine />
                ) : (
                  <p className="muted small">
                    {closedLine(step, status, state)}
                  </p>
                )}
              </div>
            </li>
          );
        })}
      </ol>
    </AppShell>
  );
}

/** The line of a skipped provider step, which leads to Settings. */
function SkippedLine() {
  return (
    <p className="muted small">
      Skipped for now. Connect a provider in{" "}
      <TextLink to={paths.overflowNotConnected}>
        Settings under Overflow
      </TextLink>
      .
    </p>
  );
}

function StepBody({ state, step }: { state: OnboardingState; step: StepKey }) {
  switch (step) {
    case "machine":
      return <MachineStep state={state} />;
    case "provider":
      return <ProviderStep state={state} />;
    case "job":
      return <FirstJobStep state={state} />;
    case "github":
      return null;
  }
}
