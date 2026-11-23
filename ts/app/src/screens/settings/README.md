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
- Where jobs run names the places of the label order that can run the label
  now, in order, such as "Your machines, then Boat", or the largest size of
  a provider. A label that no place can run now waits, with the reason of
  the first place of its order that fits its size, such as "Waits: Boat
  reached its spend limit". The routing of the API gives the places and the
  reason, so the list and the Jobs notices never disagree.
- The list shows the Linux labels first, then the macOS labels, each from
  the smallest size to the largest, as
  [The label list](../../../../../docs/protocol/labels.md#the-label-list)
  says. The dashboard sorts the labels itself.
- A label has a platform, Linux or macOS, which Add label asks for and Edit
  shows read-only, as it shows the name. A macOS label runs on the Macs
  only. Its order holds own machines only, with no handle, no Remove, and no
  Add, and it has no "When nothing fits". It sends an order of own machines
  and a fallback of `wait`. Its fit line names Macs. The API gives no fixed
  maximum free disk size yet.
- The dialog of a Linux label shows its order: own machines and each
  provider, in the order in which the label tries them. The grab handle of
  the Firna sortable list moves a place, by pointer or by keyboard. Remove
  takes a place out, and Add a place offers own machines and each offered
  provider that the order does not hold. With no offered provider, the
  dialog links to Settings under Overflow. An empty order holds the jobs,
  and the dialog says so.
- Each row of the order says what its place does with a job of the size:
  how long a job waits for own machines, the provider size that runs it, or
  why jobs skip the place. A provider row says first when overflow is off,
  or when the provider is not connected or not available, and only then
  whether one of its sizes fits. The overflow wait shows on own machines
  only, and only when a place follows them. A move keeps the words of each
  row; a new size or a new place waits for its fit.
- A new label starts with own machines, then each offered provider. Until
  the person changes that order, Add label leaves the order to the API.
- While the person types a size or changes the order in Add label or Edit,
  the dialog asks `POST /workspaces/{id}/labels/preview` again. It shows the
  VM disk of one job and how many jobs each machine runs at a time. A size
  that no machine fits shows its full request: vCPU, memory, and free disk,
  with the place that runs it or the reason of the place that would. The
  dialog names no machine offer and no provider size in its fit line, and
  "Run on the largest size" names no provider. The dialog reads the
  overflow settings for the overflow wait and the offered providers.
- The shared dialog keeps its title and its footer in view, and a long body
  scrolls between them, also in the sheet of a phone.
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

- Setup, Settings under Overflow, and the label dialog read the overflow
  settings of a workspace through one store. A change that the API accepted
  is never sent again. When the read after it fails, the page offers only
  Try again, which reads.
- With no connected provider, the page asks for a provider and its key and
  limit, as Setup does. The provider choice lists each provider of ciproxy,
  and a provider that the deployment cannot start shows as unavailable
  right now. When no provider is available, the page says so.
- Otherwise the page shows a notice for each provider that needs one, then
  the switch and the wait, which hold for every provider, then one panel for
  each provider that is connected or available. A notice names its provider
  and says when jobs start there again.
- The panel of a connected provider shows its account name, never the key,
  with Change key and Disconnect, and its own monthly spend limit with the
  spend of the month. While the deployment has no settings for the provider,
  the panel keeps the account, the limit, and Disconnect, and hides Change
  key. After the provider rejected the key, the panel asks for a new key,
  with Disconnect. An available provider with no account shows Connect, and
  its first connection puts it last in the order of each Linux label.
- The connection form of a provider asks for its key and its monthly limit.
  The descriptor on the wire says whether a key bills one of several
  accounts. For Boat, Check key sends only the key to
  `POST /api/workspaces/<id>/overflow/providers/boat/key-check`. The API uses
  the key once to return the list of wallets. The `selected_wallet` of the
  answer starts picked when the list holds it. Connect Boat sends the key,
  the limit, and the chosen wallet. A key edit drops the wallets, and the
  answer of an older check never applies. A provider without such a choice
  connects in one step.
- The form drops the key once the API accepts the connection, before the
  read after it. The pages show the account name and never the key.

## Key Code

- The [Settings screen](settings-screen.tsx) opens the tab that
  [settings-tabs.ts](../../view/settings-tabs.ts) reads from the query. The
  [settings store](../../state/settings-store.ts) holds the settings of a
  workspace. A change that succeeds reads them and the session again.
  [settings-endpoints.ts](../../api/settings-endpoints.ts) holds the routes
  of Settings on the one HTTP client.
- [label-order.tsx](label-order.tsx) is the order control of the label
  dialog. [settings-label-order.ts](../../view/settings-label-order.ts)
  words its rows, and
  [settings-label-places.ts](../../view/settings-label-places.ts) names the
  places and gives what Add a place offers.
  [settings-labels.ts](../../view/settings-labels.ts) words the routing of
  each label, and [settings-label-fit.ts](../../view/settings-label-fit.ts)
  the fit line of the dialog.
- [src/screens/overflow](../overflow) is Settings under Overflow, below the
  [Settings tabs](settings-tabs.tsx). Its `provider-panel.tsx` is the panel
  of one provider, `provider-choice.tsx` the provider choice of a first
  connection in Setup and on this page, and `provider-form.tsx` the one
  connection form, which masks the key. `use-connect-form.ts` runs the form
  and builds the connection request. [src/screens/boat](../boat) holds the
  wallet choice of the Boat branch. `spend-figures.tsx` shows the confirmed
  and the unconfirmed costs apart.
- [use-saved-draft.ts](../overflow/use-saved-draft.ts) holds the value of
  both limit fields of a provider: in the connection form, and in its panel.
  Its rules are in [saved-draft.ts](../../state/saved-draft.ts). A field
  shows the latest saved limit until the person types. The typed limit
  stays until an accepted save of its own form.
- [overflow.ts](../../state/overflow.ts) holds the overflow settings of a
  workspace, and runs each read, key check, and change of a provider one at
  a time. It never keeps the key. [key-check.ts](../../state/key-check.ts)
  holds the wallets of the latest Boat key check of a form, so an older
  answer never applies.
  [overflow-endpoints.ts](../../api/overflow-endpoints.ts) builds the
  routes of each provider. [provider-skip.ts](../../state/provider-skip.ts)
  keeps Skip for now in the tab.
- [overflow.ts](../../view/overflow.ts) turns the overflow settings into
  what the pages show. A reached limit comes only from the API flag. The
  notice of a reached limit names the budget month and the day jobs start
  there again, both from the next-month date of the API, in UTC.
  [providers.ts](../../view/providers.ts) holds the copy of each provider and
  the words of each reason why a provider cannot take a job now.
  [money.ts](../../view/money.ts) formats the decimal-string cents of the
  API, and reads a typed dollar limit with integer arithmetic.

## Related docs

- The [labels](../../../../../docs/protocol/labels.md#custom-labels), the
  [roles](../../../../../docs/protocol/security.md#roles), the repository
  [access](../../../../../docs/protocol/security.md#repository-access), and
  the [organizations](../../../../../docs/protocol/workspaces.md) of a
  workspace.
- [Overflow](../../../../../docs/protocol/overflow.md), with the provider
  accounts, the [spend limit](../../../../../docs/protocol/overflow-spend.md)
  of each provider, and the overflow wait of
  [job routing](../../../../../docs/protocol/job-routing.md#the-overflow-wait).
- The [label order](../../../../../docs/protocol/labels-order.md) and
  [a size that nothing can run](../../../../../docs/protocol/labels.md#a-size-that-nothing-can-run).
- The [dashboard scripts](../../../scripts/README.md#browser-checks) drive
  these pages in a browser.
