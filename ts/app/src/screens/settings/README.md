# Settings pages

Settings of the [dashboard](../../../README.md) holds the Labels,
Organizations, and Members tabs. Its Overflow tab opens Settings under
Overflow, a page with a route of its own. The
[dashboard guide](../../../README.md#pages) gives their addresses. Its
[Settings summary](../../../README.md#settings) gives the banner of the
Organizations tab.

## The tabs

- Settings is for Admins, and another role goes to Jobs. Each tab, also
  Settings under Overflow, reads the settings of the workspace, so Members
  shows the count of open requests for access on each tab. While the first
  read loads or after it failed, Members shows no count, never 0. A later
  read that fails keeps the last count.
- Labels lists each label with its platform and size, where its jobs run
  now, and the jobs of the last 30 days, as the API gives them. A count that
  the API does not know says so.
- The list shows the Linux labels first, then the macOS labels, each from
  the smallest size to the largest, as
  [The label list](../../../../../docs/protocol/labels.md#the-label-list)
  says. The dashboard sorts the labels itself.
- A label has a platform, Linux or macOS, which Add label asks for and Edit
  shows read-only, as it shows the name. A macOS label runs on the Macs
  only. Its dialog has no switches and no "When nothing fits". It sends
  an order of own machines and a fallback of `wait`. Its fit line names
  Macs. The API gives no fixed maximum free disk size yet.
- Without a Boat account, the jobs of a label run on the machines only, and
  the dialog keeps the fit of the machines and says that Boat is not
  connected. It never says that a Boat size does not fit before the Boat
  catalog says so. When the API knows the fit of the machines but not the
  Boat fit, the list and the dialog give the fit of the machines and say
  that the Boat fit is not known yet.
- While the person types a size or changes a switch in Add label or Edit,
  the dialog asks `POST /workspaces/{id}/labels/preview` again. It shows the
  VM disk of one job and how many jobs each machine runs at a time. A size
  that no machine fits shows its full request: vCPU, memory, and free disk.
  The dialog names no machine offer and no Boat size, also not in "Run on
  the largest Boat size".
- A label keeps its name. Each label but `ciproxy` has Delete label. When
  GitHub refuses a new name, the dialog shows why.
- Organizations shows the workspace name with Rename, Add an organization,
  and each organization of the workspace. A linked organization has its
  repository access, Take out, and the action that clears a hold: Choose
  Team or Turn on. The last linked organization has no Take out.
- Add an organization is an action of every Admin on every tier. When the
  settings say that a new organization would need Team, the caption under
  the list says that it stays inactive until Team. With one linked
  organization, a notice also offers Team with Compare plans. Compare plans
  and Choose Team on a row open Plans.
- The repository access of an organization is all its private repositories,
  or the selected ones. A public repository cannot be selected.
- When an owner removes the App from one organization on GitHub, its row
  shows "App removed" until ciproxy deletes its data. The row gives the last
  day for a new install that keeps the job history, and Install again. It
  has no repository access, and the rule of the last linked organization
  does not count it.
- Members lists the requests for access and the members. The tab gives
  access, declines a request, changes a role, and removes a member. The row
  of the signed-in person has no Remove. Before a person gets the Admin role,
  a dialog shows the trust notice of the API. The dialog also says that an
  Admin accepts new versions of the ciproxy Terms for the customer that holds
  the workspace, with a link to the Terms.
- An organization row shows the day it joined and the counts of private
  repositories of its latest complete repository snapshot. An inactive row,
  its banner, and Turn on name the person who added the organization.
- Members shows the name and the login of each person, the day of each
  role, and for each request its day and the organizations that the person
  was a member of then. A day, a name, or a count that the API does not
  know stays out, or the row says that it is not known.

## Settings under Overflow

- Setup and Settings under Overflow read the Boat settings of a workspace
  through one store. A change that the API accepted is never sent again.
  When the read after it fails, the page offers only Try again, which reads.
- The Boat connection form has two phases, in Setup and in Settings under
  Overflow. Check key sends only the key to
  `POST /api/workspaces/<id>/overflow/providers/boat/key-check`. The API
  uses the key once to return the list of wallets. The `selected_wallet` of the
  answer starts picked when the list holds it. Connect Boat sends the key,
  the limit, and the chosen wallet. A key edit drops the wallets, and the
  answer of an older check never applies.
- The form drops the key once the API accepts the connection, before the
  read after it. The pages show the account name and never the key.

## Key Code

- The [Settings screen](settings-screen.tsx) opens the tab that
  [settings-tabs.ts](../../view/settings-tabs.ts) reads from the query. The
  [settings store](../../state/settings-store.ts) holds the settings of a
  workspace. A change that succeeds reads them and the session again.
  [settings-endpoints.ts](../../api/settings-endpoints.ts) holds the routes
  of Settings on the one HTTP client.
- [src/screens/overflow](../overflow) is Settings under Overflow, below the
  [Settings tabs](settings-tabs.tsx). [src/screens/boat](../boat) holds the
  one Boat connection form of Setup and of this page, which masks the key.
  Its `use-connect-form.ts` runs the two phases and builds the connection
  request, and `wallet-choice.tsx` lists the wallets. The folder also holds
  the spend figures, which show the confirmed and the unconfirmed costs
  apart.
- [use-saved-draft.ts](../boat/use-saved-draft.ts) holds the value of both
  limit fields: in the connection form, and in the limit panel of Settings
  under Overflow. Its rules are in
  [saved-draft.ts](../../state/saved-draft.ts). A field shows the latest
  saved limit until the person types. The typed limit stays until an
  accepted save of its own form.
- [overflow.ts](../../state/overflow.ts) holds the Boat settings of a
  workspace, and runs each read, key check, and change one at a time. It
  never keeps the key. [key-check.ts](../../state/key-check.ts) holds the
  wallets of the latest key check of a form, so an older answer never
  applies. [overflow-endpoints.ts](../../api/overflow-endpoints.ts) builds
  the key check. It sends the generated connection request as it is, with a
  required wallet. [boat-skip.ts](../../state/boat-skip.ts) keeps Skip for
  now in the tab.
- [boat.ts](../../view/boat.ts) turns the Boat settings into what the pages
  show. A reached limit comes only from the API flag. The notice of a
  reached limit names the budget month and the day Boat resumes, both from
  the next-month date of the API, in UTC. [money.ts](../../view/money.ts)
  formats the decimal-string cents of the API, and reads a typed dollar
  limit with integer arithmetic.

## Related docs

- The [labels](../../../../../docs/protocol/labels.md#custom-labels), the
  [roles](../../../../../docs/protocol/security.md#roles), the repository
  [access](../../../../../docs/protocol/security.md#repository-access), and
  the [organizations](../../../../../docs/protocol/workspaces.md) of a
  workspace.
- [Overflow](../../../../../docs/protocol/overflow.md), with the Boat account
  and the spend limit, and the overflow wait of
  [job routing](../../../../../docs/protocol/job-routing.md#the-overflow-wait).
- The [dashboard scripts](../../../scripts/README.md#browser-checks) drive
  these pages in a browser.
