// The repository access of one organization: all private repositories,
// which includes the new ones that ciproxy can see, or the selected ones.
// GitHub decides which repositories ciproxy sees, and a public repository
// never gets a ciproxy runner, so only a private one can be picked.

import type { OrganizationView, RepositorySelection } from "../api/schema.ts";
import type { RepositoryView } from "../api/settings-schema.ts";

export type AccessMode = RepositorySelection["mode"];

export type AccessDraft = {
  mode: AccessMode;
  /** The picked private repositories, by ID. */
  picked: ReadonlySet<string>;
};

/**
 * The draft of the saved access. A switch from all to selected starts with
 * every private repository picked, which is what the access allows now.
 */
export function accessDraft(
  organization: OrganizationView,
  repositories: readonly RepositoryView[],
): AccessDraft {
  const { access } = organization;
  if (access.mode === "selected") {
    return { mode: "selected", picked: new Set(access.ids) };
  }
  return {
    mode: "all_private",
    picked: new Set(
      repositories.filter((repo) => repo.private).map((repo) => repo.id),
    ),
  };
}

export function toggled(draft: AccessDraft, id: string): AccessDraft {
  const picked = new Set(draft.picked);
  if (picked.has(id)) picked.delete(id);
  else picked.add(id);
  return { ...draft, picked };
}

/** The request body of a save, with the picked IDs in the order of the list. */
export function selectionOf(
  draft: AccessDraft,
  repositories: readonly RepositoryView[],
): RepositorySelection {
  if (draft.mode === "all_private") return { mode: "all_private" };
  return {
    mode: "selected",
    ids: repositories
      .filter((repo) => repo.private && draft.picked.has(repo.id))
      .map((repo) => repo.id),
  };
}

/** Whether the draft differs from the saved access. */
export function changed(
  organization: OrganizationView,
  draft: AccessDraft,
  repositories: readonly RepositoryView[],
): boolean {
  const next = selectionOf(draft, repositories);
  const saved = organization.access;
  if (next.mode !== saved.mode) return true;
  if (next.mode === "all_private" || saved.mode === "all_private") return false;
  const before = new Set(saved.ids);
  return (
    next.ids.length !== before.size || next.ids.some((id) => !before.has(id))
  );
}

export type AccessTexts = {
  allBody: string;
  selectedBody: string;
  /** The line about the public repositories, when GitHub shows some. */
  publicNote: string | null;
};

function count(value: number, one: string, many: string): string {
  return `${value} ${value === 1 ? one : many}`;
}

export function accessTexts(
  login: string,
  draft: AccessDraft,
  repositories: readonly RepositoryView[],
): AccessTexts {
  const privates = repositories.filter((repo) => repo.private);
  const publics = repositories.length - privates.length;
  const picked = privates.filter((repo) => draft.picked.has(repo.id)).length;
  return {
    allBody: `${count(privates.length, "repository", "repositories")}, and new ones as you add them`,
    selectedBody:
      draft.mode === "selected"
        ? `${picked} of ${count(privates.length, "repository", "repositories")}`
        : "Only the repositories you pick",
    publicNote:
      publics === 0
        ? null
        : publics === 1
          ? `ciproxy runs jobs from private repositories. The public repository of ${login} keeps its current runners.`
          : `ciproxy runs jobs from private repositories. The ${publics} public repositories of ${login} keep their current runners.`,
  };
}
