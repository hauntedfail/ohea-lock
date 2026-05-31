# Subtask 03: `README.md`

> **Scope:** This worker owns only `README.md`.

**Goal:** Document interactive debug usage, one-shot commands, and local-bin installation.

## Steps

- [ ] **Step 1: Replace the current `## Interactive Debug Tool` section**

Replace that section with:

````markdown
## Debug Tool

For hardware debugging and testing:

```bash
cargo run --example debug --features btleplug-support
```

Run a single command and exit:

```bash
cargo run --example debug --features btleplug-support -- state
cargo run --example debug --features btleplug-support -- lock
cargo run --example debug --features btleplug-support -- unlock
cargo run --example debug --features btleplug-support -- toggle
cargo run --example debug --features btleplug-support -- info
cargo run --example debug --features btleplug-support -- read-all
```

Install the debug example as a local command:

```bash
scripts/install-debug-bin.sh
/Users/vvx/.local/bin/ohea-lock-debug state
```
````

- [ ] **Step 2: Verify fenced code blocks**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
text = Path("README.md").read_text()
assert text.count("```") % 2 == 0
PY
```

Expected: command exits successfully.

