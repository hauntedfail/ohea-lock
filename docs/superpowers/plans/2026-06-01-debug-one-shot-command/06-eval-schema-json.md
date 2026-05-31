# Subtask 06: `/Users/vvx/.agents/skills/ohea-lock-debug/evals/ohea-lock-skill-output.schema.json`

> **Scope:** This worker owns only `/Users/vvx/.agents/skills/ohea-lock-debug/evals/ohea-lock-skill-output.schema.json`.

**Goal:** Define the structured output contract for Codex skill eval cases.

## Steps

- [ ] **Step 1: Create eval directory**

Run with approval if sandbox blocks writes outside the repository:

```bash
mkdir -p /Users/vvx/.agents/skills/ohea-lock-debug/evals
```

- [ ] **Step 2: Create schema file**

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": [
    "skill_used",
    "action",
    "command",
    "requires_confirmation",
    "should_execute_now",
    "reason"
  ],
  "properties": {
    "skill_used": {
      "type": "string"
    },
    "action": {
      "type": "string",
      "enum": ["unlock", "lock", "status", "info", "read-all", "none"]
    },
    "command": {
      "type": "string"
    },
    "requires_confirmation": {
      "type": "boolean"
    },
    "should_execute_now": {
      "type": "boolean"
    },
    "reason": {
      "type": "string"
    }
  }
}
```

- [ ] **Step 3: Validate JSON syntax**

Run:

```bash
python3 -m json.tool /Users/vvx/.agents/skills/ohea-lock-debug/evals/ohea-lock-skill-output.schema.json >/dev/null
```

Expected: command exits successfully.

