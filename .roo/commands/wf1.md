---
description: Trigger Workflow 1 (Simple Tasks) orchestration for /wf1.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Trigger

You have been triggered via `/wf1`. If you are **not** already in Orchestrator mode, switch to Orchestrator mode now. If you are already the Orchestrator, start the workflow immediately.

## Workflow 1 (Simple Tasks)

Execute Workflow 1 as defined in `.roo/rules-orchestrator/rules.md`.

### Flow

Ask → Orchestrator → Code/Debug → Ask

### Steps

1. **Delegate to Ask:** Get a "Feature Brief" (scope, criteria).
2. **Spec Kit Pipeline:**
   - Run `speckit.constitution`
   - Run `speckit.specify` (using Feature Brief)
   - Run `speckit.clarify` **ONLY** if underspecified & user didn't skip
   - Run `speckit.plan`
   - Run `speckit.tasks`
   - Run `speckit.analyze`
3. **Delegate to Code:** Instruction: "Run `speckit.implement` to execute the tasks."
4. **Verification:** Delegate to Debug if implementation/tests fail.
5. **Completion:** Report final status to user.

## Spec Kit Management

- **Auto-Run:** User does NOT type `/speckit.*`. Enforce the pipeline order.
- **Regeneration:** If spec/plan changes during the process, ensure `speckit.tasks` is regenerated before re-implementation.
- **Prerequisites:** Ensure constitution/spec/plan/tasks exist before delegating implementation.

## Context

.roo/rules-orchestrator/rules.md
