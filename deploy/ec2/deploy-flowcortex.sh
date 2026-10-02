#!/usr/bin/env bash
# Deploy FlowCortex L1 node (+ Explorer UI) to the eGenie R&D EC2 host.
#
# Usage:
#   cp deploy/ec2/host.env.example deploy/ec2/host.env   # once
#   ./deploy/ec2/deploy-flowcortex.sh
#
# Same host/key as eGenie by default (100.52.147.238, VeerSetuHost.pem).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=linux-build.sh
source "$SCRIPT_DIR/linux-build.sh"

if [[ -f "$SCRIPT_DIR/host.env" ]]; then
  # shellcheck disable=SC1091
  source "$SCRIPT_DIR/host.env"
fi

HOST="${DEPLOY_HOST:-100.52.147.238}"
KEY="${DEPLOY_KEY:-/Users/vijay/rnd/projects/VeerSetuHost.pem}"
USER="${DEPLOY_USER:-ubuntu}"
PUBLIC_URL="${FC_PUBLIC_URL:-https://flowcortex.veerlabs.solutions}"
API_PUBLIC_URL="${FC_API_PUBLIC_URL:-https://flowcortex-api.veerlabs.solutions}"
FC_L1_PORT="${FC_L1_PORT:-8200}"
FC_EXPLORER_PORT="${FC_EXPLORER_PORT:-8201}"
FC_GRPC_PORT="${FC_GRPC_PORT:-50052}"
FLOWCORTEX_ROOT="${FLOWCORTEX_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
DEPLOY_EXPLORER="${DEPLOY_EXPLORER:-1}"

linux_build_init "$HOST" "$KEY" "$USER"
SSH="$LINUX_BUILD_SSH"

if [[ ! -f "$KEY" ]]; then
  echo "SSH key not found: $KEY" >&2
  echo "Copy deploy/ec2/host.env.example → host.env and set DEPLOY_KEY." >&2
  exit 1
fi

CARGO_BUILD_CMD="cargo build --release --manifest-path flowcortex-l1/Cargo.toml"
if [[ "$DEPLOY_EXPLORER" == "1" ]]; then
  CARGO_BUILD_CMD="$CARGO_BUILD_CMD && cargo build --release --manifest-path explorer/Cargo.toml"
fi

BUILD_MODE_RESOLVED="$(linux_build_resolve_mode)"
echo "==> FlowCortex deploy to $HOST (build mode: $BUILD_MODE_RESOLVED)"

SKIP_EC2_BUILD=0
if [[ "${SKIP_BUILD:-0}" == "1" ]]; then
  SKIP_EC2_BUILD=1
elif [[ "$BUILD_MODE_RESOLVED" == "lan" ]]; then
  linux_build_on_lan "$FLOWCORTEX_ROOT" "$CARGO_BUILD_CMD" "$(linux_build_lan_cargo_target)"
  if ! linux_build_ec2_reachable; then
    linux_build_lan_only_hint
    exit 0
  fi
  SKIP_EC2_BUILD=1
fi

FC_EC2_TARGET="$(linux_build_ec2_cargo_target)"

if ! linux_build_ec2_reachable; then
  echo "EC2 ($HOST) is not reachable" >&2
  exit 1
fi

echo "==> Sync FlowCortex to $HOST:/opt/flow-cortex"
$SSH 'sudo mkdir -p /opt/flow-cortex /var/lib/flow-cortex /var/lib/flowcortex-cargo-target /etc/flowcortex && sudo chown -R ubuntu:ubuntu /opt/flow-cortex /var/lib/flow-cortex /var/lib/flowcortex-cargo-target'
linux_build_rsync_sources "$FLOWCORTEX_ROOT" "${USER}@$HOST:/opt/flow-cortex"

if [[ "$SKIP_EC2_BUILD" -eq 1 ]]; then
  echo "==> Install LAN-built binaries on EC2"
  STAGE="$(mktemp -d)"
  trap 'rm -rf "$STAGE"' EXIT
  rsync -az -e "$LAN_RSYNC" \
    "${BUILD_HOST}:$(linux_build_lan_cargo_target)/release/flowcortex-l1" "$STAGE/" 2>/dev/null || true
  if [[ "$DEPLOY_EXPLORER" == "1" ]]; then
    rsync -az -e "$LAN_RSYNC" \
      "${BUILD_HOST}:$(linux_build_lan_cargo_target)/release/flowcortex-explorer" "$STAGE/" 2>/dev/null || true
  fi
  $SSH "mkdir -p ${FC_EC2_TARGET}/release"
  rsync -az -e "$LINUX_BUILD_RSYNC" "$STAGE/" "${USER}@$HOST:${FC_EC2_TARGET}/release/"
fi

