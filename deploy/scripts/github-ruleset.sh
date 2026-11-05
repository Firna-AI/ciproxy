#!/usr/bin/env bash
# Run after the organization owner upgrades Firna-AI to GitHub Team.
set -euo pipefail
repository="${1:-Firna-AI/ciproxy}"
mkdir -p .context
scratch="$(mktemp -d "$PWD/.context/github-ruleset.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT

gh api "repos/${repository}/rulesets" --paginate --slurp > "$scratch/existing.json"
python3 deploy/scripts/github_ruleset.py "$scratch/existing.json" "$scratch/payload.json" \
  > "$scratch/id"
ruleset_id="$(cat "$scratch/id")"
if [[ -n "$ruleset_id" ]]; then
  gh api --method PUT "repos/${repository}/rulesets/${ruleset_id}" \
    --input "$scratch/payload.json" --silent
  printf 'Updated the main pull request and Repository gate ruleset.\n'
else
  gh api --method POST "repos/${repository}/rulesets" \
    --input "$scratch/payload.json" --silent
  printf 'Created the main pull request and Repository gate ruleset.\n'
fi
