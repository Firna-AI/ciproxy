# Release server

The download host runs `futex-ai/cli-release`. It serves the install script,
latest agent metadata, and release archives from `Firna-AI/ciproxy`.
The installer and agent updates use this server.

## Source and image

The pinned upstream revision is
[`53a1dd3426226a9d25defb4a899d3c9a430a3941`](https://github.com/futex-ai/cli-release/tree/53a1dd3426226a9d25defb4a899d3c9a430a3941).
The chart under `helm/cli-release-server/` is an exact upstream copy.
Its `PROVENANCE.json` records the source revision and SHA-256 of each file.
Keep local Helm overrides in `helm/release-values.yaml` and the values that
the deploy command renders.

`scripts/mirror-release-image.sh` builds that source by default.
Its optional `published` mode copies `sha-53a1dd342622` from upstream GHCR.
Both modes write `upstream-53a1dd342622` in the shared runtime registry.
The image uses Linux AMD64.

Deploy uses `upstream-53a1dd342622@sha256:<digest>`.
`RELEASE_SERVER_IMAGE_DIGEST` supplies the digest in each GitHub environment.
The digest selects the image bytes. A source-pin update alone does not
update a running server. The operator must mirror the new revision, set
the digest, and redeploy. See [image setup](setup.md#3-push-the-release-image).

## Release channels

The deploy command writes `config.releaseChannel` into the release values
for the selected environment. The chart passes it as `CLI_RELEASE_CHANNEL`.
The upstream default is `stable`.

| Environment | Channel | Latest release selection |
| --- | --- | --- |
| `main-preview` | `prerelease` | First matching, non-draft release in GitHub release-list order. It can be a pre-release or a full release. |
| `production` | `stable` | GitHub's latest full release. It never follows pre-releases. |

Both environments read the same repository with tag prefix `ciproxy-agent-v`.
No GitHub variable or ambient `CLI_RELEASE_CHANNEL` overrides this choice.
The channel applies to latest metadata routes and thus to installs and
agent updates. Exact versioned downloads still resolve their release tag.

## Install and archives

The generated installer installs the selected binary.
With script arguments, it then uses `exec` to run that binary with exactly
those arguments. It removes temporary files before this handoff.
The caller receives the binary's exit status.
Without arguments, the script only installs the binary.
The script and server do not log argument values.

The agent install command therefore runs `install --token` after download.
See the [agent install contract](../docs/protocol/agent.md#install).

The archive contract is unchanged at this revision.
The repository's `scripts/cli-release.sh package` writes
`<binary>-<version>-<target>.tar.gz` and its `.sha256` sidecar.
The archive contains `<binary>-<version>-<target>/<binary>` as an executable.
The sidecar holds the archive's hexadecimal SHA-256.
The agent and cache sidecar archives also contain their Apache-2.0 licence.
The upstream installer selects the binary by its member name and verifies
the checksum before installing it.

## Verify a rollout

The operator runs these checks after mirroring the pinned revision,
setting its digest, and completing a fresh deployment.
Use the environment variables from [setup](setup.md).

```sh
kubectl rollout status "deployment/ciproxy-$DEPLOY_ENV-release-cli-release-server" \
  --namespace="$NAMESPACE"
kubectl get deployment "ciproxy-$DEPLOY_ENV-release-cli-release-server" \
  --namespace="$NAMESPACE" \
  -o jsonpath='{.spec.template.spec.containers[0].image}{"\n"}'
kubectl get deployment "ciproxy-$DEPLOY_ENV-release-cli-release-server" \
  --namespace="$NAMESPACE" \
  -o jsonpath='{.spec.template.spec.containers[0].env[?(@.name=="CLI_RELEASE_CHANNEL")].value}{"\n"}'
```

Check that the image has the new tag and the mirrored digest.
Check that main preview reports `prerelease` and production reports `stable`.
Check latest metadata on each deployed download host:

```sh
curl -fsS "https://$RELEASE_HOST/releases/latest/ciproxy-agent/x86_64-unknown-linux-gnu"
```

With a newer matching pre-release available, preview must select it.
Production must still select the latest full release.
On a test machine, run the dashboard's install command.
Confirm that the installer starts `ciproxy-agent install --token` and
registers the machine. Keep the token and the command arguments out of logs.
