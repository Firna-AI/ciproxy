// Setup of a new organization: install the GitHub App, add a machine with one
// command, connect Boat, and run the first job. One screen; the state picks
// the open step. The Boat step can be skipped: it then shows a dash and a
// link to Settings under Overflow, and it does not count as done.

import { Check, Minus } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { TextLink } from "../../components/actions.tsx";
import { PageHeader } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { dotJoin, monthly } from "../../fixtures/format.ts";
import { boat } from "../../fixtures/machines.ts";
import { org, workspace } from "../../fixtures/org.ts";
import { BoatStep, FirstJobStep, MachineStep } from "./step-bodies.tsx";
import "./onboarding.css";

export const onboardingStates = [
  "machine-waiting",
  "machine-connected",
  "machine-no-kvm",
  "boat",
  "boat-invalid",
  "boat-connect-failed",
  "boat-rejected",
  "boat-unavailable",
  "boat-load-failed",
  "boat-skipped",
  "first-job-waiting",
  "first-job-passed",
] as const;

export type OnboardingState = (typeof onboardingStates)[number];

type StepKey = "github" | "machine" | "boat" | "job";
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
    todo: "Run one command on a Linux machine.",
    done: "hetzner-ax42-1 · 6 slots",
  },
  {
    key: "boat",
    title: "Connect Boat for overflow",
    todo: "Jobs start on Boat when your machines are busy.",
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
  "machine-connected": "machine",
  "machine-no-kvm": "machine",
  boat: "boat",
  "boat-invalid": "boat",
  "boat-connect-failed": "boat",
  "boat-rejected": "boat",
  "boat-unavailable": "boat",
  "boat-load-failed": "boat",
  "boat-skipped": "job",
  "first-job-waiting": "job",
  "first-job-passed": "job",
};

function statusOf(state: OnboardingState, key: StepKey): StepStatus {
  const order: StepKey[] = ["github", "machine", "boat", "job"];
  const open = openStep[state];
  if (state === "boat-skipped" && key === "boat") return "skipped";
  if (state === "first-job-passed") return "done";
  if (open === key) return "active";
  return order.indexOf(key) < order.indexOf(open) ? "done" : "todo";
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
                    {status === "done" ? step.done : step.todo}
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

/** The line of a skipped Boat step, which leads to Settings under Overflow. */
function SkippedLine() {
  return (
    <p className="muted small">
      Skipped for now. Connect Boat in{" "}
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
    case "boat":
      return <BoatStep state={state} />;
    case "job":
      return <FirstJobStep passed={state === "first-job-passed"} />;
    case "github":
      return null;
  }
}
