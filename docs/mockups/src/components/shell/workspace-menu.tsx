// The menu of the workspace switch in the top bar: the workspaces of the
// signed-in person, each with its tier and its organizations, and the install
// on another organization, which makes a new workspace.

import { Check, Plus } from "lucide-react";
import { Avatar } from "@firna/ui/avatar";

import { dotJoin } from "../../fixtures/format.ts";
import {
  organizationsText,
  workspace,
  workspaceChoices,
} from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import "./workspace-menu.css";

export function WorkspaceMenu() {
  return (
    <div className="workspace-menu" role="menu" aria-label="Workspaces">
      <p className="caption workspace-menu-title">Workspaces</p>
      <ul className="workspace-menu-list">
        {workspaceChoices.map((choice) => (
          <li
            key={choice.name}
            className={`workspace-menu-item${choice.name === workspace.name ? " is-current" : ""}`}
          >
            <Avatar
              label={choice.initial}
              accessibilityLabel={choice.name}
              size={24}
              shape="square"
            />
            <span className="workspace-menu-text">
              <span className="strong">{choice.name}</span>
              <span className="caption">
                {dotJoin(
                  tiers[choice.tier].name,
                  organizationsText(choice.organizations),
                )}
              </span>
            </span>
            {choice.name === workspace.name ? (
              <Check size={16} className="workspace-menu-check" />
            ) : null}
          </li>
        ))}
      </ul>
      <div className="workspace-menu-foot">
        <span className="workspace-menu-action">
          <Plus size={16} strokeWidth={1.75} />
          <span>Install on another organization</span>
        </span>
      </div>
    </div>
  );
}
