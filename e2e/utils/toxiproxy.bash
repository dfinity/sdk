#!/usr/bin/env bash
set -euo pipefail

# Helpers for toxiproxy to use toxiproxy-server and toxiproxy-cli

: "${TOXIPROXY_HOST:=127.0.0.1}"
: "${TOXIPROXY_PORT:=8474}"

# Check if toxiproxy server is running
toxiproxy_is_running() {
  curl --silent --fail "http://${TOXIPROXY_HOST}:${TOXIPROXY_PORT}/version" >/dev/null 2>&1
}

# Start toxiproxy server
toxiproxy_start() {
  if toxiproxy_is_running; then
    return 0
  fi

  if ! command -v toxiproxy-server >/dev/null 2>&1; then
    echo "toxiproxy-server not found in PATH" >&2
    return 1
  fi

  toxiproxy-server -host "$TOXIPROXY_HOST" -port "$TOXIPROXY_PORT" >/dev/null 2>&1 &
  export E2E_TOXIPROXY_PID=$!

  for _ in $(seq 1 50); do
    if toxiproxy_is_running; then
      return 0
    fi
    sleep 0.1
  done

  echo "Toxiproxy server did not become available on ${TOXIPROXY_HOST}:${TOXIPROXY_PORT}" >&2
  return 1
}

# Stop toxiproxy server
toxiproxy_stop() {
  if [ -n "${E2E_TOXIPROXY_PID:-}" ]; then
    kill "$E2E_TOXIPROXY_PID" >/dev/null 2>&1 || true
    unset E2E_TOXIPROXY_PID
  fi
}

# Create or replace a proxy
toxiproxy_create_proxy() {
  local listen=$1 upstream=$2 name=$3

  # Ensure toxiproxy-cli is available
  if ! command -v toxiproxy-cli >/dev/null 2>&1; then
    echo "toxiproxy-cli not found in PATH" >&2
    return 1
  fi

  toxiproxy-cli delete "$name" >/dev/null 2>&1 || true
  toxiproxy-cli create --listen "$listen" --upstream "$upstream" "$name" >/dev/null 2>&1
}

# Delete a proxy
toxiproxy_delete_proxy() {
  local name=$1
  toxiproxy-cli delete "$name" >/dev/null 2>&1 || true
}

# Set a proxy to enabled or disabled
toxiproxy_toggle_proxy() {
  local name=$1
  toxiproxy-cli toggle "$name" >/dev/null 2>&1
}

# Add latency toxic (downstream)
toxiproxy_add_latency() {
  local latency=$1 jitter=$2 name=$3
  toxiproxy-cli toxic add -t latency -a latency="$latency" -a jitter="$jitter" -d "$name" >/dev/null
}
