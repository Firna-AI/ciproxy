---
title: GitHub App permissions
description: The ciproxy GitHub App asks for three permissions, and no other.
---

ciproxy works on GitHub as the ciproxy GitHub App. The App lets ciproxy
start runners for the jobs of your organization.

## Permissions

| Permission          | Access         | What ciproxy uses it for                                                     |
| ------------------- | -------------- | ---------------------------------------------------------------------------- |
| Self-hosted runners | Read and write | To start a runner for each job                                               |
| Metadata            | Read           | To list your repositories                                                    |
| Actions             | Read           | To read the event and the head repository of the run of each job from GitHub |

The App asks for no other permission. It does not ask for "Contents", and
the control plane of ciproxy never reads the code of your repositories.

ciproxy needs the event and the head repository of a job for the cache and
the Docker disks. They decide which jobs may write data that other jobs
restore, as [Job isolation](/docs/security/job-isolation/#the-cache) says.

The "Actions" permission also lets the App read the logs and the artifacts
of each run. The control plane never reads them.

## Install the App

- On GitHub, you pick the organization and its repositories.
- A member who is not an owner can ask for the install. GitHub sends the
  request to the owners. When an owner approves it, GitHub installs the App
  on the organization.
- ciproxy serves GitHub organizations only. When you install the App on a
  personal account, ciproxy offers to install it on an organization, and
  links to the GitHub settings that remove it.
- The person who installs the App on the first organization of a new
  workspace becomes its first Admin. When an owner approves the request of
  a member, the member who asked becomes the first Admin.
- An Admin adds another organization to the workspace with
  **Add an organization**, in **Settings** under **Organizations**. Several
  organizations come with Team and Business.

## Repository access

- ciproxy runs the jobs of private repositories only. A job of a public
  repository never reaches a ciproxy runner.
- GitHub decides which repositories ciproxy can see. Change them on GitHub.
- In **Settings**, under **Organizations**, allow all private repositories
  or selected repositories of each organization. All private repositories
  include the new private repositories that ciproxy can see.
- A job of a repository that you do not allow gets no ciproxy runner, and
  GitHub keeps it queued.

## Suspend or remove the App

An owner can suspend or remove the App on GitHub.

- **Suspend:** while the App is suspended, no job of the organization starts
  on a ciproxy runner. Running jobs finish, and nothing else changes. After
  an unsuspend, the jobs start on ciproxy again.
- **Remove:** the organization leaves its workspace. Its warm runners stop
  at once, and its running jobs finish. A job that a warm runner took just
  before the removal can fail.
- Boat sandboxes of the organization whose runner started end when their
  runner exits, or at their lifetime of 6 hours, or 2 hours on a Boat free
  trial.
- The scale sets and the runners of the organization stay on GitHub, because
  ciproxy has no GitHub access left to delete them.
- ciproxy deletes the data of the organization 30 days after the removal. A
  new install within those 30 days brings the organization back to its
  workspace, with its job history.
- When the last organization of a workspace removes the App, ciproxy revokes
  each machine and each install token of the workspace at once, and starts
  no new Boat runner. The subscription ends at the end of the paid period.
  ciproxy deletes the data of the workspace 30 days after the removal.
