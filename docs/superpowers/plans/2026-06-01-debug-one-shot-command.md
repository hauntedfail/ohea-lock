# Debug One-Shot Command Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add one-shot subcommands to the Ohea Lock debug example, install the built binary as a local command, and create a globally available skill that maps natural-language Ohea Lock requests to safe one-shot operations.

**Architecture:** This plan is split by target file so each subagent or worker can load only the document for the file it owns. The index below is the routing document; do not load every subtask unless you own integration or final verification.

**Tech Stack:** Rust 2024, `btleplug`, `tokio`, Cargo examples, POSIX shell, Codex skill files, Codex JSONL evals.

---

## Dispatch Map

Each worker should read this file plus exactly one subtask file unless it is explicitly assigned final integration.

| Subtask | Target file | Worker document |
|---|---|---|
| CLI parser and one-shot runtime | `examples/debug.rs` | `docs/superpowers/plans/2026-06-01-debug-one-shot-command/01-examples-debug-rs.md` |
| Local binary installer | `scripts/install-debug-bin.sh` | `docs/superpowers/plans/2026-06-01-debug-one-shot-command/02-install-debug-bin-sh.md` |
| README usage docs | `README.md` | `docs/superpowers/plans/2026-06-01-debug-one-shot-command/03-readme-md.md` |
| Global skill definition | `/Users/vvx/.agents/skills/ohea-lock-debug/SKILL.md` | `docs/superpowers/plans/2026-06-01-debug-one-shot-command/04-skill-md.md` |
| Deterministic skill contract test | `/Users/vvx/.agents/skills/ohea-lock-debug/scripts/test_skill_contract.py` | `docs/superpowers/plans/2026-06-01-debug-one-shot-command/05-test-skill-contract-py.md` |
| Codex eval output schema | `/Users/vvx/.agents/skills/ohea-lock-debug/evals/ohea-lock-skill-output.schema.json` | `docs/superpowers/plans/2026-06-01-debug-one-shot-command/06-eval-schema-json.md` |
| Codex eval runner | `/Users/vvx/.agents/skills/ohea-lock-debug/evals/run-codex-skill-eval.sh` | `docs/superpowers/plans/2026-06-01-debug-one-shot-command/07-run-codex-skill-eval-sh.md` |
| Final integration and Sawyer handoff | multiple files | `docs/superpowers/plans/2026-06-01-debug-one-shot-command/08-final-verification.md` |

## Execution Rules

- Keep ownership file-scoped. A worker assigned `README.md` must not edit `examples/debug.rs`.
- External skill files under `/Users/vvx/.agents/skills/ohea-lock-debug/` are outside this repository and may need sandbox approval.
- Automated skill verification is mandatory: both `test_skill_contract.py` and `run-codex-skill-eval.sh` must pass before completion.
- The Codex eval must not operate hardware. It must inspect agent decisions and JSONL traces, including verifying that no `command_execution` events occurred during eval cases.
- After repository changes are complete and verified, follow the repo instruction to use the `sawyer` skill for staging, committing, and pushing.

## Success Criteria

- `cargo run --example debug --features btleplug-support -- --help` exits before Bluetooth initialization.
- `cargo test --example debug --features btleplug-support` passes.
- `/Users/vvx/.local/bin/ohea-lock-debug --help` works after installer execution.
- `ohealock 鍵を開けて`, `ohealock 解錠`, and `ohealock 開けて` map to unlock in the skill and eval.
- `ohealock 鍵を閉めて`, `ohealock 施錠`, and `ohealock 閉めて` map to lock in the skill and eval.
- Generic `開けて` without Ohea Lock context does not trigger a physical lock command.
- `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-targets --all-features` pass.
