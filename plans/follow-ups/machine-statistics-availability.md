# Unread cache and Docker statistics show as zeros

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-16, from `conductor/m26-docker-disks`, the Docker
  disks audit of Milestone 26 and its independent availability check.
- **Source:** Implementation.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-agent/src/reporting.rs:19-22` and `:68-76`,
  `crates/ciproxy-protocol/src/reports.rs:33-36`,
  `crates/ciproxy-protocol/src/cache.rs:14-28` and `:110-119`,
  `crates/ciproxy-api/src/handlers/machine_views.rs:77-84`,
  `crates/ciproxy-control-plane/src/capacity.rs:15-20`,
  `ts/api/generated/api.d.ts:895-908`,
  `docs/protocol/docker-disks.md:67-70`, and
  `docs/protocol/cache.md:178-182`.
- **Reason:** No requirement covers figures that the agent could not read,
  and the recommended fix adds that rule across the agent, the protocol,
  and the API.

- **Terms:**
  - **Machine report:** `MachineReport` in
    `crates/ciproxy-protocol/src/reports.rs:16-39`, the full machine state
    that the agent sends at registration, at each reconnect, and every 10
    seconds.
  - **Statistics sections:** the `cache` and `docker_disks` sections of the
    machine report. They hold the cache space, the seven-day restores and
    hits, the cache facts of each job, and the Docker figures.
  - **Docker figures:** the count of repositories with a Docker disk on the
    machine, the bytes of these disks, and `leased_bytes`, the bytes that
    running jobs reserve for their Docker disks.
  - **Measured zero:** a zero that the agent read, such as a machine with no
    Docker disks. The approved page shows it as "0 repositories · 0 GB".
  - **Admission guard:** the check at
    `crates/ciproxy-agent/src/launches.rs:53-63` that reads the Docker
    leases again before each runner start.
- **Background:**
  - The agent builds the machine report in `machine_report()`, at
    `crates/ciproxy-agent/src/reporting.rs:15-79`. The messages `Register`,
    `Hello`, and `Report` each carry the full report, at
    `crates/ciproxy-protocol/src/agent.rs:19-44`.
  - When the machine cache report fails, the agent logs "machine cache
    reports unavailable". It then sends `empty_cache()`: zero figures and no
    cache facts of any job. See `reporting.rs:19-22` and `:82-91`.
  - When the Docker read fails, `.ok()` drops the error without a log. The
    agent then sends zero repositories, zero bytes, and zero
    `leased_bytes`. See `reporting.rs:68-76`.
  - Production always builds the Docker reader, at
    `crates/ciproxy-agent/src/application.rs:168`. Only tests leave it out,
    at `crates/ciproxy-agent/src/_tests_/fixtures.rs:129`. The fallback
    branch therefore runs only after a failed read in production.
  - `LinuxDockers::report()`, at `crates/ciproxy-agent/src/docker.rs:174-190`,
    reads the repository disks and the lease files, through
    `crates/ciproxy-agent/src/docker_files.rs:45-66` and `:96-119`. A
    missing directory gives a true empty list. Another I/O error, or a lease
    that does not decode, fails the read.
  - One known cause is a race: a job end can remove a lease file that the
    read already listed. `plans/follow-ups/docker-lease-report-read-race.md`
    records it. After that fix, a lasting I/O or decode error still fails
    the read.
  - The statistics sections hold only numbers and no status, at
    `reports.rs:33-36`, `cache.rs:14-28`, and `cache.rs:110-119`. So a
    measured zero and a failed read reach the control plane as the same
    report.
  - Each contact replaces the stored report, at
    `crates/ciproxy-store-pg/src/gateway_contacts.rs:73-89`. Registration
    stores the first full report with the machine record in one
    transaction, at `crates/ciproxy-store-pg/src/gateway_registration.rs:81-87`.
  - One stored report that does not decode fails the machine read of its
    whole workspace, at `crates/ciproxy-store-pg/src/machine_reads.rs:96-106`.
  - The API copies the figures into the required strings of `CacheView`, at
    `crates/ciproxy-api/src/handlers/machine_views.rs:77-84` and
    `ts/api/generated/api.d.ts:895-908`.
  - A null `cache` in the API means only that the store holds no report for
    the machine. Only machine records from before the stored reports have
    none, as `crates/ciproxy-store-interface/src/gateway.rs:18-19` says.
    Such a machine is also offline, because each fresh contact stores a
    report.
  - So no machine has a phase without figures before its first report,
    because the registration message already carries a full report.
  - The app shows the figures as "0 of 0 GB" and "0 repositories · 0 GB",
    at `ts/app/src/view/machines-panels.ts:124-142`,
    `ts/app/src/view/machines-copy.ts:161-176`, and
    `ts/app/src/screens/machines/machine-cache.tsx:42-60`.
  - For a null `cache`, the app shows "Cache use shows after the machine
    reports it." with no figures, at `machines-panels.ts:124-127`. No
    mockup shows that state.
  - The approved mockups give "0 repositories · 0 GB" to machines with no
    Docker disks, at
    `docs/mockups/src/screens/machines/machine-cache.tsx:51-56` and
    `docs/mockups/src/fixtures/machines.ts:117` and `:144`. No approved
    mockup shows figures that are not known.
  - The dashboard capacity subtracts the reported `leased_bytes` from the
    job disk, at `crates/ciproxy-control-plane/src/capacity.rs:15-20`. While
    jobs hold Docker disks, a zero then overstates the job disk and the
    jobs that fit.
  - The scaler input subtracts the same figure, at
    `crates/ciproxy-control-plane/src/snapshot.rs:149-158`. Nobody traced
    what the scaler does with an overstated job disk.
  - The page has an unknown capacity state. When `capacity_known` is false,
    it omits how many jobs fit and the disk-limit notice, at
    `ts/app/src/view/machines-capacity.ts:17-23` and `:38-40`.
  - The admission guard refuses a runner start when the free job disk minus
    the leases cannot hold the VM disk. When its own lease read fails, the
    start returns that error. See `launches.rs:53-63`.
  - The machine cache budget reads the leases in the same way, at
    `crates/ciproxy-agent/src/capacity.rs:19-25`. Both reads run before any
    Docker disk attaches.
  - So the guard is a capacity check, not the optional attachment that
    `docs/protocol/docker-disks.md:54-55` describes. Nobody showed that it
    fails a job that only lost its Docker disk.
  - `docker-disks.md:69-70` says that the machine page shows how many
    repositories have a Docker disk, and their space. `cache.md:178-182`
    says that the Machines page and the machine page show the cache space
    and the seven-day hits.
  - No requirement says what these pages show, or what capacity uses, when
    the agent could not read the figures. The answer to this item sets that
    requirement.
  - The `product-ui` rules forbid a fake metric and ask for an explicit
    unavailable state. The `shared-types` rules forbid a frontend guess of a
    backend status.
  - The gateway accepts the current protocol version and the one before
    it, at `crates/ciproxy-gateway/src/versions.rs:7-17`. No protocol type
    rejects unknown fields.
  - Until the user answers, the page keeps its current behaviour: it shows
    the reported figures as measured.
  - Every option keeps three facts: stored reports still decode, a null
    `cache` still means no stored report, and the admission guard stays.
  - The cache fallback came with commit `17d96b9` of 2026-11-04. The Docker
    fallback came with commit `703b9b1` of 2026-11-06.
- **Problem:** A failed read of a statistics section reaches the page and
  the capacity figures as measured zeros. Nothing tells it apart from a
  machine with no cache entries or no Docker disks.
- **Impact if no action:** In a rare failure, the page shows a false count
  and size, and it can overstate the job disk. People can trust a false
  empty state. These reporting paths delete no Docker disk.
- **Root question:** What does ciproxy show and use for the cache and
  Docker figures of a machine when the agent cannot measure them?
- **Options:**
  - **A.** Word each zero Docker summary as possibly unread, in the app and
    the mockups, with no backend change.
    - Scope: the app and the mockups only. The test "the cache shows its
      reported use, and only an Admin clears an online cache" in
      `ts/app/_tests_/machines-detail.test.ts`, and
      `ts/app/src/screens/machines/README.md`, change with it.
    - Legacy reports: no change.
    - Capacity safety: no change. A failed read still overstates the job
      disk.
    - Rollout: one app release.
    - Cost: it changes the approved known-empty state, and it loses every
      measured zero that `docker-disks.md:69-70` asks the page to show.
    - Effort: Small.
  - **B.** Add one shared observation type that each statistics section
    uses: measured figures, or unavailable. The agent sends unavailable
    after a failed read, with a log line. The control plane, the API, and
    the app carry it to the page.
    - Scope: the agent, `ciproxy-protocol`, the dashboard capacity, the
      scaler input, the API, the generated types, the app, the mockups, and
      new lines in `docker-disks.md` and `cache.md`. Each layer gets its
      tests.
    - Legacy reports: an additive change. Stored reports and the reports of
      older agents carry no status. They still decode, and they count as
      measured. A null `cache` keeps its meaning.
    - Capacity safety: while the leases are unavailable, the page shows its
      unknown capacity state. No hosted reader counts unavailable leases as
      zero bytes. The fix branch decides how the scaler input meets that
      rule. The admission guard stays.
    - Rollout: the control plane deploys before agents that send
      unavailable.
    - Cost: the largest change. It prevents the class in both sections.
    - Effort: Large.
  - **C.** Do B, and also keep the last measured figures with their time.
    The page shows them marked as old, or unavailable when none exist.
    - Scope, legacy reports, and rollout: as B, plus stored last figures and
      a rule for their age.
    - Capacity safety: as B. No reader uses old lease figures as current.
    - Precedent: `docs/protocol/agent.md:198-199` marks the old values of an
      offline machine.
    - Cost: stored state and an age rule for a rare case.
    - Effort: Large.
  - **D.** Make only the Docker section optional, and log the read error.
    The API returns unknown Docker figures, and the app shows a new unknown
    row state with its mockup.
    - Scope: the agent, `ciproxy-protocol`, the dashboard capacity, the
      scaler input, the API, the generated types, the app, the mockups, and
      a new line in `docker-disks.md`. The cache section keeps its zero
      fallback.
    - Legacy reports: an additive change. Each stored report holds the
      Docker section, so it decodes and counts as measured.
    - Capacity safety: as B.
    - Rollout: as B, because a control plane before the change cannot
      decode a report without the Docker section.
    - Cost: a smaller change. The class stays in the cache section.
    - Effort: Medium.
- **Left out:**
  - A guess in the app or the API from zero figures, because the same
    numbers arrive in both cases.
  - A null `cache` after a failed read, because it hides measured cache
    figures and claims that no report is stored.
  - No change, because a false count and size stay.
  - A failed report after a statistics read error, as for hardware and
    images. `send_reports` then stops, at
    `crates/ciproxy-agent/src/runtime_reports.rs:53`, and the machine shows
    as offline with the wrong fix.
  - The last figures from agent memory without a marker, because a restart
    leaves none, and a lasting error shows old figures as current. C is the
    marked form.
  - A health check beside KVM and free disk, because it names a cause, and
    the figures still need B or D.
  - A log line alone, or the race fix alone, because a lasting I/O or
    decode error still gives zeros. The log line fits inside B or D.
  - A mockup of a phase before the first report, because registration
    already carries a full report. The unknown state of B, C, or D gets the
    mockup instead. That mockup can also cover a null `cache`.
- **Wider fix:** B. One shared observation type that each statistics
  section uses stops a failed read from looking like a measured value. Two
  separate optional fields would be two direct fixes.
- **Previous examples:**
  - The cache section has the same fallback, `empty_cache()` at
    `reporting.rs:19-22`, from commit `17d96b9`.
  - `plans/follow-ups/job-cache-cause.md`: the job page turns absent cache
    counts into a confirmed cause.
- **Related:**
  - `plans/follow-ups/docker-lease-report-read-race.md`: the same reader
    and the same report path. That race is one cause of a failed Docker
    read. Its direct fix does not depend on this choice, and this choice
    does not wait for that fix.
  - `plans/follow-ups/job-cache-report-writer.md`: the same code. Its future
    writer reads the cache facts of each job from the cache section, which
    `empty_cache()` empties after a failed read. Under B or C, that writer
    must not read an unavailable section as no cache use.
- **Recommendation:** B. It is the structural fix, it keeps the approved
  measured zero, and it covers the capacity use. D is the smaller direct
  alternative, and a branch may apply D first and the cache section later.
- **Action:** ask the user, because no requirement covers unread figures,
  and every option changes the agent, the protocol, and the API, or an
  approved state.
