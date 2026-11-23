# Stage new dashboard pages before the API rollout

Status: Active. On branch `conductor/ciproxy-deploy-order-pages-before-api`,
no pull request yet. The pull request that completes the work deletes this
plan, as the `plans` skill says.

A workload deploy in `deploy/scripts/deploy.py` rolls out the API, the Helm
release of the chart `deploy/helm/ciproxy`, before it publishes the
dashboard export. In that window, the new API can send a person to a new
dashboard page, such as `/continue`, that the dashboard bucket does not
hold yet. The follow-up item `plans/follow-ups/deploy-order-pages-and-api.md`
records the problem. This change applies its option B and deletes the item.

The change does not cover a tab that reloads for the new build in the
window. That tab still gets the old export of an existing page. The item
`plans/follow-ups/build-gate-old-export-after-deploy.md` holds that case,
and option B does not fix it.

## Decision

- **Apply option B of the follow-up item.** Agent call by the orchestrator
  of this workspace, 23 November 2026, under the user's delegation of 22
  November 2026. The item records B as its fix-after-merge action. B stages
  the new dashboard routes and their assets before the API rollout. It
  keeps the usual publication order for the existing pages. The item left
  these alternatives:
  - A: publish the new export before the API, for one release.
  - C: release the API and the dashboard as a versioned pair.
  - D: keep the current order until the launch.
  - E: publish both exports before the Helm upgrades in every release.
- Reason: B removes the missing page for each later route. It does not
  give the existing pages new code before the new API serves.

## Design

Each workload deploy runs these steps in this order:

1. Stage the dashboard export with `publish_static.py --new-only`. The
   stage step creates each object that the bucket lacks. It sends all
   assets before any HTML, and it replaces no object.
2. Run the Helm upgrades of the release server and of the API, as now.
3. Publish the site export and the dashboard export in full, as now. The
   full publication sends all assets before any HTML, writes each object,
   and deletes no object.

Design calls, each an agent call of 23 November 2026:

- **A create-only upload finds the new objects.** In `--new-only` mode,
  each upload carries the Cloud Storage precondition `ifGenerationMatch=0`.
  Cloud Storage answers HTTP 412 when the bucket holds a live object of
  that name. The publisher then skips the object. Any other error stops
  the publisher. The alternative was a list of the bucket before the
  uploads. Reason: the precondition is atomic for each object, needs no
  paging, and its cost does not grow with the old objects in the bucket.
  Cost: the stage step sends the export once more. For the current
  dashboard export, that is 83 requests and 4.6 MB.
- **The stage step covers every object of the export.** New hashed chunks
  arrive before the pages that load them. An existing page and the files
  that it loads stay unchanged until the full publication. A changed file
  under an old name, such as `_expo/.routes.json`, also waits for the full
  publication. The dashboard bundle does not read that file.
- **The stage step runs before both Helm upgrades.** A missing or broken
  export then stops the deploy before any rollout. The release server does
  not use the dashboard.
- **Only the dashboard export is staged.** The API sends people only to
  dashboard paths, such as `/continue`, `/sign-in`, and `/install`. The
  site keeps its order.
- **Every workload deploy stages, also with the runtime off.** One order
  then holds for every deploy. Without a running API, the step only adds
  the new pages early.
- **A name that an older export left counts as present.** The publisher
  deletes nothing, and the bucket has no record of the export that serves
  now. Such a page keeps its old content until the full publication. It
  still serves, so nobody meets a missing page.
- **The dry run does not read the bucket.** With `--new-only`, it lists
  every candidate object in the format of today.
  `deploy/tests/test_static_routes.py` reads that format.

Docs that state the publication order:

- [Static delivery](../deploy/static-delivery.md#lifetimes-and-publishing)
  is the home of the publication facts.
- The [deploy README](../deploy/README.md) and
  [Operations](../deploy/operations.md) summarize them.
- [Promotion](../deploy/promotion.md) says that the publisher sends assets
  before HTML. That sentence stays true, so it needs no change.

## Milestone 1: Documentation

Depends on: none

State the stage step and the deploy order in the deploy docs.

- [x] `deploy/static-delivery.md`, "Lifetimes and publishing": state the
      three steps of the design, and why the stage step comes first.
- [x] `deploy/README.md`: name the stage step in the row of
      `scripts/deploy.py`. Say that static delivery gives the order.
- [x] `deploy/operations.md`, "Static files and protection": add the stage
      step to the publisher bullet, with a link to static delivery.

## Milestone 2: Stage step in the deploy scripts

Depends on: none

- [x] `deploy/scripts/publish_static.py`: add `--new-only`, as the design
      says.
- [x] `deploy/scripts/deploy.py`: run the stage step before the Helm
      upgrades.
- [x] Tests, each seen to fail on an assertion before the change:
  - [x] `deploy/tests/test_publish_static.py`: `--new-only` creates only
        the objects that the fake bucket lacks. It sends each upload with
        the precondition, and every asset before any HTML.
  - [x] The same file: an error other than HTTP 412 stops the stage step.
  - [x] A deploy test with the fake tools of
        `deploy/tests/fake_promotion_tools.py`: the stage step runs before
        both Helm upgrades, and the full publications run after them.
- [x] Run `python3 -m unittest discover -s deploy/tests -p 'test_*.py'`.

## Milestone 3: Verification, commit, review, and pull request

Depends on: 1, 2

- [x] Delete `plans/follow-ups/deploy-order-pages-and-api.md`.
- [x] Run the deploy tests, the early check, and the complete gate named
      in `AGENTS.md`.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Open a ready pull request against `main`, as the task asks.
- [ ] Review the complete local diff against `origin/main` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the fix rule: fix the `fix now` findings, re-review once,
      and report. The task scope allows no new follow-up item, so the final
      report lists each other finding.
- [ ] Delete this plan in the last commit of the work.
- [ ] Update the pull request description. Give the decision and the
      design calls with their dates, and a permalink to this plan.
