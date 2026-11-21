// The page that every page of the workspace shows after an owner removes the
// ciproxy GitHub App on GitHub from its only organization. ciproxy then has
// no GitHub access to the organization, so it shows no data of it: only that
// the App was removed, and how to install it again. A new install within 30
// days finds the labels, the settings, and the job history; each machine
// needs the install command again, and Boat a new connection.

import { Github } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { appRemoval } from "../../fixtures/access.ts";
import { unbroken } from "../../fixtures/format.ts";
import { org, user, workspace } from "../../fixtures/org.ts";
import { AuthShell } from "./auth-shell.tsx";

export function AppRemovedScreen({ viewport }: { viewport: Viewport }) {
  return (
    <AuthShell
      viewport={viewport}
      footer={`Signed in as ${user.login} on GitHub.`}
    >
      <div className="stack-tight">
        <h1 className="auth-title">
          The GitHub App was removed from {org.login}
        </h1>
        <p className="auth-lead">
          {appRemoval.by} removed it on {appRemoval.on}, so jobs of{" "}
          <span className="nowrap">{org.login}</span> no longer start on
          ciproxy. Team ends on {appRemoval.planEnds}, with no more charges.
        </p>
      </div>
      <Notice
        tone="info"
        title={`Install it again before ${appRemoval.deletedOn}`}
      >
        ciproxy keeps the labels, settings, and job history of{" "}
        <span className="nowrap">{workspace.name}</span> for{" "}
        {unbroken(`${appRemoval.keepDays} days`)}. Each machine then needs the
        install command again, and Boat a new connection.
      </Notice>
      <div className="auth-actions">
        <Action tone="primary" size="lg" icon={Github} block>
          Install on GitHub
        </Action>
      </div>
    </AuthShell>
  );
}
