# Subtask 05: `/Users/vvx/.agents/skills/ohea-lock-debug/scripts/test_skill_contract.py`

> **Scope:** This worker owns only `/Users/vvx/.agents/skills/ohea-lock-debug/scripts/test_skill_contract.py`.

**Goal:** Add deterministic checks that the skill follows basic skill best practices and contains the required Japanese/English triggers, command mappings, and safety guardrails.

## Steps

- [ ] **Step 1: Create script directory**

Run with approval if sandbox blocks writes outside the repository:

```bash
mkdir -p /Users/vvx/.agents/skills/ohea-lock-debug/scripts
```

- [ ] **Step 2: Create `test_skill_contract.py`**

```python
#!/usr/bin/env python3
from pathlib import Path
import re
import sys

ROOT = Path("/Users/vvx/.agents/skills/ohea-lock-debug")
SKILL = ROOT / "SKILL.md"


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


text = SKILL.read_text(encoding="utf-8")
if not text.startswith("---\n"):
    fail("SKILL.md must start with YAML frontmatter")

parts = text.split("---\n", 2)
if len(parts) != 3:
    fail("SKILL.md must contain one YAML frontmatter block")

frontmatter = parts[1]
body = parts[2]

name = re.search(r"^name:\s*(.+)$", frontmatter, re.MULTILINE)
description = re.search(r"^description:\s*(.+)$", frontmatter, re.MULTILINE)

if not name or name.group(1).strip() != "ohea-lock-debug":
    fail("name must be ohea-lock-debug")

if not description:
    fail("description is required")

description_text = description.group(1).strip()
if not description_text.startswith("Use when "):
    fail("description must start with 'Use when '")

if len(frontmatter) > 1024:
    fail("frontmatter must be at most 1024 characters")

required_terms = [
    "ohealock",
    "Ohea Lock",
    "鍵を開けて",
    "解錠",
    "開けて",
    "鍵を閉めて",
    "施錠",
    "閉めて",
    "状態確認",
    "unlock",
    "lock",
    "status",
    "/Users/vvx/.local/bin/ohea-lock-debug unlock",
    "/Users/vvx/.local/bin/ohea-lock-debug lock",
    "/Users/vvx/.local/bin/ohea-lock-debug state",
]

missing = [term for term in required_terms if term not in text]
if missing:
    fail("missing required trigger or command terms: " + ", ".join(missing))

required_sections = [
    "## Triggering",
    "## Intent Mapping",
    "## Safety",
    "## Commands",
    "## Verification",
]

missing_sections = [section for section in required_sections if section not in body]
if missing_sections:
    fail("missing required sections: " + ", ".join(missing_sections))

if "Do not use this skill for generic" not in text:
    fail("skill must prevent generic open/close trigger overreach")

if "Run them only when the current user request explicitly asks" not in text:
    fail("skill must constrain physical lock-changing commands")

print("skill contract ok")
```

- [ ] **Step 3: Mark executable**

Run:

```bash
chmod +x /Users/vvx/.agents/skills/ohea-lock-debug/scripts/test_skill_contract.py
```

- [ ] **Step 4: Run after Subtask 04 exists**

Run:

```bash
python3 /Users/vvx/.agents/skills/ohea-lock-debug/scripts/test_skill_contract.py
```

Expected:

```text
skill contract ok
```

