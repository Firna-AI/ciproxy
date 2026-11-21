#!/usr/bin/env bash
set -euo pipefail
umask 077
daemon=$(command -v buildkitd)
runtime=$(command -v buildkit-runc)
socket_directory=$(mktemp -d /tmp/ciproxy-bk.XXXXXXXX)
chmod 700 "$socket_directory"
export BUILDKIT_HOST="unix://$socket_directory/buildkit.sock"
runner_group=$(id -g)
mkdir -p .context/control-plane-image
prefix="$PWD/.context/control-plane-image/${socket_directory##*/}"
log="$prefix.log"
readiness_error="$prefix-readiness.log"
: > "$log"
daemon_pid=$(sudo -n sh -c '
  unset RUNNER_TRACKING_ID
  nohup "$1" --containerd-worker=false --oci-worker-binary "$2" \
    --addr "$4" --root "$5" --group "$6" >"$3" 2>&1 &
  printf "%s\n" "$!"
' sh "$daemon" "$runtime" "$log" "$BUILDKIT_HOST" \
  "$socket_directory/state" "$runner_group")
for attempt in {1..30}; do
  if buildctl debug workers >"$prefix-workers.log" 2>"$readiness_error"; then
    if [[ -n ${GITHUB_ENV:-} ]]; then
      printf 'BUILDKIT_HOST=%s\n' "$BUILDKIT_HOST" >> "$GITHUB_ENV"
    fi
    printf 'export BUILDKIT_HOST=%q\nCIPROXY_BUILDKIT_PID=%q\nCIPROXY_BUILDKIT_DIRECTORY=%q\n' \
      "$BUILDKIT_HOST" "$daemon_pid" "$socket_directory" \
      > .context/control-plane-image/buildkit-env.sh
    exit 0
  fi
  sleep 1
done
case "$(cat "$readiness_error")" in
  *'permission denied'*) cause='local socket permission denied' ;;
  *'connection refused'*) cause='local socket connection refused' ;;
  *'no such file or directory'*) cause='local socket is absent' ;;
  *) cause='local worker connection did not become ready' ;;
esac
printf 'BuildKit did not start at %s: %s.\n' "$BUILDKIT_HOST" "$cause" >&2
printf 'See %s and %s.\n' "$log" "$readiness_error" >&2
exit 1
