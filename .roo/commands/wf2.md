---
description: Trigger Workflow 2 (Complex Features) orchestration for /wf2.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Trigger

You have been triggered via `/wf2`. If you are **not** already in Orchestrator mode, switch to Orchestrator mode now. If you are already the Orchestrator, start the workflow immediately.

## Workflow 2 (Complex Features)

Execute Workflow 2 as defined in `.roo/rules-orchestrator/rules.md`.

### Flow

Ask → Architect → Orchestrator → Code/Debug → Orchestrator → Ask

### Steps

1. **Delegate to Ask:** Get a "Feature Brief".
2. **Delegate to Architect:**
   - Instruction: "Create a technical plan and Spec Kit artifacts (specify, plan) for this feature. Review architecture and integration points."
   - Await "Plan Ready" summary.
3. **Spec Kit Pipeline (Orchestration):**
   - Ensure `speckit.tasks` is generated/updated based on Architect's plan.
   - Run `speckit.analyze`.
4. **Delegate to Code:** Instruction: "Run `speckit.implement`."
5. **Verification:** Delegate to Debug if needed.
6. **Completion:** Report final status.

## Spec Kit Management

- **Auto-Run:** User does NOT type `/speckit.*`. Enforce the pipeline order.
- **Regeneration:** If spec/plan changes during the process, ensure `speckit.tasks` is regenerated before re-implementation.
- **Prerequisites:** Ensure constitution/spec/plan/tasks exist before delegating implementation.

## Context

.roo/rules-orchestrator/rules.md
