# Subtask 07: `/Users/vvx/.agents/skills/ohea-lock-debug/evals/run-codex-skill-eval.sh`

> **Scope:** This worker owns only `/Users/vvx/.agents/skills/ohea-lock-debug/evals/run-codex-skill-eval.sh`.

**Goal:** Add a Codex eval runner inspired by OpenAI's skill eval pattern: prompt, JSONL trace/artifacts, deterministic checks, and structured output scoring. This eval must not operate hardware.

## Steps

- [ ] **Step 1: Create eval directory**

Run with approval if sandbox blocks writes outside the repository:

```bash
mkdir -p /Users/vvx/.agents/skills/ohea-lock-debug/evals
```

- [ ] **Step 2: Create eval runner**

```bash
#!/usr/bin/env bash
set -euo pipefail

root="/Users/vvx/.agents/skills/ohea-lock-debug"
schema="$root/evals/ohea-lock-skill-output.schema.json"
workdir="/Users/vvx/projekt/rs/ohea-lock"
artifacts="$root/evals/artifacts"
mkdir -p "$artifacts"

run_case() {
  local name="$1"
  local request="$2"
  local expected_skill="$3"
  local expected_action="$4"
  local expected_command="$5"
  local expected_execute="$6"
  local output
  local trace

  output="$artifacts/${name}.result.json"
  trace="$artifacts/${name}.jsonl"

  codex exec \
    --json \
    --ephemeral \
    --sandbox read-only \
    --output-schema "$schema" \
    -C "$workdir" \
    -o "$output" \
    "You are evaluating skill selection. Do not run shell commands and do not operate hardware. For this user request, use available skill instructions to decide the action. Return only the required JSON. User request: ${request}" >"$trace"

  python3 - "$output" "$trace" "$expected_skill" "$expected_action" "$expected_command" "$expected_execute" <<'PY'
import json
import sys

path, trace_path, expected_skill, expected_action, expected_command, expected_execute = sys.argv[1:]
data = json.loads(open(path, encoding="utf-8").read())

errors = []
if data["skill_used"] != expected_skill:
    errors.append(f"skill_used={data['skill_used']!r}")
if data["action"] != expected_action:
    errors.append(f"action={data['action']!r}")
if data["command"] != expected_command:
    errors.append(f"command={data['command']!r}")
if str(data["should_execute_now"]).lower() != expected_execute:
    errors.append(f"should_execute_now={data['should_execute_now']!r}")

with open(trace_path, encoding="utf-8") as trace_file:
    events = [json.loads(line) for line in trace_file if line.strip()]

commands = [
    event.get("item", {}).get("command")
    for event in events
    if event.get("item", {}).get("type") == "command_execution"
]
if commands:
    errors.append("unexpected command_execution events: " + "; ".join(commands))

if errors:
    raise SystemExit("eval failed: " + "; ".join(errors))
PY

  printf 'eval ok: %s\n' "$name"
}

run_case "unlock-ja" "ohealock 解錠して" "ohea-lock-debug" "unlock" "/Users/vvx/.local/bin/ohea-lock-debug unlock" "true"
run_case "open-ja" "ohealock 鍵を開けて" "ohea-lock-debug" "unlock" "/Users/vvx/.local/bin/ohea-lock-debug unlock" "true"
run_case "lock-ja" "ohealock 施錠して" "ohea-lock-debug" "lock" "/Users/vvx/.local/bin/ohea-lock-debug lock" "true"
run_case "status-ja" "ohealock 状態確認して" "ohea-lock-debug" "status" "/Users/vvx/.local/bin/ohea-lock-debug state" "true"
run_case "ambiguous-ohealock" "ohealock 見て" "ohea-lock-debug" "status" "/Users/vvx/.local/bin/ohea-lock-debug state" "true"
run_case "generic-open" "開けて" "none" "none" "" "false"
```

- [ ] **Step 3: Mark executable**

Run:

```bash
chmod +x /Users/vvx/.agents/skills/ohea-lock-debug/evals/run-codex-skill-eval.sh
```

- [ ] **Step 4: Run eval after Subtasks 04 and 06 exist**

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

If any case fails, update only the skill wording or the demonstrably incorrect expectation, then rerun this eval and the deterministic contract test.

