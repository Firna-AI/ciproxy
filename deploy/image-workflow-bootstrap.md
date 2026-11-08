# Protected image workflow bootstrap

The protected `main` branch initially contains only `README.md`. Its rules
require a pull request and a passing `Repository gate`. A scripts-only
bootstrap PR cannot run that gate. The operator must first land a reviewed
complete repository tree that includes the reusable workflow, its tools,
the CI workflow, and all files that the repository gate needs.
Until that workflow exists on protected `main`, GitHub cannot resolve the
primary preview's reusable call. Those preview runs cannot start.

Use a reviewed primary revision containing this image workflow. Keep the
repository production switch false until the separate launch procedure.
Create a bootstrap PR without changing the primary branch:

```sh
git fetch origin
git switch --create operator/protected-control-plane-image \
  origin/calummoore/ci-proxy-load-balancer
git push --set-upstream origin operator/protected-control-plane-image
gh pr create --repo=Firna-AI/ciproxy --base=main \
  --head=operator/protected-control-plane-image
gh pr checks BOOTSTRAP_PR --repo=Firna-AI/ciproxy --watch
```

Review the complete bootstrap diff. Confirm `Repository gate` passes.
Then merge that reviewed PR through the protected-main rules:

```sh
gh pr merge BOOTSTRAP_PR --repo=Firna-AI/ciproxy --merge
git fetch origin main
gh api 'repos/Firna-AI/ciproxy/contents/.github/workflows/deploy-control-plane-image.yml?ref=main' \
  --jq .sha
```

Only after protected code exists, use the reviewed deployment checkout to
run [repository grant activation][grants]. It adds the restrictive claim to
both providers while preserving runtime UIDs, launch phase, and registry
readiness. Set the four image variables from that guide. Then run an accepted
preview push or rerun all jobs of its current run. The bootstrap merge can
start a preview before federation activation. Retry that run after setup.
Never bypass the required gate or use primary tools to bridge this step.
[GitHub OIDC claims][oidc-claims] define `job_workflow_sha` as the reusable
workflow revision. The request reads it directly from GitHub before checkout.

[oidc-claims]: https://docs.github.com/en/actions/reference/security/oidc
[grants]: control-plane-images.md#install-the-repository-grants