echo "==> Remote install"
$SSH "SKIP_EC2_BUILD=$SKIP_EC2_BUILD FC_CARGO_TARGET=$FC_EC2_TARGET FORCE_BUILD=${FORCE_BUILD:-0} DEPLOY_EXPLORER=$DEPLOY_EXPLORER FC_L1_PORT=$FC_L1_PORT FC_EXPLORER_PORT=$FC_EXPLORER_PORT FC_GRPC_PORT=$FC_GRPC_PORT FC_PUBLIC_URL=$PUBLIC_URL FC_API_PUBLIC_URL=$API_PUBLIC_URL bash -s" <<'REMOTE'
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive

# Rust toolchain
if ! command -v rustc >/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi
# shellcheck disable=SC1091
source "$HOME/.cargo/env"
rustup default stable

sudo apt-get update -qq
sudo apt-get install -y -qq build-essential pkg-config libssl-dev clang libc6-dev protobuf-compiler

export FC_CARGO_TARGET="${FC_CARGO_TARGET:-/var/lib/flowcortex-cargo-target}"
mkdir -p "$FC_CARGO_TARGET/release"

FC_L1_PORT="${FC_L1_PORT:-8200}"
FC_EXPLORER_PORT="${FC_EXPLORER_PORT:-8201}"
FC_GRPC_PORT="${FC_GRPC_PORT:-50052}"

if [[ "${SKIP_EC2_BUILD:-0}" != "1" && "${SKIP_BUILD:-0}" != "1" ]]; then
  NEED_L1_BUILD=0
  NEED_EXPLORER_BUILD=0
  if [[ "${FORCE_BUILD:-0}" == "1" || ! -x "$FC_CARGO_TARGET/release/flowcortex-l1" ]]; then
    NEED_L1_BUILD=1
  fi
  if [[ "${DEPLOY_EXPLORER:-1}" == "1" ]]; then
    if [[ "${FORCE_BUILD:-0}" == "1" || ! -x "$FC_CARGO_TARGET/release/flowcortex-explorer" ]]; then
      NEED_EXPLORER_BUILD=1
    fi
  fi

  if [[ "$NEED_L1_BUILD" -eq 1 ]]; then
    echo "Building flowcortex-l1 on EC2 (cache: $FC_CARGO_TARGET)..."
    cd /opt/flow-cortex
    CARGO_TARGET_DIR="$FC_CARGO_TARGET" cargo build --release --manifest-path flowcortex-l1/Cargo.toml
  else
    echo "Skipping flowcortex-l1 compile (cached; FORCE_BUILD=1 to rebuild)"
  fi

  if [[ "${DEPLOY_EXPLORER:-1}" == "1" && "$NEED_EXPLORER_BUILD" -eq 1 ]]; then
    echo "Building flowcortex-explorer on EC2..."
    cd /opt/flow-cortex
    CARGO_TARGET_DIR="$FC_CARGO_TARGET" cargo build --release --manifest-path explorer/Cargo.toml
  elif [[ "${DEPLOY_EXPLORER:-1}" == "1" ]]; then
    echo "Skipping explorer compile (cached; FORCE_BUILD=1 to rebuild)"
  fi
else
  echo "Skipping EC2 compile (LAN-built or SKIP_BUILD)"
fi

sudo install -m 755 "$FC_CARGO_TARGET/release/flowcortex-l1" /usr/local/bin/flowcortex-l1
if [[ "${DEPLOY_EXPLORER:-1}" == "1" && -x "$FC_CARGO_TARGET/release/flowcortex-explorer" ]]; then
  sudo install -m 755 "$FC_CARGO_TARGET/release/flowcortex-explorer" /usr/local/bin/flowcortex-explorer
fi

# Persist node state outside the rsync tree when possible
if [[ -f /opt/flow-cortex/flowcortex-l1/node_state.json && ! -f /var/lib/flow-cortex/node_state.json ]]; then
  cp /opt/flow-cortex/flowcortex-l1/node_state.json /var/lib/flow-cortex/node_state.json
fi
if [[ -f /var/lib/flow-cortex/node_state.json ]]; then
  ln -sf /var/lib/flow-cortex/node_state.json /opt/flow-cortex/flowcortex-l1/node_state.json
fi

sudo tee /etc/flowcortex/l1.env >/dev/null <<ENV
BIND_ADDR=127.0.0.1:${FC_L1_PORT}
GRPC_ADDR=127.0.0.1:${FC_GRPC_PORT}
TLS_ENABLED=false
ENV
sudo chmod 600 /etc/flowcortex/l1.env

sudo tee /etc/systemd/system/flowcortex-l1.service >/dev/null <<'UNIT'
[Unit]
Description=FlowCortex L1 Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=ubuntu
Group=ubuntu
EnvironmentFile=/etc/flowcortex/l1.env
WorkingDirectory=/opt/flow-cortex/flowcortex-l1
ExecStart=/usr/local/bin/flowcortex-l1
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
UNIT

