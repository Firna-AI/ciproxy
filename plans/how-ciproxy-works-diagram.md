# How ciproxy works diagram

Status: Active. On branch `calummoore/diagram-request-3jpv`, no pull request
yet. The pull request that completes the work deletes this plan, as the
`plans` skill says. Cal approved this plan on 23 November 2026.

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
- The picture follows the job. On a desktop, the first row holds three boxes
  from left to right, and the second row holds the Boat box and step 4. On a
  phone, the boxes stack in one column. Each box is the step of the list "A
  job on your machines" with its number, and names the part that acts:
  1. GitHub queues a job of a ciproxy label.
  2. ciproxy keeps a warm runner ready, or starts a runner where a machine
     has room.
  3. The runner runs the job in a fresh VM that no other job used.
  4. On your machine, the agent ends the VM when the job ends.
- Review finding 2 of 23 November 2026 changed the boxes to this order. The
  first boxes put the start of a VM at step 3, but a warm runner's VM is
  ready before the job arrives, and list step 3 runs the job.
- Step 4 sits under step 3, with a down arrow. A full-width row under the
  Boat box read as a step after Boat, but the VM ends only on your machines.
  This replaces the full-width row of the first sketch. The mockup found it.
- A branch from ciproxy goes to your Boat account. Its label says "Or, when
  your machines cannot take the job in time and Boat can". This covers a
  label that allows only Boat, as review finding 1 of 23 November 2026
  found. It stays true for macOS jobs, which never go to Boat. The Boat box
  has no number. On a phone, the Boat way comes after step 4, below a
  hairline, because Boat is not a step between 3 and 4.
- Each box has a small icon: GitHub, the ciproxy mark, a box for the
  runner, a server for your machine, and a cloud for Boat. The icons and
  arrows use the faint grey token.
- The figure takes its layout from its own width, not from the window: a row
  from 560 px, a column below that. The docs column can be narrow beside the
  sidebar and "On this page".
- The numbers 1 to 4 match the numbered list under "A job on your machines".
  So the list explains the picture.
- The arrows follow the job. They do not show which side opens a connection.
- The picture shows no setting values, such as the overflow wait. The text
  keeps them.
- The picture leaves out the job image, because the open follow-up item
  `plans/follow-ups/boat-image-services.md` questions the claim of one image.
- The text in the boxes is real text in reading order, inside a figure with
  a caption. The arrows are hidden from screen readers. They turn through a
  class that the icon component takes as a prop.
- A browser test in `ts/site/_tests_/browser/docs.test.ts` compares the
  place of each box with a literal table, at a 720 px window and at an
  840 px window beside the sidebar.
- The figure carries `data-pagefind-ignore`, so that search shows the text
  of the page and not a copy of it.
- The mockup showed that one picture is not too dense, so it stays one
  picture.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

Add the page How ciproxy works to the docs mockups, with the picture, in a
desktop and a mobile variant.

- [x] Add a figure component for the picture under
      `docs/mockups/src/screens/docs/`, from the tokens.
- [x] Add the screen `how-ciproxy-works.tsx` in the docs shell, with the
      intro, the picture, and the text of the page.
- [x] Add the entry
      `docs/mockups/src/entries/docs/how-ciproxy-works/index.mockup.tsx`
      with its rationale and `relatedDocs`. It has its own folder, because
      the Docs page of the catalogue already shows its limit of five screens.
      Add it to `_folder.json` and to `paths.ts`.
- [x] Add the Mac sentence of Decision 2 to "Warm runners" of the screen.
- [x] Link How ciproxy works in `docs-nav.ts`.
- [x] Update the mockups README for the new screen.
- [x] Run `npm --prefix ts run mockups:check`. Smoke-test the desktop and the
      mobile variant, and save screenshots under `.context/`.
- [x] Update the design and the TODOs of this plan for each problem that the
      mockups show.

The screenshots are in `.context/how-ciproxy-works-diagram/screenshots/`.

## Milestone 2: Documentation

Depends on: none

No protocol doc changes, because the behaviour of the product stays the
same. The docs page text gets the Mac rule.

- [x] Add the Mac sentence to "Warm runners" on the How ciproxy works page:
      on a Mac, warm runners serve the label `ciproxy-macos`.

## Milestone 3: The picture on the site

Tags: ui

Depends on: 1, 2

Build the picture of the mockup in the site, and show it on the page.

- [x] Add the data of the picture in `ts/site/src/data/`.
- [x] Add the component and its CSS under `ts/site/src/components/docs/`.
      Follow `docs/mockups/src/screens/docs/job-path-figure.tsx` and its CSS.
      Take every colour, font, and radius from the tokens. Use the icons of
      `src/components/icon/icons.ts`.
- [x] Make the ciproxy mark of `src/components/Brand.astro` a component of
      its own, so that the picture and the brand use one mark.
- [x] Rename the page to `how-ciproxy-works.mdx`. Show the picture with one
      `jsx()` call.
- [x] Update the two paths in `plans/follow-ups/boat-image-services.md` that
      name the old file.
- [x] Update the site README for the component and the page.
- [x] Run the site typecheck, lint, format check, and build.
- [x] Check the page at 1440, 720, and 390 px wide. Save the screenshots
      under `.context/`.

## Milestone 4: Verification, commit, review, and pull request

Depends on: every other milestone

- [x] Run the complete gate named in `AGENTS.md`.
- [x] Commit with Conventional Commits and push the branch.
- [x] Review the complete local diff against `origin/main` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the fix rule: fix the `fix now` findings, record
      follow-up items under `plans/follow-ups/` for the rest, re-review
      once, and report.
- [ ] Delete this plan when the `plans` skill says, under "Deleting the
      plan".
- [ ] Open the pull request. Its description gives the decisions and the
      design calls of this plan with their dates, a permalink to this plan,
      and the follow-up items that the change adds.
