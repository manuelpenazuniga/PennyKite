#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_PORT="${TARGET_PORT:-4100}"
PROXY_PORT="${PROXY_PORT:-8787}"
DASHBOARD_PORT="${DASHBOARD_PORT:-3000}"
PRICE_USD="${PRICE_USD:-0.05}"
DB_PATH="${DB_PATH:-/tmp/pennykite-demo.db}"
LOG_DIR="${LOG_DIR:-$(mktemp -d "${TMPDIR:-/tmp}/pennykite-demo.XXXXXX")}"
TARGET_URL="http://127.0.0.1:${TARGET_PORT}"
PROXY_URL="http://127.0.0.1:${PROXY_PORT}"
DASHBOARD_URL="http://127.0.0.1:${DASHBOARD_PORT}"
UNPROTECTED_SESSION="${UNPROTECTED_SESSION:-demo-unprotected}"
PROTECTED_SESSION="${PROTECTED_SESSION:-demo-protected}"

PIDS=()

bold() { printf '\033[1m%s\033[0m\n' "$*"; }
info() { printf '\033[36m[demo]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[demo]\033[0m %s\n' "$*"; }
fail() { printf '\033[31m[demo]\033[0m %s\n' "$*" >&2; exit 1; }

cleanup() {
  local status=$?
  if ((${#PIDS[@]} > 0)); then
    info "Stopping demo services..."
    for pid in "${PIDS[@]}"; do
      kill "$pid" >/dev/null 2>&1 || true
    done
    for pid in "${PIDS[@]}"; do
      wait "$pid" >/dev/null 2>&1 || true
    done
  fi
  if [[ $status -ne 0 ]]; then
    warn "Demo failed. Logs are in: ${LOG_DIR}"
  else
    info "Logs saved in: ${LOG_DIR}"
  fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"
}

ensure_port_free() {
  local port="$1"
  python3 - "$port" <<'PY'
import socket
import sys

port = int(sys.argv[1])
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    try:
        sock.bind(("127.0.0.1", port))
    except OSError:
        sys.exit(1)
PY
}

wait_for_http() {
  local name="$1"
  local url="$2"
  local log="$3"
  for _ in $(seq 1 80); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      info "${name} is ready (${url})"
      return 0
    fi
    sleep 0.25
  done
  tail -n 80 "$log" >&2 || true
  fail "${name} did not become ready at ${url}"
}

ensure_dependencies() {
  require_cmd node
  require_cmd npm
  require_cmd cargo
  require_cmd curl
  require_cmd python3

  if [[ ! -d "${ROOT}/demo/target-api/node_modules" ]]; then
    info "Installing target API dependencies..."
    (cd "${ROOT}/demo/target-api" && npm install)
  fi

  if [[ ! -d "${ROOT}/dashboard/node_modules" ]]; then
    info "Installing dashboard dependencies..."
    (cd "${ROOT}/dashboard" && npm install)
  fi

  if [[ ! -x "${ROOT}/demo/runaway-agent/.venv/bin/python" ]]; then
    info "Creating runaway-agent virtualenv..."
    python3 -m venv "${ROOT}/demo/runaway-agent/.venv"
  fi

  if [[ ! -f "${ROOT}/demo/runaway-agent/.venv/.pennykite-installed" ]]; then
    info "Installing runaway-agent dependencies..."
    "${ROOT}/demo/runaway-agent/.venv/bin/python" -m pip install -r "${ROOT}/demo/runaway-agent/requirements.txt"
    touch "${ROOT}/demo/runaway-agent/.venv/.pennykite-installed"
  fi

  info "Building proxy binary..."
  (cd "${ROOT}" && cargo build -p pennykite-proxy)
}

start_target_api() {
  info "Starting target API on :${TARGET_PORT}"
  (
    cd "${ROOT}/demo/target-api"
    exec env PORT="${TARGET_PORT}" PRICE_USD="${PRICE_USD}" node server.js
  ) >"${LOG_DIR}/target-api.log" 2>&1 &
  PIDS+=("$!")
  wait_for_http "target API" "${TARGET_URL}/health" "${LOG_DIR}/target-api.log"
}

start_proxy() {
  rm -f "$DB_PATH" "${DB_PATH}-wal" "${DB_PATH}-shm"
  info "Starting PennyKite proxy on :${PROXY_PORT}"
  (
    cd "${ROOT}"
    exec "${ROOT}/target/debug/pennykite-proxy" \
      --listen "127.0.0.1:${PROXY_PORT}" \
      --policy "policy/examples/conservative.yaml" \
      --upstream "${TARGET_URL}" \
      --db "${DB_PATH}"
  ) >"${LOG_DIR}/proxy.log" 2>&1 &
  PIDS+=("$!")
  wait_for_http "proxy" "${PROXY_URL}/health" "${LOG_DIR}/proxy.log"
}

start_dashboard() {
  info "Starting dashboard on :${DASHBOARD_PORT}"
  (
    cd "${ROOT}/dashboard"
    exec env PENNYKITE_PROXY_URL="${PROXY_URL}" ./node_modules/.bin/next dev --port "${DASHBOARD_PORT}"
  ) >"${LOG_DIR}/dashboard.log" 2>&1 &
  PIDS+=("$!")
  wait_for_http "dashboard" "${DASHBOARD_URL}" "${LOG_DIR}/dashboard.log"
}

pause_for_enter() {
  if [[ "${PENNYKITE_DEMO_AUTO:-0}" == "1" || ! -t 0 ]]; then
    warn "Skipping ENTER pause because the script is running non-interactively."
    return
  fi
  printf '\nOpen %s to watch the live feed, then press ENTER to run the protected scenario. ' "$DASHBOARD_URL"
  read -r _
}

run_unprotected() {
  bold "1/2 - Unprotected runaway agent"
  (
    cd "${ROOT}/demo/runaway-agent"
    exec .venv/bin/python agent.py unprotected \
      --target "${TARGET_URL}" \
      --session "${UNPROTECTED_SESSION}" \
      --calls "${UNPROTECTED_CALLS:-8}" \
      --budget 5.0 \
      --match-id "runaway-direct"
  )
}

run_protected() {
  bold "2/2 - Protected agent through PennyKite"
  (
    cd "${ROOT}/demo/runaway-agent"
    exec .venv/bin/python agent.py protected \
      --proxy "${PROXY_URL}" \
      --session "${PROTECTED_SESSION}" \
      --calls "${PROTECTED_CALLS:-8}" \
      --budget 5.0 \
      --match-id "runaway-loop"
  )
}

print_final_summary() {
  bold "Demo summary"
  printf '  Dashboard      : %s\n' "$DASHBOARD_URL"
  printf '  Live feed      : %s\n' "$DASHBOARD_URL"
  printf '  Protected run  : %s/sessions/%s\n' "$DASHBOARD_URL" "$PROTECTED_SESSION"
  printf '  Kill switch    : %s/kill-switch\n' "$DASHBOARD_URL"
  printf '  Proxy sessions : %s/api/sessions\n' "$PROXY_URL"
  printf '  Logs           : %s\n' "$LOG_DIR"
}

pause_before_cleanup() {
  if [[ "${PENNYKITE_DEMO_AUTO:-0}" == "1" || ! -t 0 ]]; then
    return
  fi
  printf '\nDemo complete. Press ENTER to stop services and clean up child processes. '
  read -r _
}

main() {
  bold "PennyKite one-command demo"
  info "Repo: ${ROOT}"
  info "Logs: ${LOG_DIR}"

  ensure_port_free "$TARGET_PORT" || fail "Port ${TARGET_PORT} is already in use"
  ensure_port_free "$PROXY_PORT" || fail "Port ${PROXY_PORT} is already in use"
  ensure_port_free "$DASHBOARD_PORT" || fail "Port ${DASHBOARD_PORT} is already in use"

  ensure_dependencies
  start_target_api
  start_proxy
  start_dashboard

  printf '\n'
  info "Dashboard ready: ${DASHBOARD_URL}"
  info "Protected session id: ${PROTECTED_SESSION}"
  printf '\n'

  run_unprotected
  pause_for_enter
  run_protected
  print_final_summary
  pause_before_cleanup
}

main "$@"
