# Dashboard dialogs misplace or lose their first focus

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from pull request #125, the branch
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, review round
  2, the wider fix of finding 8, with a problem outside the scope that the
  review and the check of finding 8 found.
- **Location:** `ts/app/src/components/feature-parts.tsx:202`,
  `FeatureDialog`, and `:227`, its placement.
  `ts/app/src/components/viewport.ts:31`, `useViewport`.
  `ts/app/src/screens/billing/move-to-free-dialog.tsx:41` and `:60`, the
  placement of `MoveToFreeDialog`.
  `ts/app/src/screens/overflow/disconnect-dialog.tsx:32`, `:54`, and `:60`,
  the placement, the key, and the focus target of `DisconnectDialog`.
  `ts/app/src/screens/settings/grant-admin-dialog.tsx`,
  `ts/app/src/screens/settings/organization-dialogs.tsx:57`,
  `ts/app/src/screens/machines/machine-cache.tsx:129`, and
  `ts/app/src/screens/settings/label-dialog.tsx:153`, the other callers of
  `FeatureDialog`. `ts/app/scripts/m20-browser-settings.ts`, the browser
  check of the grant dialog.
- **Reason:** The shared frame reaches five files outside the diff of pull
  request #125, and the focus loss on a phone is older than that branch.

- **Terms:**
  - **`WebModalFrame`:** the dialog frame of `@firna/ui/modal`, version
    4.0.0. Its `placement` prop draws a centred dialog or a bottom sheet.
  - **First focus:** the control that has the keyboard focus when a dialog
    opens.
  - **Confirmation:** a dialog that asks a person to confirm one change.
    Its footer has a safe action, such as Cancel, and the action itself.
  - **Phone width:** a window narrower than 640 px. The dashboard then
    shows each dialog as a bottom sheet.
- **Background:**
  - Without `initialFocusRef`, `WebModalFrame` gives the first focus to the
    first control that the caller rendered, and skips its own close button.
    That control can be a form field, a link in the body, or the first
    button of the footer.
  - The frame moves the focus only when its `visible` prop changes. It
    builds the sheet and the centred dialog as different trees. So when
    the placement of an open dialog changes, the focused control unmounts,
    and the focus falls out of the dialog. The next Tab brings it back.
  - `FeatureDialog` in `ts/app/src/components/feature-parts.tsx` wraps the
    frame. It takes its placement from `useViewport` at line 227.
    `useViewport` in `ts/app/src/components/viewport.ts` gives the wide
    class until its component has mounted, so that the static HTML and the
    first client render agree.
  - So a `FeatureDialog` that mounts when it opens draws a centred dialog
    first, and the frame focuses a control in it. At a phone width, the
    dialog then becomes a sheet, and that control unmounts. In Chrome at
    390 px, the grant dialog lost its first focus in this way, also without
    its Terms line.
  - `MoveToFreeDialog` has the same cause at line 41. It takes its
    placement from `useViewport` and renders the frame directly.
  - `DisconnectDialog` in `ts/app/src/screens/overflow/disconnect-dialog.tsx`
    avoids both problems. It opens only after a press, so it reads the
    window width at once, at lines 32 to 35. It keys the frame on the
    placement at line 54, so a change of the placement opens a new frame.
    Its `initialFocusRef` at line 60 then focuses Cancel again. The comment
    at the top of the file says why.
  - Pull request #125 added a line with a Terms link to the body of the
    grant dialog. The dialog named no focus target, so it opened with the
    first focus on the Terms link. In Chrome at 1280 px, the next Enter
    opened the public site in another tab.
  - Review round 2 of pull request #125 fixed that one dialog on 22
    November 2026. The grant dialog now passes its Cancel button as
    `initialFocusRef` to `FeatureDialog`. Its browser check in
    `ts/app/scripts/m20-browser-settings.ts` asserts that the dialog does
    not open with the focus on its Terms link.
  - That check does not assert Cancel. The browser smoke of that file runs
    each case at 1440, 720, and 390 px, and at 390 px the sheet loses the
    focus.
  - The confirmations of the dashboard:
    - Give the Admin role: `GrantAdminDialog`.
    - Take an organization out, and turn one on: `TakeOutDialog` and
      `TurnOnDialog`, through `ChangeDialog` in `organization-dialogs.tsx`.
    - Clear the cache of a machine: `ClearCacheDialog` in
      `machine-cache.tsx`.
    - Move to Free: `MoveToFreeDialog`. Its safe action says "Keep" and the
      current plan, or "Close" after an unknown result.
    - Disconnect Boat: `DisconnectDialog`.
  - Only the grant dialog and the disconnect dialog name a focus target.
  - Two dialogs hold a form: Rename the workspace, `RenameDialog` through
    `ChangeDialog`, and the label dialog, `LabelDialog` in
    `label-dialog.tsx`. Their first focus belongs on their first field,
    which the frame gives by default.
  - From the code, `ClearCacheDialog` stays mounted and changes only
    `visible`. So its placement is already right when it opens, and it
    keeps its focus at a phone width.
