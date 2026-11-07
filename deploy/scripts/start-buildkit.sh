#!/usr/bin/env bash
set -euo pipefail
daemon=$(command -v buildkitd)
runtime=$(command -v buildkit-runc)
mkdir -p .context/control-plane-image
log="$PWD/.context/control-plane-image/buildkit.log"
sudo -n sh -c '
  unset RUNNER_TRACKING_ID
  nohup "$1" --containerd-worker=false --oci-worker-binary "$2" >"$3" 2>&1 &
' sh "$daemon" "$runtime" "$log"
for attempt in {1..30}; do
  if sudo -n test -S /run/buildkit/buildkitd.sock; then
    sudo -n chown "$(id -u):$(id -g)" /run/buildkit/buildkitd.sock
    if buildctl debug workers >/dev/null 2>&1; then
      exit 0
    fi
  fi
  sleep 1
done
echo 'BuildKit did not start. Check .context/control-plane-image/buildkit.log.' >&2
exit 1
