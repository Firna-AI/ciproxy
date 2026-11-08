# The installer supports few Linux distributions

- **Severity:** Low. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from plan item 93, option D, which the user asked
  to record as a follow-up item.
- **Source:** Implementation, pull request #40.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-agent/src/host_packages.rs:39`.
- **Reason:** Item 93 = C states today's support. This item widens it.

- **Terms:**
  - **cgroup v2:** the Linux feature that the agent uses to limit the
    resources of each job VM.
- **Background:** The installer accepts only Debian, Ubuntu, Fedora, RHEL,
  Rocky Linux, and AlmaLinux, with systemd and cgroup v2 on x64. Item
  93 = C states these limits in the protocol, the install note, and the
  docs.
- **Problem:** A customer with another Linux distribution cannot install
  the agent.
- **Impact if no action:** Some customers cannot use their machines.
- **Root question:** Which further hosts does the installer support?
- **Options:**
  - **A.** Add the next most common distributions, such as openSUSE,
    Arch, and Amazon Linux, with their package names. Effort: Medium.
  - **B.** Detect the host tools instead of the distribution name, and
    install missing packages through a short list of package managers.
    Effort: Large.
- **Left out:** Hosts without systemd or cgroup v2, because the agent
  needs both.
- **Wider fix:** B.
- **Recommendation:** B. It covers new distributions without a code change.
- **Action:** fix after merge, apply B, because the user asked to widen
  the support after the MVP.
