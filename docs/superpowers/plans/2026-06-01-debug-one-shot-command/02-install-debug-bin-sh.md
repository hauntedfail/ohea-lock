# Subtask 02: `scripts/install-debug-bin.sh`

> **Scope:** This worker owns only `scripts/install-debug-bin.sh`.

**Goal:** Build the debug example and install it as a stable local command at `/Users/vvx/.local/bin/ohea-lock-debug`.

## Steps

- [ ] **Step 1: Create `scripts/install-debug-bin.sh`**

```bash
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
```

- [ ] **Step 2: Mark executable**

Run:

```bash
chmod +x scripts/install-debug-bin.sh
```

Expected: script is executable.

- [ ] **Step 3: Run installer**

Run:

```bash
scripts/install-debug-bin.sh
```

Expected: Cargo builds `target/debug/examples/debug` and installs `/Users/vvx/.local/bin/ohea-lock-debug`.

- [ ] **Step 4: Verify installed binary help**

Run:

```bash
/Users/vvx/.local/bin/ohea-lock-debug --help
```

Expected: usage text prints and command exits successfully without Bluetooth initialization.

