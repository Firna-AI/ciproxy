# Runtime token job identity sources

This guide traces the Worker job ID through pinned GitHub sources. The
[crate README](README.md#runtime-token-job-identity) gives the job ID check
of the runtime token and its live proof.

## What the sources show

Pinned GitHub sources tie the runtime token to the Worker job ID through
the Results service. The Worker gets a [UUID `JobId`][message] in its job
request. The runner sends it to Results as
[`WorkflowJobRunBackendId`][backend] through its [Results queue][queue].
GitHub's artifact client reads that field from the
[`Actions.Results` scope][scope] of the `scp` claim in
[`ACTIONS_RUNTIME_TOKEN`][token-env], the original job access token.

## What the sources do not show

These sources do not define the top-level `job_id` claim of the token. They
do not link it to the Worker job ID, the [scale-set `jobId`][scale-set], or
the [Actions REST job ID][rest-job]. So they cannot select one of these IDs
for the signed `job_id` check.

[message]: https://github.com/actions/runner/blob/d1eb12d4b115c301201de27d667284e1a80f2ab5/src/Sdk/DTPipelines/Pipelines/AgentJobRequestMessage.cs#L112-L117
[backend]: https://github.com/actions/runner/blob/d1eb12d4b115c301201de27d667284e1a80f2ab5/src/Sdk/WebApi/WebApi/ResultsHttpClient.cs#L98-L108
[queue]: https://github.com/actions/runner/blob/d1eb12d4b115c301201de27d667284e1a80f2ab5/src/Runner.Common/JobServerQueue.cs#L108-L162
[scope]: https://github.com/actions/toolkit/blob/0c3a9c332b678ff1e776f93d728a4ad279803abe/packages/artifact/src/internal/shared/util.ts#L19-L68
[token-env]: https://github.com/actions/runner/blob/d1eb12d4b115c301201de27d667284e1a80f2ab5/src/Runner.Worker/Handlers/NodeScriptActionHandler.cs#L54-L73
[scale-set]: https://github.com/actions/scaleset/blob/716908e507993947ada4273aecf2361c02e5836c/types.go#L26-L61
[rest-job]: https://github.com/github/rest-api-description/blob/c9c95fe34ced05fe076001f5c55704742bc8e7f8/descriptions/api.github.com/api.github.com.yaml#L285107-L285114
