// Setup of a new organization: install the GitHub App, add a machine with one
// command, connect Boat, and run the first job. One screen; the state picks
// the open step.

import { Check } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { PageHeader } from "../../components/page.tsx";
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
  "first-job-waiting",
  "first-job-passed",
] as const;

export type OnboardingState = (typeof onboardingStates)[number];

type StepKey = "github" | "machine" | "boat" | "job";
type StepStatus = "done" | "active" | "todo";

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
  "first-job-waiting": "job",
  "first-job-passed": "job",
};

function statusOf(state: OnboardingState, key: StepKey): StepStatus {
  const order: StepKey[] = ["github", "machine", "boat", "job"];
  const open = openStep[state];
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
                ) : (
                  index + 1
                )}
              </span>
              <div className="setup-content">
                <h2 className="setup-title">{step.title}</h2>
                {open ? (
                  <StepBody state={state} step={step.key} />
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

function StepBody({ state, step }: { state: OnboardingState; step: StepKey }) {
  switch (step) {
    case "machine":
      return <MachineStep state={state} />;
    case "boat":
      return <BoatStep invalid={state === "boat-invalid"} />;
    case "job":
      return <FirstJobStep passed={state === "first-job-passed"} />;
    case "github":
      return null;
  }
}
