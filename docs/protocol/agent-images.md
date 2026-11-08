# Agent images

This doc states the image requirements for [job VMs](agent.md#job-vms).

- The image builds on the open-source GitHub runner images.
- The image has the software of the GitHub runner image that
  `ubuntu-latest` names, so a workflow keeps its YAML apart from the
  `runs-on` line.
- The image root is built at the smallest size that holds the full image.
- Each job root disk reserves 1% of its blocks for root.
- The free disk of the label remains available to the unprivileged runner,
  as `df` reports it at job start.
- The runner in the image is always a version that GitHub still accepts.
- So ciproxy releases a new image before GitHub stops accepting the runner
  of the current image.
- Each image release supplies its logical root disk bytes.
- The agent keeps those facts with the verified image and its preparation hold.
- It calculates the root allocation from the held release and the free
  label disk.
- A job VM starts from the newest image that its machine has, and a
  running job keeps its image.
- Each runner retains its original free-size request and actual allocation.
- Publication and restart do not change those facts for an existing runner.
- **Decision:** one image definition serves the job VMs and the Boat
  sandboxes. Rejected: a separate image for each, because a job must behave
  the same wherever it runs.
- A job can run its steps in a Docker image of its own choice, with the
  `container:` key of GitHub Actions, on every tier.
- **Decision:** every label starts from the one image until launch, and a
  job picks other software with a job container. Rejected for now: a
  catalog of images that each label names, custom VM images, and toolset
  layers. Reason: the user chose this on 5 November 2026, and plans the
  catalog and custom images after launch.
- No workspace can run its jobs on a custom VM image yet.
- Custom VM images come later, on Business, as
  [Tier features](billing.md#tier-features) says.
