// Repository access of one organization of the workspace, mokly-ai, which
// the Organizations tab opens: which private repositories can use ciproxy
// runners. The GitHub App installation decides which repositories ciproxy
// can see.

import { Check, ExternalLink } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { RadioCard, RadioCardGroup } from "@firna/ui/radio";

import { Action, TextLink, noop } from "../../components/actions.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { org } from "../../fixtures/org.ts";
import { repositories } from "../../fixtures/settings.ts";
import { SettingsFrame } from "./settings-frame.tsx";

export const repositoriesStates = ["all", "selected"] as const;
export type RepositoriesState = (typeof repositoriesStates)[number];

export function RepositoriesScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: RepositoriesState;
}) {
  const selected = repositories.filter((repo) => repo.selected).length;
  return (
    <SettingsFrame viewport={viewport} active={paths.organizations}>
      <p className="small">
        <TextLink to={paths.organizations}>Organizations</TextLink>
        <span className="muted"> / </span>
        <strong>{org.login}</strong>
      </p>
      <Panel title="Repositories that use ciproxy runners">
        <div className="stack settings-form">
          <RadioCardGroup label="Repository access">
            <RadioCard
              title="All private repositories"
              body={`${org.repositories} repositories, and new ones as you add them`}
              checked={state === "all"}
              onPress={noop}
            />
            <RadioCard
              title="Selected repositories"
              body={
                state === "selected"
                  ? `${selected} of ${org.repositories} repositories`
                  : "Only the repositories you pick"
              }
              checked={state === "selected"}
              onPress={noop}
            />
          </RadioCardGroup>
          {state === "selected" ? (
            <ul className="repo-list">
              {repositories.map((repo) => (
                <li key={repo.name} className="repo-row">
                  <span
                    className={`check${repo.selected ? " is-checked" : ""}`}
                    aria-hidden="true"
                  >
                    {repo.selected ? <Check size={12} strokeWidth={3} /> : null}
                  </span>
                  <span className="small">
                    {org.login}/{repo.name}
                  </span>
                </li>
              ))}
            </ul>
          ) : null}
          <p className="caption">
            ciproxy runs jobs from private repositories. The{" "}
            {org.publicRepositories} public repositories of {org.login} keep
            their current runners.
          </p>
          <div className="row">
            <Action tone="primary">Save</Action>
          </div>
        </div>
      </Panel>
      <Panel title="GitHub App">
        <div className="spread spread-stack">
          <p className="small">
            GitHub decides which repositories ciproxy can see. Change them on
            GitHub.
          </p>
          <Action icon={ExternalLink} size="sm">
            Manage on GitHub
          </Action>
        </div>
      </Panel>
    </SettingsFrame>
  );
}
