// How it works, in three steps: install the GitHub App, add machines with one
// command, and change runs-on. Each step shows what the person sees there.

import type { ReactNode } from "react";
import { ArrowRight, Check } from "lucide-react";

import { TextLink } from "../../components/actions.tsx";
import { CodeBlock } from "../../components/figures.tsx";
import { installCommand } from "../../components/install-command.tsx";
import { paths } from "../../components/paths.ts";
import { appPermissions } from "../../fixtures/github-app.ts";
import { runsOnDiff, workflowFile } from "../../fixtures/workflow.ts";
import { Cells, Section, SectionHead } from "./section.tsx";

/** The install command on three lines, so it fits a narrow column. */
function commandLines(): string[] {
  const [fetch = "", run = ""] = installCommand.split(" | ");
  const [shell = "", token = ""] = run.split(" --token ");
  return [`${fetch} \\`, `  | ${shell} \\`, `    --token ${token}`];
}

function Permissions() {
  return (
    <div className="hiw-permissions">
      <p className="caption">ciproxy asks GitHub for</p>
      <ul className="hiw-permission-list">
        {appPermissions.map((permission) => (
          <li key={permission.name}>
            <Check size={14} className="hiw-check" />
            <span>
              <strong>{permission.name}</strong> · {permission.access}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}

const steps: readonly { title: string; text: string; visual: ReactNode }[] = [
  {
    title: "Install the GitHub App",
    text: "Pick your organization and its repositories on GitHub. ciproxy asks to manage runners and to read your workflow runs.",
    visual: <Permissions />,
  },
  {
    title: "Add your machines",
    text: "Run one command on each Linux machine. The agent connects out to ciproxy and keeps warm runners ready.",
    visual: <CodeBlock label="On each machine" lines={commandLines()} />,
  },
  {
    title: "Change runs-on",
    text: "Point a workflow at ciproxy. Its jobs start on your machines, and on Boat when they are full.",
    visual: <CodeBlock label={workflowFile} lines={runsOnDiff} />,
  },
];

export function HowItWorks() {
  return (
    <Section flush>
      <SectionHead
        eyebrow="How it works"
        title={["Set up once.", "Change one line."]}
        lead="ciproxy registers every runner for you. Your workflows keep their YAML, apart from one line."
      />
      <Cells columns={3}>
        {steps.map((step, index) => (
          <div key={step.title} className="mk-cell hiw-step">
            <span className="hiw-number">
              {String(index + 1).padStart(2, "0")}
            </span>
            <h3 className="hiw-title">{step.title}</h3>
            <p className="hiw-text">{step.text}</p>
            <div className="hiw-visual">{step.visual}</div>
          </div>
        ))}
      </Cells>
      <div className="mk-foot hiw-foot">
        <TextLink to={paths.quickstart}>Follow the quickstart</TextLink>
        <ArrowRight size={14} className="icon-muted" />
      </div>
    </Section>
  );
}
