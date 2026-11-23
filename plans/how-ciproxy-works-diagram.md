# How ciproxy works diagram

Status: Active. On branch `calummoore/diagram-request-3jpv`, no pull request
yet. The pull request that completes the work deletes this plan, as the
`plans` skill says.

The docs page How ciproxy works at `/docs/how-ciproxy-works/` is text only.
A new reader must read all of it to see how GitHub, ciproxy, the machines of
the customer, and Boat connect. This change adds one picture of the parts of
ciproxy and the path of one job. The picture adds to the text. The text stays
the full version. The change also makes the warm runner text match the Mac
rule of the protocol. It does not change the behaviour of the product, and it
adds no picture to other pages.

Contract owners:

- [Job routing](../docs/protocol/job-routing.md) gives the rules that the
  picture must not contradict.
- The [site README](../ts/site/README.md).
- The [mockups README](../docs/mockups/README.md).

## Decisions

### 1. How to build the diagram

- **Decided:** 23 November 2026, by Cal: option B.
- **Root question:** How should the page show the parts and the path of a
  job as a picture?
- **Options:**
  - **A.** An ASCII picture in a `text` code block, as in
    `docs/architecture.md`. Wide lines break on a phone, and screen readers
    read each character.
  - **B.** An Astro component of the site that draws HTML boxes in a CSS
    grid, with Lucide arrows, from the tokens. The boxes stack in one column
    on a phone. No new dependency.
  - **C.** An Astro component with a hand-drawn inline SVG picture, and a
    second drawing for phones. SVG text does not wrap.
  - **D.** A diagram tool: Mermaid in the browser, or Graphviz in WASM at
    build time. Either one adds a dependency. Mermaid needs JavaScript, and
    Graphviz does not size its boxes for Geist.
- **Left out:** change nothing, because it fails the goal. An SVG or PNG
  image file, because it cannot read the tokens. A change to the docs lint
  rule that refuses HTML, because `jsx()` gives the same result.
- **Reason:** B meets every site rule with no new dependency, and it reads
  well on a phone and with a screen reader.

### 2. Warm runner text leaves out the Mac case

- **Decided:** 23 November 2026, by Cal: option A.
- **Problem:** The page says that warm runners serve the label `ciproxy`. On
  a Mac, warm runners serve only `ciproxy-macos`, as
  [Warm runners](../docs/protocol/job-routing.md) says.
- **Options:**
  - **A.** Add one sentence for the Mac case.
- **Reason:** It makes the page match the protocol doc.

## Design

These are design calls of the agent, made on 23 November 2026.

- One picture goes under the intro of the page, before "The parts". This is
  the first place that a new reader looks.
- The picture follows the job, from left to right on a desktop and from top
  to bottom on a phone:
  1. GitHub queues a job of a ciproxy label.
  2. The ciproxy control plane decides how many runners start, and where.
  3. On your machine, the agent starts a fresh VM with a runner.
  4. The runner takes a job from GitHub, runs it, and the VM ends.
- A branch from the control plane goes to your Boat account. Its label says
  that Boat runs the job when your machines have no room and Boat can take
  the job. This stays true for macOS jobs, which never go to Boat.
- The numbers 1 to 4 match the numbered list under "A job on your machines".
  So the list explains the picture.
- The arrows follow the job. They do not show which side opens a connection.
- The picture shows no setting values, such as the overflow wait. The text
  keeps them.
- The picture leaves out the job image, because the open follow-up item
  `plans/follow-ups/boat-image-services.md` questions the claim of one image.
- The text in the boxes is real text in reading order, inside a figure with
  a caption. The arrows are hidden from screen readers.
- The figure carries `data-pagefind-ignore`, so that search shows the text
  of the page and not a copy of it.
- If the mockup shows that one picture is too dense, the picture splits into
  two: the parts, and the path of a job.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

Add the page How ciproxy works to the docs mockups, with the picture, in a
desktop and a mobile variant.

- [ ] Add a figure component for the picture under
      `docs/mockups/src/screens/docs/`, from the tokens.
- [ ] Add the screen `how-ciproxy-works.tsx` in the docs shell, with the
      intro, the picture, and the text of the page.
- [ ] Add the entry `docs/mockups/src/entries/docs/how-ciproxy-works.mockup.tsx`
      with its rationale and `relatedDocs`. Add it to `_folder.json` and to
      `paths.ts`.
- [ ] Link How ciproxy works in `docs-nav.ts`.
- [ ] Update the mockups README for the new screen.
- [ ] Run `npm --prefix ts run mockups:check`. Smoke-test the desktop and the
      mobile variant, and save screenshots under `.context/`.
- [ ] Update the design and the TODOs of this plan for each problem that the
      mockups show.

## Milestone 2: Documentation

Depends on: 1

No protocol doc changes, because the behaviour of the product stays the
same. The docs page text gets the Mac rule.

- [ ] Add the Mac sentence to "Warm runners" on the How ciproxy works page:
      on a Mac, warm runners serve the label `ciproxy-macos`.
- [ ] Add the same sentence to the mockup screen of the page.

## Milestone 3: The picture on the site

Tags: ui

Depends on: 1, 2

Build the picture of the mockup in the site, and show it on the page.

- [ ] Add the data of the picture in `ts/site/src/data/`.
- [ ] Add the component and its CSS under `ts/site/src/components/docs/`.
      Take every colour, font, and radius from the tokens. Use the icons of
      `src/components/icon/icons.ts`.
- [ ] Rename the page to `how-ciproxy-works.mdx`. Show the picture with one
      `jsx()` call.
- [ ] Update the two paths in `plans/follow-ups/boat-image-services.md` that
      name the old file.
- [ ] Update the site README for the component and the page.
- [ ] Run the site typecheck, lint, format check, and build.
- [ ] Check the page at 1440, 720, and 390 px wide. Save the screenshots
      under `.context/`.

## Milestone 4: Verification, commit, review, and pull request

Depends on: every other milestone

- [ ] Run the complete gate named in `AGENTS.md`.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Review the complete local diff against `origin/main` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the fix rule: fix the `fix now` findings, record
      follow-up items under `plans/follow-ups/` for the rest, re-review
      once, and report.
- [ ] Delete this plan when the `plans` skill says, under "Deleting the
      plan".
- [ ] Open the pull request. Its description gives the decisions and the
      design calls of this plan with their dates, a permalink to this plan,
      and the follow-up items that the change adds.
