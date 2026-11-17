# Prelaunch images can cross the launch boundary

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 801.
- **Source:** Review.
- **Category:** Security.
- **Location:** `deploy/scripts/image_registry.py:70`,
  `deploy/scripts/image_registry.py:87`,
  `deploy/terraform/bootstrap/control-plane-images.tf:21`,
  `deploy/image-workflow-bootstrap.md:131`, and
  `deploy/control-plane-images.md:14`.
- **Reason:** Every option needs a launch step or new infrastructure, and
  the recommended B changes the shared image registry that the user chose
  on 5 November 2026.

- **Terms:**
  - **Primary branch:** `calummoore/ci-proxy-load-balancer`, the
    unprotected branch that holds the work before the launch.
  - **Control plane:** the hosted service that handles the requests of the
    customers. It holds the runtime credentials.
  - **Source tag:** a registry tag `source-<hash>`. The hash covers the
    build inputs, so the tag selects the image of those inputs.
  - **Trust move:** the launch step that moves the trust of the image
    workflow from the primary branch to protected `main`.
  - **Publisher:** the service account `ciproxy-image-publish`. It is the
    one account that can push control-plane images.
- **Background:**
  - One shared Artifact Registry repository holds the control-plane images
    of the main preview and of production:
    `us-central1-docker.pkg.dev/ciproxy/ciproxy-shared-runtime/control-plane`,
    at `deploy/scripts/image_registry.py:12`.
  - It is shared because of a user decision of 5 November 2026: the main
    preview and production both run in the GCP project `ciproxy`, with one
    shared GKE cluster.
  - On 7 November 2026, the user chose the image flow: a build job with no
    cloud credentials, a publish job that pushes by digest, deploys of that
    digest, and a reuse of the image of unchanged sources. The user ruled
    out a manual build.
  - Each accepted preview push builds or reuses the image. Production
    neither builds nor publishes. It deploys the digest that the preview
    rollout recorded, as `deploy/control-plane-images.md:3-7` says.
  - `lookup` at `deploy/scripts/image_registry.py:63-71` reads the digest
    of the source tag. When the tag exists, the image job skips the build
    and uses that digest. It checks no build provenance.
  - `publish` at `:81-96` also returns the digest of an existing source
    tag, at `:87-89`, in place of its own build.
  - The source hash covers public files: the selected tracked sources, the
    Dockerfile, the build scripts, and the reusable workflow configuration.
    `deploy/control-plane-images.md:61-67` lists them. So anyone can compute
    the hash of a tree that is not built yet.
  - On 8 November 2026, the user chose to pin the protected image workflow
    to the primary branch until the launch, and to move the pin to `main`
    at the launch. The user rejected a ruleset on the primary branch.
  - The trust move, at `deploy/image-workflow-bootstrap.md:131-170`, merges
    one reviewed change into `main`. The change switches the caller
    reference, the expected claim, and the claim of both image providers to
    `main`. The repository, the stored images, and the publisher stay.
  - The launch guide disables the prelaunch shared and trust accounts, at
    `deploy/launch.md:55-68`. It notes that a disabled provider alone
    leaves the issued tokens valid. The publisher stays enabled, because
    the preview must still publish after the launch.
  - The ruleset of `main` requires a pull request and the
    `Repository gate` check, with zero approvals, at
    `deploy/scripts/github_ruleset.py:21-29`. So a writer can merge a pull
    request of their own. Each option only makes sure that every production
    image went through a pull request and the gate on `main`. No option
    adds a second review.
  - Only the control-plane image is selected by a tag name.
    `deploy/scripts/deploy_support.py` pins the release-server image by
    digest.
  - The Dockerfile uses mutable base tags and Debian package indexes. So a
    rebuild does not give the same digest, as
    `deploy/control-plane-images.md:81-83` says.
  - The image job has a limit of 45 minutes, at
    `.github/workflows/deploy-control-plane-image.yml:91`.
- **Problem:**
  - A writer of the primary branch can publish any image bytes under the
    source tag of the reviewed launch tree. The tree after the trust move
    is public in the pull request of the move, so the writer can compute
    its hash.
  - The trust move does not remove that tag. `lookup` then accepts its
    digest, the protected workflow skips the build, and production promotes
    the planted image.
  - An issued token of the publisher stays valid after the provider
    switch. So the window stays open until that token expires.
  - The finding was reproduced: a read-only probe selected a supplied
    digest with one registry lookup.
- **Impact if no action:** Attacker code can run with the identity of the
  production control plane. It can read the production runtime secrets and
  the database.
- **Root question:** How should production accept images only from its
  trusted build process?