if [[ "${DEPLOY_EXPLORER:-1}" == "1" && -x /usr/local/bin/flowcortex-explorer ]]; then
  sudo tee /etc/flowcortex/explorer.env >/dev/null <<ENV
BIND_ADDR=127.0.0.1:${FC_EXPLORER_PORT}
L1_API_BASE=http://127.0.0.1:${FC_L1_PORT}
TLS_ENABLED=false
ENV
  sudo chmod 600 /etc/flowcortex/explorer.env

  sudo tee /etc/systemd/system/flowcortex-explorer.service >/dev/null <<'UNIT'
[Unit]
Description=FlowCortex Explorer UI
After=network-online.target flowcortex-l1.service
Wants=network-online.target flowcortex-l1.service

[Service]
Type=simple
User=ubuntu
Group=ubuntu
EnvironmentFile=/etc/flowcortex/explorer.env
WorkingDirectory=/opt/flow-cortex/explorer
ExecStart=/usr/local/bin/flowcortex-explorer
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
UNIT
fi

PUBLIC_URL="${FC_PUBLIC_URL:-https://flowcortex.veerlabs.solutions}"
API_PUBLIC_URL="${FC_API_PUBLIC_URL:-https://flowcortex-api.veerlabs.solutions}"
EXPLORER_HOST="$(echo "$PUBLIC_URL" | sed -E 's#https?://##')"
API_HOST="$(echo "$API_PUBLIC_URL" | sed -E 's#https?://##')"

if command -v caddy >/dev/null; then
  if [[ "${DEPLOY_EXPLORER:-1}" == "1" ]] && ! sudo grep -q "$EXPLORER_HOST" /etc/caddy/Caddyfile 2>/dev/null; then
    echo "==> Adding Caddy block for $EXPLORER_HOST"
    sudo tee -a /etc/caddy/Caddyfile >/dev/null <<CADDY

${EXPLORER_HOST} {
	reverse_proxy 127.0.0.1:${FC_EXPLORER_PORT}
}
CADDY
  fi
  if ! sudo grep -q "$API_HOST" /etc/caddy/Caddyfile 2>/dev/null; then
    echo "==> Adding Caddy block for $API_HOST"
    sudo tee -a /etc/caddy/Caddyfile >/dev/null <<CADDY

${API_HOST} {
	reverse_proxy 127.0.0.1:${FC_L1_PORT}
}
CADDY
  fi
  sudo systemctl reload caddy || sudo systemctl restart caddy
fi

if [[ -f /etc/egenie/idle-shutdown.conf ]]; then
  if sudo grep -q '^MONITOR_PORTS=' /etc/egenie/idle-shutdown.conf; then
    for port in "${FC_L1_PORT}" "${FC_EXPLORER_PORT}"; do
      CURRENT=$(sudo grep '^MONITOR_PORTS=' /etc/egenie/idle-shutdown.conf | head -1)
      if ! echo "$CURRENT" | grep -q "$port"; then
        sudo sed -i "s/^MONITOR_PORTS=\"\\([^\"]*\\)\"/MONITOR_PORTS=\"\\1 ${port}\"/" /etc/egenie/idle-shutdown.conf
      fi
    done
  fi
fi

sudo systemctl daemon-reload
sudo systemctl enable flowcortex-l1
sudo systemctl restart flowcortex-l1
if [[ "${DEPLOY_EXPLORER:-1}" == "1" && -f /etc/systemd/system/flowcortex-explorer.service ]]; then
  sudo systemctl enable flowcortex-explorer
  sudo systemctl restart flowcortex-explorer
fi

sleep 3
curl -sf "http://127.0.0.1:${FC_L1_PORT}/pool" | head -c 200 && echo
sudo systemctl --no-pager status flowcortex-l1 | head -12
if [[ "${DEPLOY_EXPLORER:-1}" == "1" ]]; then
  curl -sf "http://127.0.0.1:${FC_EXPLORER_PORT}/" | head -c 80 && echo
  sudo systemctl --no-pager status flowcortex-explorer | head -12
fi
REMOTE

echo ""
echo "Deploy complete."
echo "  Host:       $HOST"
echo "  Explorer:   $PUBLIC_URL"
echo "  L1 API:     $API_PUBLIC_URL"
echo "  Pool:       curl $API_PUBLIC_URL/pool"
echo "  gRPC:       127.0.0.1:$FC_GRPC_PORT (localhost only)"
echo ""
echo "DNS:"
echo "  flowcortex.veerlabs.solutions      → $HOST"
echo "  flowcortex-api.veerlabs.solutions  → $HOST"
