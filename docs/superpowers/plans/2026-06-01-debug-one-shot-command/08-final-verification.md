# Subtask 08: Final Verification And Handoff

> **Scope:** This worker owns final integration verification only. Do not make feature edits unless a verification command proves a specific integration defect.

**Goal:** Verify repository changes, local binary installation, global skill contract, Codex skill evals, and Sawyer handoff readiness.

## Steps

- [ ] **Step 1: Confirm repository status**

Run:

```bash
git status --short
```

Expected: only intentional files are modified or untracked.

- [ ] **Step 2: Verify Rust formatting**

Run:

```bash
cargo fmt --all --check
```

Expected: formatter check passes.

- [ ] **Step 3: Verify Rust linting**

Run:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: clippy passes.

- [ ] **Step 4: Verify Rust tests**

Run:

```bash
cargo test --all-targets --all-features
```

Expected: all tests pass.

- [ ] **Step 5: Verify installed binary**

Run:

```bash
/Users/vvx/.local/bin/ohea-lock-debug --help
```

Expected: usage text prints and exits successfully without Bluetooth initialization.

- [ ] **Step 6: Verify global skill contract**

Run:

```bash
python3 /Users/vvx/.agents/skills/ohea-lock-debug/scripts/test_skill_contract.py
```

Expected:

```text
skill contract ok
```

- [ ] **Step 7: Verify Codex skill evals**

Run:

```bash
/Users/vvx/.agents/skills/ohea-lock-debug/evals/run-codex-skill-eval.sh
```

Expected:

```text
eval ok: unlock-ja
eval ok: open-ja
eval ok: lock-ja
eval ok: status-ja
eval ok: ambiguous-ohealock
eval ok: generic-open
```

- [ ] **Step 8: Optional live hardware smoke test**

Run only when the user explicitly wants a live lock operation:

```bash
/Users/vvx/.local/bin/ohea-lock-debug state
```

Expected: command scans, connects, initializes, prints `Lock state: Locked` or `Lock state: Unlocked`, and exits.

- [ ] **Step 9: Prepare Sawyer context pack**

Use the `sawyer` skill and delegate staging, committing, and pushing to `sawyer the cleaner` with this context:

```text
Working directory: /Users/vvx/projekt/rs/ohea-lock
Branch/remote target: current feature branch, push to origin
Work summary: debug example supports one-shot subcommands; installer copies built example binary to local bin; README documents usage; global ohea-lock-debug skill and eval tools were created outside repo
Verification: cargo fmt --all --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features; /Users/vvx/.local/bin/ohea-lock-debug --help; python3 /Users/vvx/.agents/skills/ohea-lock-debug/scripts/test_skill_contract.py; /Users/vvx/.agents/skills/ohea-lock-debug/evals/run-codex-skill-eval.sh
Intended repository files: examples/debug.rs, README.md, scripts/install-debug-bin.sh
External files not in git: /Users/vvx/.agents/skills/ohea-lock-debug/SKILL.md, /Users/vvx/.agents/skills/ohea-lock-debug/scripts/test_skill_contract.py, /Users/vvx/.agents/skills/ohea-lock-debug/evals/ohea-lock-skill-output.schema.json, /Users/vvx/.agents/skills/ohea-lock-debug/evals/run-codex-skill-eval.sh
Do not commit: target/, unrelated pre-existing user changes
Global git safety rule: no force-push, remote history rewrite, or destructive ref update without current-task user authorization
```

Expected: Sawyer reports commit SHA and push result, or a concrete blocker.
