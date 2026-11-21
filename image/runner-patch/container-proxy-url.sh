#!/usr/bin/env bash
set -euo pipefail

cache_proxy_port="${1:-8230}"
cache_bridge_name="${CIPROXY_DOCKER_BRIDGE:-bridge}"
cache_bridge_gateway="$(docker network inspect "$cache_bridge_name" --format '{{(index .IPAM.Config 0).Gateway}}')"
printf 'http://%s:%s\n' "$cache_bridge_gateway" "$cache_proxy_port"
