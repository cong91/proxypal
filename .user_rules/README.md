# User Rules for S4VN Webots Project

This directory contains project-specific coding rules and conventions for the Antigravity agent. These rules ensure consistent, high-quality code generation across the project.

## Available Rules

| Rule File | Scope | Purpose |
|-----------|-------|---------|
| [coding-principles.md](coding-principles.md) | All files | Core DRY, KISS, SOLID principles |
| [no-fallback-policy.md](no-fallback-policy.md) | All code | Strict fail-fast error handling |
| [python-rules.md](python-rules.md) | `**/*.py` | Python-specific conventions |
| [typescript-nextjs-rules.md](typescript-nextjs-rules.md) | `**/*.ts`, `**/*.tsx` | TypeScript, Next.js, React rules |
| [docker-rules.md](docker-rules.md) | Docker files | Dockerfile, docker-compose, GPU config |
| [project-specific.md](project-specific.md) | All files | Webots project conventions |
| [database-testing-websocket.md](database-testing-websocket.md) | Specific files | DB, Testing, WebSocket rules |

## Critical Rules

### 🔴 No Fallback Policy (MANDATORY)
**Never implement fallback logic.** If GPU fails, do NOT fall back to CPU. If a dependency is missing, FAIL IMMEDIATELY with clear instructions.

See: [no-fallback-policy.md](no-fallback-policy.md)

### 📝 Documentation Hygiene
- Research existing files before creating new ones
- Update > Create (prefer updating existing files)
- Maximum 2 documentation files per task
- No `FINAL_*`, `*_COMPLETE.md`, `*_V2.md` filenames

### 🏗️ SDD (Spec-Driven Development)
- Plan → Approve → Execute workflow
- Create specs before code
- Use workflows in `.agent/workflows/`

## Using Workflows

Workflows are available in [.agent/workflows/](../.agent/workflows/):
- `/deploy-simulation` - Deploy S4VN-Simulation server
- `/build-webots-controller` - Build Webots controllers
- `/sdd-brief` - Create 30-minute feature briefs
- `/sdd-implement` - Execute SDD implementations

See [.agent/workflows/README.md](../.agent/workflows/README.md) for details.

## Quick Reference

### Error Handling Pattern
```python
# ✅ CORRECT - Fail fast
if not gpu_available:
    error = "GPU not found. Install NVIDIA Container Toolkit: sudo apt install nvidia-container-toolkit"
    logging.error(error)
    raise RuntimeError(error)

# ❌ FORBIDDEN - Fallback logic
if not gpu_available:
    use_cpu_rendering()  # NO!
```

### Python Conventions
- `snake_case` for functions/variables
- `PascalCase` for classes
- Type hints required
- PEP 8 style guide

### TypeScript Conventions
- `camelCase` for functions/variables
- `PascalCase` for Components/Types
- Use Server Components by default
- Proper error handling

### Docker Conventions
- Use `Dockerfile.default` for Webots
- NVIDIA GPU support required
- No software rendering fallback
- Environment variables for config

## Enforcement

These rules are automatically applied by the Antigravity agent. Violations are considered bugs and must be fixed immediately.
