# Subtask 04: `/Users/vvx/.agents/skills/ohea-lock-debug/SKILL.md`

> **Scope:** This worker owns only `/Users/vvx/.agents/skills/ohea-lock-debug/SKILL.md`.

**Goal:** Create a global skill that triggers on Ohea Lock/OheaLock/ohealock requests, including Japanese requests such as `鍵を開けて`, `解錠`, and `開けて`, and maps them to the installed one-shot binary.

## Steps

- [ ] **Step 1: Create the skill directory**

Run with approval if sandbox blocks writes outside the repository:

```bash
mkdir -p /Users/vvx/.agents/skills/ohea-lock-debug
```

- [ ] **Step 2: Create `SKILL.md`**

````markdown
---
name: ohea-lock-debug
description: Use when the user asks to operate an Ohea Lock/OheaLock/ohealock BLE smart lock, including Japanese requests like 鍵を開けて, 解錠, 開けて, 鍵を閉めて, 施錠, 閉めて, 状態確認, or English lock, unlock, open, close, status.
---

# Ohea Lock Debug

Use this skill to operate the local Ohea Lock through the repository-built debug binary. Treat this as a physical-device operation: choose the smallest single command that matches the user's current request.

## Preconditions

- The Ohea Lock is powered, in range, and already paired or pairable through the host Bluetooth stack.
- The debug binary exists at `/Users/vvx/.local/bin/ohea-lock-debug`.
- If the binary is missing or stale, rebuild it from `/Users/vvx/projekt/rs/ohea-lock` with:

```bash
scripts/install-debug-bin.sh
```

## Triggering

Use this skill when the current user request names `ohealock`, `Ohea Lock`, `OheaLock`, `おへやろっく`, or clearly continues an Ohea Lock operation already in context.

Do not use this skill for generic "open", "close", "開けて", or "閉めて" requests unless the request also identifies the Ohea Lock or the active context is already the Ohea Lock.

## Intent Mapping

| User wording | Action | Command |
|---|---|---|
| `ohealock 鍵を開けて`, `ohealock 解錠`, `ohealock 開けて`, `unlock`, `open` | unlock | `/Users/vvx/.local/bin/ohea-lock-debug unlock` |
| `ohealock 鍵を閉めて`, `ohealock 施錠`, `ohealock 閉めて`, `lock`, `close` | lock | `/Users/vvx/.local/bin/ohea-lock-debug lock` |
| `ohealock 状態確認`, `status`, `state`, `今開いてる?`, `今閉まってる?` | status check | `/Users/vvx/.local/bin/ohea-lock-debug state` |
| `ohealock 情報`, `info`, `battery`, `firmware` | device info | `/Users/vvx/.local/bin/ohea-lock-debug info` |
| `ohealock 全部読んで`, `read all`, `characteristics` | read all characteristics | `/Users/vvx/.local/bin/ohea-lock-debug read-all` |
| Ambiguous Ohea Lock request without a lock/unlock verb | status check | `/Users/vvx/.local/bin/ohea-lock-debug state` |

## Safety

- `lock`, `unlock`, and `toggle` change a physical lock state. Run them only when the current user request explicitly asks for that exact action.
- `ohealock 鍵を開けて`, `ohealock 解錠`, and `ohealock 開けて` are explicit unlock requests.
- `ohealock 鍵を閉めて`, `ohealock 施錠`, and `ohealock 閉めて` are explicit lock requests.
- Do not run repeated lock/unlock loops.
- If the user's request is ambiguous, run `state` or `info` first and report the result.
- If multiple Ohea Lock devices are detected, the binary refuses one-shot operation. Use interactive mode from the repository checkout.
- Do not substitute a different binary path. If `/Users/vvx/.local/bin/ohea-lock-debug` is missing, rebuild it from `/Users/vvx/projekt/rs/ohea-lock`.

## Commands

Read current state:

```bash
/Users/vvx/.local/bin/ohea-lock-debug state
```

Show device information:

```bash
/Users/vvx/.local/bin/ohea-lock-debug info
```

Lock once:

```bash
/Users/vvx/.local/bin/ohea-lock-debug lock
```

Unlock once:

```bash
/Users/vvx/.local/bin/ohea-lock-debug unlock
```

Toggle once:

```bash
/Users/vvx/.local/bin/ohea-lock-debug toggle
```

Read all known characteristics:

```bash
/Users/vvx/.local/bin/ohea-lock-debug read-all
```

Interactive fallback:

```bash
cd /Users/vvx/projekt/rs/ohea-lock
cargo run --example debug --features btleplug-support
```

## Verification

Use `--help` when verifying installation without touching the lock:

```bash
/Users/vvx/.local/bin/ohea-lock-debug --help
```
````

- [ ] **Step 3: Run skill contract test after Subtask 05 exists**

Run:

```bash
python3 /Users/vvx/.agents/skills/ohea-lock-debug/scripts/test_skill_contract.py
```

Expected:

```text
skill contract ok
```

