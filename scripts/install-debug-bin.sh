#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bin_dir="${OHEA_LOCK_BIN_DIR:-$HOME/.local/bin}"
binary_name="${OHEA_LOCK_BINARY_NAME:-ohea-lock-debug}"
source_binary="$repo_root/target/debug/examples/debug"
target_binary="$bin_dir/$binary_name"

cargo build \
  --manifest-path "$repo_root/Cargo.toml" \
  --example debug \
  --features btleplug-support

mkdir -p "$bin_dir"
install -m 0755 "$source_binary" "$target_binary"

printf 'Installed %s\n' "$target_binary"
