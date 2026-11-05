# Let each label pick a job image after launch

- **Severity:** Medium. **Effort:** Large. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-05, from `calummoore/ci-proxy-load-balancer`, the
  answer C to the question of how a workspace picks its base image.
- **Location:** `docs/protocol/labels.md`, `docs/protocol/agent.md` (The
  image), `docs/protocol/overflow.md`, `image/`, the image store of
  `crates/ciproxy-agent`, and the Boat snapshots of `crates/ciproxy-overflow`.
- **Reason:** On 5 November 2026 the user chose one image until launch, and
  a catalog of images and custom images after launch.

- **Terms:**
  - **Job image:** the disk that every job VM starts from, on own machines
    and on Boat.
  - **Label:** the name in a workflow's `runs-on:` line. It sets the VM
    size: vCPU, memory, and disk.
  - **Boat snapshot:** the saved disk that a Boat sandbox starts from.
- **Background:**
  - Until launch, every label starts from one image with the software of
    GitHub's `ubuntu-latest` runner. It uses 58.3 GB on an 80 GB job disk.
  - Boat `small` and `default` have 51 GB, so this image needs Boat
    `large` at $0.072 an hour. The default label `ciproxy`, with 2 vCPU and
    8 GB, would otherwise fit `default` at $0.036 an hour.
  - Without the largest tools, such as Android, Haskell, Swift, Julia, and
    the cached Docker images, an image of about 35 GB remains. A label of
    50 GB then leaves a job about 14 GB free, and it fits Boat `default`.
  - Until launch, a job picks other software with a job container: the
    `container:` key of GitHub Actions runs the steps in a Docker image.
  - `docs/protocol/billing.md` plans custom VM images for the Business
    tier.
  - `docs/protocol/agent.md` requires one image definition for own
    machines and Boat, so each image needs its own Boat snapshot.
  - Other runner providers let a job pick an image in its `runs-on` line,
    or attach a custom image to a named runner.
- **Task:**
  - Build a catalog of ciproxy images, at least `full` and `slim`, and let
    each label name one. The default labels keep `full`. Add a default
    label for `slim` that fits Boat `default`.
  - Download to each machine only the images that its labels use, and keep
    one Boat snapshot for each image.
  - Then add custom VM images for Business: a workspace gives a build
    definition, ciproxy builds a VM image from it, and a label names it.
  - Each step changes protocol docs, so it starts with the decisions that
    its plan needs.
- **Impact if no action:** Overflow jobs pay twice the hourly Boat price
  when their software would fit a smaller image, and a machine fits fewer
  jobs.
- **Needs:** the launched product, a second image build, and Boat
  snapshots for each image.
- **Done when:** a label can name `full` or `slim`, its jobs start from that
  image on own machines and on Boat, and a Business workspace can name a
  custom image.
