#!/usr/bin/env bash
# Linux build routing for FlowCortex EC2 deploys.
#
# Default (BUILD_MODE=auto): compile on EC2 when SSH works; otherwise use the
# LAN builder at 192.168.29.78 (same pattern as eGenie deploy).
#
# Override: BUILD_MODE=ec2 | lan

linux_build_init() {
  local host="$1" key="$2" user="${3:-ubuntu}"
  LINUX_BUILD_USER="$user"
  LINUX_BUILD_HOST="${DEPLOY_HOST:-$host}"
  LINUX_BUILD_KEY="${DEPLOY_KEY:-$key}"
  LINUX_BUILD_SSH="ssh -i $LINUX_BUILD_KEY -o BatchMode=yes -o StrictHostKeyChecking=accept-new ${LINUX_BUILD_USER}@$LINUX_BUILD_HOST"
  LINUX_BUILD_RSYNC="ssh -i $LINUX_BUILD_KEY -o StrictHostKeyChecking=accept-new"

  BUILD_HOST="${BUILD_HOST:-vijay@192.168.29.78}"
  BUILD_ROOT="${BUILD_ROOT:-/home/vijay/flowcortex-ec2-build}"
  if [[ -n "${RSYNC_SSH:-}" ]]; then
    LAN_SSH="$RSYNC_SSH"
    LAN_RSYNC="$RSYNC_SSH"
  elif [[ -n "${SSHPASS:-}" ]] && command -v sshpass >/dev/null 2>&1; then
    LAN_SSH="sshpass -e ssh -o PreferredAuthentications=password -o PubkeyAuthentication=no -o StrictHostKeyChecking=accept-new"
    LAN_RSYNC="$LAN_SSH"
  else
    LAN_SSH="${LAN_SSH:-ssh -o ConnectTimeout=8 -o BatchMode=yes}"
    LAN_RSYNC="$LAN_SSH"
  fi
}

linux_build_ec2_reachable() {
  $LINUX_BUILD_SSH -o ConnectTimeout=8 'echo ok' >/dev/null 2>&1
}

linux_build_lan_reachable() {
  if ! $LAN_SSH "$BUILD_HOST" 'echo ok' >/dev/null 2>&1; then
    if [[ -z "${SSHPASS:-}" ]] && [[ "$LAN_SSH" == ssh* ]]; then
      echo "LAN builder ($BUILD_HOST) unreachable via SSH key." >&2
      echo "Set SSHPASS + sshpass, or export RSYNC_SSH (see eGenie deploy/ec2/linux-build.sh)." >&2
    fi
    return 1
  fi
}

linux_build_resolve_mode() {
  case "${BUILD_MODE:-auto}" in
    ec2)
      linux_build_ec2_reachable || {
        echo "BUILD_MODE=ec2 but EC2 ($LINUX_BUILD_HOST) is not reachable" >&2
        return 1
      }
      echo ec2
      ;;
    lan)
      linux_build_lan_reachable || {
        echo "BUILD_MODE=lan but LAN builder ($BUILD_HOST) is not reachable" >&2
        return 1
      }
      echo lan
      ;;
    auto)
      if linux_build_ec2_reachable; then
        echo ec2
      elif linux_build_lan_reachable; then
        echo lan
      else
        echo "Neither EC2 ($LINUX_BUILD_HOST) nor LAN builder ($BUILD_HOST) is reachable" >&2
        return 1
      fi
      ;;
    *)
      echo "Invalid BUILD_MODE=${BUILD_MODE} (use auto, ec2, or lan)" >&2
      return 1
      ;;
  esac
}

linux_build_ec2_rust_deps() {
  cat <<'DEPS'
export DEBIAN_FRONTEND=noninteractive
if ! command -v rustc >/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi
# shellcheck disable=SC1091
source "$HOME/.cargo/env"
rustup default stable
sudo apt-get update -qq
sudo apt-get install -y -qq build-essential pkg-config libssl-dev clang libc6-dev protobuf-compiler
DEPS
}

linux_build_lan_only_hint() {
  echo ""
  echo "Binaries built on LAN ($BUILD_HOST). EC2 is stopped or unreachable."
  echo "Start the instance, then re-run: ./deploy/ec2/deploy-flowcortex.sh"
  echo "BUILD_MODE=lan skips rebuild on EC2 when the host is back up."
}

linux_build_ec2_cargo_target() {
  echo "/var/lib/flowcortex-cargo-target"
}

linux_build_lan_cargo_target() {
  echo "${BUILD_ROOT}/cargo-target"
}

linux_build_rsync_sources() {
  local local_path="$1"
  local remote_spec="$2"
  rsync -az --delete \
    --exclude target --exclude .git --exclude node_modules \
    --exclude 'flowcortex-l1/node_state.json' \
    --exclude 'flowcortex-l1/blocks.log' \
    --exclude 'examples/l1-integration-clients/typescript/node_modules' \
    --filter 'P target/' \
    -e "$LINUX_BUILD_RSYNC" \
    "$local_path/" "$remote_spec/"
}

linux_build_rsync_sources_lan() {
  local local_path="$1"
  local remote_spec="$2"
  rsync -az --delete \
    --exclude target --exclude .git --exclude node_modules \
    --exclude 'flowcortex-l1/node_state.json' \
    --exclude 'flowcortex-l1/blocks.log' \
    --exclude 'examples/l1-integration-clients/typescript/node_modules' \
    --filter 'P target/' \
    -e "$LAN_RSYNC" \
    "$local_path/" "$remote_spec/"
}

linux_build_ensure_ec2_cargo_target() {
  local target_dir
  target_dir="$(linux_build_ec2_cargo_target)"
  $LINUX_BUILD_SSH "sudo mkdir -p '$target_dir' && sudo chown ubuntu:ubuntu /var/lib/flowcortex-cargo-target '$target_dir'"
}

linux_build_on_lan() {
  local repo_path="$1"
  local cargo_cmd="$2"
  local cargo_target="${3:-$(linux_build_lan_cargo_target)}"

  echo "==> Sync to LAN builder $BUILD_HOST"
  $LAN_SSH "$BUILD_HOST" "mkdir -p ${BUILD_ROOT}/flow-cortex '$cargo_target'"
  linux_build_rsync_sources_lan "$repo_path" "${BUILD_HOST}:${BUILD_ROOT}/flow-cortex"

  echo "==> cargo build on LAN target=$cargo_target"
  $LAN_SSH "$BUILD_HOST" "bash -s" <<REMOTE
set -euo pipefail
export PATH="\$HOME/.cargo/bin:\$PATH"
# shellcheck disable=SC1091
source "\$HOME/.cargo/env" 2>/dev/null || true
export CARGO_TARGET_DIR="${cargo_target}"
export CARGO_BUILD_JOBS="\${CARGO_BUILD_JOBS:-4}"
mkdir -p "\$CARGO_TARGET_DIR"
cd "${BUILD_ROOT}/flow-cortex"
${cargo_cmd}
REMOTE
}