- **Options:**
  - **A.** Retire the old publisher, remove the prelaunch source tags, and
    rebuild after the trust move. This needs a complete operator cleanup
    after the prelaunch write access ends, and before the first preview run
    on `main`. Effort: Medium.
  - **B.** Create a fresh repository and a fresh publisher identity after
    the trust move. Give no prelaunch identity write access to them.
    - Prelaunch identities and their tokens never get write access to the
      new repository. So the order of the launch steps does not matter.
    - `immutable` at `deploy/scripts/image_registry.py:16` then also
      refuses prelaunch digests for a rollback.
    - Cost: new cloud resources and operator credentials, and a change of
      the shared repository.
    - Effort: Medium.
  - **C.** Require signed build provenance from the protected workflow
    before an image is reused. Rebuild each image that has no acceptable
    provenance.
    - C must also cover `publish`, which returns the digest of an existing
      tag in place of its own build.
    - The repository is public, so GitHub's artifact attestations are
      available. They can reduce the effort.
    - Cost: new verification infrastructure.
    - Effort: Large.
  - **D.** Change the reuse key after the prelaunch write access ends.
    - Merge a change that adds a random value to a hashed build tool, such
      as `deploy/scripts/image_context.py`. No prelaunch tag can match a
      later source hash, because no prelaunch writer knew the value.
    - No tag is deleted, and no cloud resource is added.
    - The provider switch alone does not revoke the issued tokens. So first
      replace the old publisher, or wait until its issued tokens expire.
      Then open the pull request. Run no preview on `main` until it merges.
    - Risk: a wrong order opens the hole again. An explicit rollback can
      still select a prelaunch digest.
    - Effort: Small.
  - **E.** Stop the reuse by name.
    - The protected workflow builds each accepted preview push, and it
      deploys the digest of its own archive.
    - `lookup` checks only an explicit rollback digest. `publish` always
      pushes its own digest.
    - Then no stored name can select an image, whoever writes to the
      repository, now or later. The change also removes code.
    - Cost: each preview push waits for a full release build, also a push
      that changes only docs. The registry keeps more versions.
    - E changes the reuse part of the user's choice of 7 November 2026, so
      the user must agree.
    - Effort: Small.
- **Left out:**
  - Another tag spelling made from public inputs: an attacker can compute
    it while the prelaunch writers can still write. D is the form with a
    value that no prelaunch writer knew.
  - Immutable tags: the attacker creates the tag first.
  - A reproducible rebuild with a digest comparison: a rebuild does not
    give the same digest, because the Dockerfile uses mutable base tags and
    Debian package indexes.
  - Production builds its own image: it breaks the user's choice of
    7 November 2026 that production runs the digest that the preview rolled
    out. B is also better on every point.
  - A writer check through audit logs: prelaunch pushes and trusted pushes
    used the same publisher account, so only the push time differs. A and D
    already use the time.
  - A check where production accepts the image, in
    `deploy/scripts/image_promotion.py` or with GKE Binary Authorization:
    it is not a separate choice. Production cannot build, so a refused
    image still needs the trusted rebuild of C. It extends C to rollback
    digests.
  - A ruleset on the primary branch: the user rejected it on 8 November
    2026, and it removes no tag that exists.
- **Wider fix:** C, not B.
  - B is a direct fix of this boundary only. It depends on knowing which
    identities wrote before the launch.
  - A later writer to the new repository brings the problem back. An
    example is a publisher for the pull request previews of
    `plans/follow-ups/pr-preview-environments.md`.
  - C refuses each image that has no provenance from protected `main`,
    whoever can write to the repository.
  - If the user drops the reuse, E is a simpler wider fix.
- **Previous examples:** The machine cache had the same class.
  `docs/protocol/cache.md:141-143` lets only jobs on the default branch
  write entries for every branch, because a pull request could otherwise
  poison the cache that trusted jobs read.
- **Related:** `plans/follow-ups/production-launch.md`, work that waits.
  The launch runs the trust move. Each option adds a launch step or changes
  one, so apply the answer before production launches.
- **Recommendation:** B.
  - Two facts make the order of the steps hard. The planted tag must match
    the tree after the move, and that tree is public in the pull request
    of the move. Issued tokens stay valid after the provider switch.
  - B is the only option where neither fact matters.
  - A and D must clean up or change the key after the prelaunch write
    access ends, and before the first preview run on `main`.
  - D is the small choice if the user wants no new cloud resource. E is the
    small choice that removes the whole class, if the user accepts a full
    build on each push.
  - C best protects the codebase, because it refuses every image without
    trusted provenance.
- **Action:** ask the user, because every option needs a launch step or new
  infrastructure, and the recommended B changes the shared image registry
  that the user chose on 5 November 2026.