- **Problem:** A help link before the footer of a confirmation takes the
  first focus, unless the confirmation names a target. At a phone width, a
  dialog that mounts when it opens loses its first focus, because its
  placement changes after the frame moved the focus.
- **Impact if no action:** The next help link in a confirmation can again
  send a keyboard user out of the confirmation with one Enter. At a phone
  width, the focus of the grant, organization, label, and Move to Free
  dialogs falls out of the dialog when it opens.
- **Root question:** How does each dashboard dialog open with its first
  focus on the right control, at every width?
- **Options:**
  - **A.** Keep only the direct fix of pull request #125: the grant dialog
    names Cancel, and each other dialog keeps its own focus code. A new
    help link can still take the first focus, and the sheets still lose
    it. Effort: Small.
  - **B.** Give the first focus to the title of each confirmation, so that
    a person starts with the explanation. This changes the first focus of
    the grant dialog and the disconnect dialog. On its own, it does not
    stop a sheet from losing the focus. Effort: Medium.
  - **C.** One shared confirmation frame:
    - It renders the safe action and the confirm action. The safe action
      is Cancel, or a label that the caller gives, such as "Keep Team".
    - It gives the safe action the first focus.
    - It builds on `FeatureDialog`, which keys `WebModalFrame` on the
      placement from now on, as the disconnect dialog does. A change of the
      placement then opens a new frame, which focuses the target again.
    - Move the six confirmations to it. Rename the workspace and the label
      dialog keep `FeatureDialog`, so their first field also keeps the
      focus at a phone width.
    - Make the check of the grant dialog in
      `ts/app/scripts/m20-browser-settings.ts` assert that Cancel has the
      first focus, at every width. Add the same check where a browser
      smoke opens another confirmation.
    - Cost: six confirmations in five files change.
    - Effort: Medium.
  - **D.** Give each confirmation its own `WebModalFrame`, keyed on the
    placement, with the safe action as its focus target, as the disconnect
    dialog does. No part is shared, so each new confirmation must repeat
    the pattern. Effort: Medium.
  - **E.** Mark help links, such as the Terms link, so that `FeatureDialog`
    skips them when it chooses the first focus. The first focus then lands
    on the next control, Cancel in a confirmation. `FeatureDialog` must
    then choose the control itself, because the frame chooses it inside
    `@firna/ui` today. E does not stop a sheet from losing the focus.
    Effort: Medium.
  - **F.** Put the marker of E into `@firna/ui`, so that `WebModalFrame`
    skips marked help links in every app. The change and its release are
    outside this repository. F does not stop a sheet from losing the focus
    either. Effort: Medium.
- **Left out:** Take help links out of the tab order: keyboard users could
  not reach them.
- **Wider fix:** C. A new help link cannot take the first focus of a
  confirmation that uses the shared frame, and a change of the placement
  cannot drop it.
- **Previous examples:**
  - `ts/app/src/screens/overflow/disconnect-dialog.tsx:60` already gives
    Cancel the first focus, and its key at line 54 keeps it at a phone
    width.
  - The grant dialog opened on its Terms link until review round 2 of pull
    request #125.
- **Related:** `plans/follow-ups/dashboard-duplicate-controls.md`, the same
  code. Its option C, which its recommended option E includes, moves
  `FeatureDialog` into its own module. That one dialog frame then serves
  every dialog, also the disconnect dialog and Move to Free, which repeat
  its placement. C of this item can build the confirmation frame on that
  module. The options do not conflict, and either item can go first.
- **Recommendation:** C. One frame gives every confirmation its safe first
  focus at every width, so neither a new help link nor a phone can move it.
- **Action:** fix after merge, apply C, because the shared frame reaches
  confirmations outside the diff of pull request #125, and the focus loss
  on a phone is older than that branch.
